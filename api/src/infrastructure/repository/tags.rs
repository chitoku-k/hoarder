use std::{
    cell::RefCell,
    collections::HashSet,
    rc::{Rc, Weak},
};

use anyhow::Context;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use derive_more::Constructor;
use futures::TryStreamExt;
use indexmap::{IndexMap, IndexSet};
use sea_query::{Alias, Cond, Condition, Expr, Iden, JoinType, LikeExpr, LockType, Order, PostgresQueryBuilder, Query, SelectStatement};
use sea_query_binder::SqlxBinder;
use sqlx::{Acquire, FromRow, PgPool, Postgres, Transaction, PgConnection};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    domain::{
        entity::tags::{Tag, TagDepth, TagError, TagId},
        repository::{tags::TagsRepository, DeleteResult, OrderDirection},
    },
    infrastructure::repository::{
        expr::array::ArrayExpr,
        sea_query_uuid_value,
    },
};

#[derive(Clone, Constructor)]
pub struct PostgresTagsRepository {
    pool: PgPool,
}

#[derive(Debug, FromRow)]
struct PostgresTagRow {
    id: Uuid,
    name: String,
    kana: String,
    aliases: Vec<String>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Debug, FromRow)]
struct PostgresTagDescendantRow {
    descendant_id: Uuid,
}

#[derive(Debug, FromRow)]
struct PostgresTagRelativeRow {
    distance: i32,

    ancestor_id: Uuid,
    ancestor_name: String,
    ancestor_kana: String,
    ancestor_aliases: Vec<String>,
    ancestor_created_at: NaiveDateTime,
    ancestor_updated_at: NaiveDateTime,

    descendant_id: Uuid,
    descendant_name: String,
    descendant_kana: String,
    descendant_aliases: Vec<String>,
    descendant_created_at: NaiveDateTime,
    descendant_updated_at: NaiveDateTime,
}

#[derive(Clone, Debug)]
struct TagRelation {
    id: Uuid,
    name: String,
    kana: String,
    aliases: Vec<String>,
    parent: Weak<RefCell<Self>>,
    children: Vec<Rc<RefCell<Self>>>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Iden)]
pub enum PostgresTag {
    #[iden = "tags"]
    Table,
    Id,
    Name,
    Kana,
    Aliases,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
pub enum PostgresTagPath {
    #[iden = "tag_paths"]
    Table,
    AncestorId,
    DescendantId,
    Distance,
}

#[derive(Iden)]
enum PostgresTagRelation {
    AncestorId,
    AncestorName,
    AncestorKana,
    AncestorAliases,
    AncestorCreatedAt,
    AncestorUpdatedAt,
    DescendantId,
    DescendantName,
    DescendantKana,
    DescendantAliases,
    DescendantCreatedAt,
    DescendantUpdatedAt,
}

#[derive(Debug, Eq, Error, PartialEq)]
pub enum PostgresTagError {
    #[error("root tag cannot be updated")]
    RootUpdated,
    #[error("root tag cannot be attached")]
    RootAttached,
    #[error("root tag cannot be detached")]
    RootDetached,
    #[error("root tag cannot be deleted")]
    RootDeleted,
    #[error("{0} children exist")]
    ChildrenExist(usize),
}

sea_query_uuid_value!(TagId);

impl Default for TagRelation {
    fn default() -> Self {
        Self {
            id: Default::default(),
            name: Default::default(),
            kana: Default::default(),
            aliases: Default::default(),
            parent: Default::default(),
            children: Default::default(),
            created_at: NaiveDateTime::MIN,
            updated_at: NaiveDateTime::MIN,
        }
    }
}

impl From<PostgresTagRow> for Tag {
    fn from(row: PostgresTagRow) -> Self {
        Self {
            id: row.id.into(),
            name: row.name,
            kana: row.kana,
            aliases: row.aliases.into(),
            created_at: row.created_at,
            updated_at: row.updated_at,
            ..Default::default()
        }
    }
}

impl From<PostgresTagRelativeRow> for (i32, TagRelation, TagRelation) {
    fn from(row: PostgresTagRelativeRow) -> Self {
        (
            row.distance,
            TagRelation {
                id: row.ancestor_id,
                name: row.ancestor_name,
                kana: row.ancestor_kana,
                aliases: row.ancestor_aliases,
                created_at: row.ancestor_created_at,
                updated_at: row.ancestor_updated_at,
                ..Default::default()
            },
            TagRelation {
                id: row.descendant_id,
                name: row.descendant_name,
                kana: row.descendant_kana,
                aliases: row.descendant_aliases,
                created_at: row.descendant_created_at,
                updated_at: row.descendant_updated_at,
                ..Default::default()
            },
        )
    }
}

impl FromIterator<PostgresTagRelativeRow> for IndexMap<TagId, Rc<RefCell<TagRelation>>> {
    fn from_iter<T>(rows: T) -> Self
    where
        T: IntoIterator<Item = PostgresTagRelativeRow>,
    {
        let mut tags = IndexMap::new();

        for tag in rows {
            let (distance, ancestor, descendant) = tag.into();
            match distance {
                0 => {
                    let ancestor_id = ancestor.id;
                    let ancestor = Rc::new(RefCell::new(ancestor));
                    tags.insert(TagId::from(ancestor_id), ancestor);
                },
                _ => {
                    {
                        let ancestor = tags.get(&TagId::from(ancestor.id)).and_then(|r| r.try_borrow_mut().ok());
                        let descendant = tags.get(&TagId::from(descendant.id));

                        if let (Some(mut ancestor), Some(descendant)) = (ancestor, descendant) {
                            ancestor.children.push(descendant.clone());
                        }
                    }
                    {
                        let ancestor = tags.get(&TagId::from(ancestor.id));
                        let descendant = tags.get(&TagId::from(descendant.id)).and_then(|r| r.try_borrow_mut().ok());

                        if let (Some(ancestor), Some(mut descendant)) = (ancestor, descendant) {
                            descendant.parent = Rc::downgrade(ancestor);
                        }
                    }
                },
            }
        }

        tags
    }
}

fn extract(rc: Rc<RefCell<TagRelation>>, depth: TagDepth) -> Tag {
    let relation = Rc::try_unwrap(rc)
        .unwrap_or_else(|rc| (*rc).clone())
        .into_inner();

    let parent = depth
        .has_parent()
        .then_some(relation.parent)
        .and_then(|parent| parent.upgrade())
        .map(|relation| extract(relation, TagDepth::new(depth.parent() - 1, 0)))
        .map(Box::new);

    let children = depth
        .has_children()
        .then_some(relation.children)
        .unwrap_or_default()
        .into_iter()
        .map(|relation| extract(relation, TagDepth::new(0, depth.children() - 1)))
        .collect();

    Tag {
        id: relation.id.into(),
        name: relation.name,
        kana: relation.kana,
        aliases: relation.aliases.into(),
        parent,
        children,
        created_at: relation.created_at,
        updated_at: relation.updated_at,
    }
}

pub async fn fetch_tag_relatives<T>(conn: &mut PgConnection, ids: T, depth: TagDepth, root: bool) -> anyhow::Result<Vec<Tag>>
where
    T: IntoIterator<Item = TagId>,
{
    let ids: HashSet<_> = ids.into_iter().collect();
    let tag_ancestors = Alias::new("tag_ancestors");
    let tag_descendants = Alias::new("tag_descendants");

    let relatives = {
        let conditions = Cond::any()
            .add_option(depth.has_parent().then(|| {
                Expr::col(PostgresTagPath::DescendantId).in_subquery(
                    Query::select()
                        .from(PostgresTagPath::Table)
                        .column(PostgresTagPath::AncestorId)
                        .and_where(Expr::col(PostgresTagPath::Distance).lte(depth.parent()))
                        .and_where(Expr::col(PostgresTagPath::DescendantId).is_in(ids.iter().cloned()))
                        .take()
                    )
            }))
            .add_option(depth.has_children().then(|| {
                Expr::col(PostgresTagPath::AncestorId).in_subquery(
                    Query::select()
                        .from(PostgresTagPath::Table)
                        .column(PostgresTagPath::DescendantId)
                        .and_where(Expr::col(PostgresTagPath::Distance).lte(depth.children()))
                        .and_where(Expr::col(PostgresTagPath::AncestorId).is_in(ids.iter().cloned()))
                        .take()
                    )
            }));

        if conditions.is_empty() {
            Cond::all()
        } else {
            conditions
        }
    };

    let (sql, values) = Query::select()
        .column(PostgresTagPath::Distance)
        .expr_as(Expr::col((tag_ancestors.clone(), PostgresTag::Id)), PostgresTagRelation::AncestorId)
        .expr_as(Expr::col((tag_ancestors.clone(), PostgresTag::Name)), PostgresTagRelation::AncestorName)
        .expr_as(Expr::col((tag_ancestors.clone(), PostgresTag::Kana)), PostgresTagRelation::AncestorKana)
        .expr_as(Expr::col((tag_ancestors.clone(), PostgresTag::Aliases)), PostgresTagRelation::AncestorAliases)
        .expr_as(Expr::col((tag_ancestors.clone(), PostgresTag::CreatedAt)), PostgresTagRelation::AncestorCreatedAt)
        .expr_as(Expr::col((tag_ancestors.clone(), PostgresTag::UpdatedAt)), PostgresTagRelation::AncestorUpdatedAt)
        .expr_as(Expr::col((tag_descendants.clone(), PostgresTag::Id)), PostgresTagRelation::DescendantId)
        .expr_as(Expr::col((tag_descendants.clone(), PostgresTag::Name)), PostgresTagRelation::DescendantName)
        .expr_as(Expr::col((tag_descendants.clone(), PostgresTag::Kana)), PostgresTagRelation::DescendantKana)
        .expr_as(Expr::col((tag_descendants.clone(), PostgresTag::Aliases)), PostgresTagRelation::DescendantAliases)
        .expr_as(Expr::col((tag_descendants.clone(), PostgresTag::CreatedAt)), PostgresTagRelation::DescendantCreatedAt)
        .expr_as(Expr::col((tag_descendants.clone(), PostgresTag::UpdatedAt)), PostgresTagRelation::DescendantUpdatedAt)
        .from(PostgresTagPath::Table)
        .join_as(
            JoinType::InnerJoin,
            PostgresTag::Table,
            tag_ancestors.clone(),
            Expr::col((tag_ancestors.clone(), PostgresTag::Id))
                .equals((PostgresTagPath::Table, PostgresTagPath::AncestorId)),
        )
        .join_as(
            JoinType::InnerJoin,
            PostgresTag::Table,
            tag_descendants.clone(),
            Expr::col((tag_descendants.clone(), PostgresTag::Id))
                .equals((PostgresTagPath::Table, PostgresTagPath::DescendantId)),
        )
        .and_where(Expr::col(PostgresTagPath::Distance).lte(1i32))
        .order_by(PostgresTagPath::Distance, Order::Asc)
        .order_by((tag_ancestors, PostgresTag::Kana), Order::Asc)
        .order_by((tag_descendants, PostgresTag::Kana), Order::Asc)
        .cond_where(relatives)
        .build_sqlx(PostgresQueryBuilder);

    let rows: Vec<_> = sqlx::query_as_with::<_, PostgresTagRelativeRow, _>(&sql, values)
        .fetch(&mut *conn)
        .try_collect()
        .await?;

    let mut relations: IndexMap<_, _> = rows.into_iter().collect();
    let tags =
        if root {
            relations
                .remove(&TagId::root())
                .map(|relation| extract(relation, depth).children)
                .unwrap_or_default()
        } else {
            relations.remove(&TagId::root());
            relations
                .values()
                .map(|relation| extract(relation.clone(), depth))
                .filter(|tag| ids.contains(&tag.id))
                .collect()
        };

    Ok(tags)
}

fn ancestor_relations(id: TagId) -> SelectStatement {
    Query::select()
        .columns([
            PostgresTagPath::AncestorId,
            PostgresTagPath::DescendantId,
            PostgresTagPath::Distance,
        ])
        .cond_where(
            Condition::all()
                .add(Expr::col(PostgresTagPath::DescendantId).eq(id))
                .add(Expr::col(PostgresTagPath::AncestorId).ne(Expr::col(PostgresTagPath::DescendantId)))
        )
        .take()
}

fn descendant_relations(id: TagId) -> SelectStatement {
    let tag_path_ancestors = Alias::new("tag_path_ancestors");
    let tag_path_descendants = Alias::new("tag_path_descendants");

    Query::select()
        .expr(Expr::col((tag_path_ancestors.clone(), PostgresTagPath::AncestorId)))
        .expr(Expr::col((tag_path_descendants.clone(), PostgresTagPath::DescendantId)))
        .expr(Expr::col((tag_path_ancestors.clone(), PostgresTagPath::Distance))
            .add(Expr::col((tag_path_descendants.clone(), PostgresTagPath::Distance))))
        .from_as(PostgresTagPath::Table, tag_path_ancestors.clone())
        .join_as(
            JoinType::InnerJoin,
            PostgresTagPath::Table,
            tag_path_descendants.clone(),
            Expr::col((tag_path_ancestors.clone(), PostgresTagPath::DescendantId))
                .equals((tag_path_descendants.clone(), PostgresTagPath::AncestorId)),
        )
        .cond_where(
            Condition::all()
                .add(Expr::col((tag_path_ancestors.clone(), PostgresTagPath::DescendantId)).eq(id))
                .add(Expr::col((tag_path_descendants.clone(), PostgresTagPath::AncestorId)).eq(id))
                .add(Expr::col((tag_path_ancestors, PostgresTagPath::AncestorId)).ne(id))
                .add(Expr::col((tag_path_descendants, PostgresTagPath::DescendantId)).ne(id))
        )
        .take()
}

async fn attach_parent(tx: &mut Transaction<'_, Postgres>, id: TagId, parent_id: TagId) -> anyhow::Result<()> {
    let mut tx = tx.begin().await?;

    let (sql, values) = Query::insert()
        .into_table(PostgresTagPath::Table)
        .columns([
            PostgresTagPath::AncestorId,
            PostgresTagPath::DescendantId,
            PostgresTagPath::Distance,
        ])
        .select_from(
            Query::select()
                .exprs([
                    Expr::col(PostgresTagPath::AncestorId).into(),
                    Expr::val(id).into(),
                    Expr::col(PostgresTagPath::Distance).add(1i32),
                ])
                .from(PostgresTagPath::Table)
                .and_where(Expr::col(PostgresTagPath::DescendantId).eq(parent_id))
                .take()
        )?
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values).execute(&mut tx).await?;

    let (sql, values) = Query::insert()
        .into_table(PostgresTagPath::Table)
        .columns([
            PostgresTagPath::AncestorId,
            PostgresTagPath::DescendantId,
            PostgresTagPath::Distance,
        ])
        .select_from(descendant_relations(id))?
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values).execute(&mut tx).await?;

    tx.commit().await?;
    Ok(())
}

async fn detach_parent(tx: &mut Transaction<'_, Postgres>, id: TagId) -> anyhow::Result<()> {
    let mut tx = tx.begin().await?;

    let (sql, values) = Query::delete()
        .from_table(PostgresTagPath::Table)
        .and_where(
            Expr::tuple([
                Expr::col(PostgresTagPath::AncestorId).into(),
                Expr::col(PostgresTagPath::DescendantId).into(),
                Expr::col(PostgresTagPath::Distance).into(),
            ])
            .in_subquery(descendant_relations(id))
        )
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values).execute(&mut tx).await?;

    let (sql, values) = Query::delete()
        .from_table(PostgresTagPath::Table)
        .and_where(
            Expr::tuple([
                Expr::col(PostgresTagPath::AncestorId).into(),
                Expr::col(PostgresTagPath::DescendantId).into(),
                Expr::col(PostgresTagPath::Distance).into(),
            ])
            .in_subquery(ancestor_relations(id))
        )
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values).execute(&mut tx).await?;

    tx.commit().await?;
    Ok(())
}

#[async_trait]
impl TagsRepository for PostgresTagsRepository {
    async fn create(&self, name: &str, kana: &str, aliases: &[String], parent_id: Option<TagId>, depth: TagDepth) -> anyhow::Result<Tag> {
        let mut tx = self.pool.begin().await?;

        let (sql, values) = Query::insert()
            .into_table(PostgresTag::Table)
            .columns([
                PostgresTag::Name,
                PostgresTag::Kana,
                PostgresTag::Aliases,
            ])
            .values([
                Expr::val(name).into(),
                Expr::val(kana).into(),
                ArrayExpr::val(aliases)?,
            ])?
            .returning(
                Query::returning()
                    .columns([
                        PostgresTag::Id,
                        PostgresTag::Name,
                        PostgresTag::Kana,
                        PostgresTag::Aliases,
                        PostgresTag::CreatedAt,
                        PostgresTag::UpdatedAt,
                    ])
            )
            .build_sqlx(PostgresQueryBuilder);

        let tag: Tag = sqlx::query_as_with::<_, PostgresTagRow, _>(&sql, values)
            .fetch_one(&mut tx)
            .await?
            .into();

        let (sql, values) = Query::insert()
            .into_table(PostgresTagPath::Table)
            .columns([
                PostgresTagPath::AncestorId,
                PostgresTagPath::DescendantId,
                PostgresTagPath::Distance,
            ])
            .values([
                tag.id.into(),
                tag.id.into(),
                0.into(),
            ])?
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut tx).await?;

        let parent_id = parent_id.unwrap_or_default();
        attach_parent(&mut tx, tag.id, parent_id).await?;

        let mut relatives = fetch_tag_relatives(&mut tx, [tag.id], depth, false).await?;
        let tag = relatives.pop().context(TagError::NotFound(tag.id))?;

        tx.commit().await?;
        Ok(tag)
    }

    async fn fetch_by_ids<T>(&self, ids: T, depth: TagDepth) -> anyhow::Result<Vec<Tag>>
    where
        T: IntoIterator<Item = TagId> + Send + Sync + 'static,
    {
        let mut conn = self.pool.acquire().await?;

        let tags = fetch_tag_relatives(&mut conn, ids, depth, false).await?;
        Ok(tags)
    }

    async fn fetch_by_name_or_alias_like(&self, name_or_alias_like: &str, depth: TagDepth) -> anyhow::Result<Vec<Tag>> {
        let mut conn = self.pool.acquire().await?;

        let tags_aliases = Alias::new("tags_aliases");
        let alias = Alias::new("alias");

        let name_or_alias_like = format!(
            "%{}%",
            name_or_alias_like
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_"),
        );

        let (sql, values) = Query::select()
            .columns([
                (PostgresTag::Table, PostgresTag::Id),
                (PostgresTag::Table, PostgresTag::Name),
                (PostgresTag::Table, PostgresTag::Kana),
                (PostgresTag::Table, PostgresTag::Aliases),
                (PostgresTag::Table, PostgresTag::CreatedAt),
                (PostgresTag::Table, PostgresTag::UpdatedAt),
            ])
            .from(PostgresTag::Table)
            .join_subquery(
                JoinType::LeftJoin,
                Query::select()
                    .column(PostgresTag::Id)
                    .expr_as(ArrayExpr::unnest(Expr::col(PostgresTag::Aliases)), alias.clone())
                    .from(PostgresTag::Table)
                    .take(),
                tags_aliases.clone(),
                Expr::col((tags_aliases, PostgresTag::Id)).equals((PostgresTag::Table, PostgresTag::Id)),
            )
            .cond_where(
                Cond::any()
                    .add(Expr::col(PostgresTag::Name).like(LikeExpr::str(&name_or_alias_like)))
                    .add(Expr::col(alias).like(LikeExpr::str(&name_or_alias_like))),
            )
            .build_sqlx(PostgresQueryBuilder);

        let ids: Vec<_> = sqlx::query_as_with::<_, PostgresTagRow, _>(&sql, values)
            .fetch(&mut conn)
            .map_ok(|r| TagId::from(r.id))
            .try_collect()
            .await?;

        let tags = fetch_tag_relatives(&mut conn, ids, depth, false).await?;
        Ok(tags)
    }

    async fn fetch_all(&self, depth: TagDepth, root: bool, after: Option<(String, TagId)>, before: Option<(String, TagId)>, order: OrderDirection, limit: u64) -> anyhow::Result<Vec<Tag>> {
        let mut conn = self.pool.acquire().await?;

        let mut query = Query::select();
        query
            .columns([
                (PostgresTag::Table, PostgresTag::Id),
                (PostgresTag::Table, PostgresTag::Name),
                (PostgresTag::Table, PostgresTag::Kana),
                (PostgresTag::Table, PostgresTag::Aliases),
                (PostgresTag::Table, PostgresTag::CreatedAt),
                (PostgresTag::Table, PostgresTag::UpdatedAt),
            ])
            .from(PostgresTag::Table)
            .and_where(Expr::col((PostgresTag::Table, PostgresTag::Id)).ne(TagId::root()))
            .and_where_option(
                after.map(|(kana, tag_id)| {
                    Expr::tuple([
                        Expr::col((PostgresTag::Table, PostgresTag::Kana)).into(),
                        Expr::col((PostgresTag::Table, PostgresTag::Id)).into(),
                    ]).gt(Expr::tuple([
                        Expr::value(kana),
                        Expr::value(tag_id),
                    ]))
                })
            )
            .and_where_option(
                before.map(|(kana, tag_id)| {
                    Expr::tuple([
                        Expr::col((PostgresTag::Table, PostgresTag::Kana)).into(),
                        Expr::col((PostgresTag::Table, PostgresTag::Id)).into(),
                    ]).lt(Expr::tuple([
                        Expr::value(kana),
                        Expr::value(tag_id),
                    ]))
                })
            )
            .order_by((PostgresTag::Table, PostgresTag::Kana), order.into())
            .order_by((PostgresTag::Table, PostgresTag::Id), order.into())
            .limit(limit);

        if root {
            query
                .join(
                    JoinType::InnerJoin,
                    PostgresTagPath::Table,
                    Expr::col((PostgresTagPath::Table, PostgresTagPath::DescendantId))
                        .equals((PostgresTag::Table, PostgresTag::Id)),
                )
                .and_where(Expr::col((PostgresTagPath::Table, PostgresTagPath::AncestorId)).eq(TagId::root()))
                .and_where(Expr::col((PostgresTagPath::Table, PostgresTagPath::Distance)).eq(1));
        }

        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let ids: IndexSet<_> = sqlx::query_as_with::<_, PostgresTagRow, _>(&sql, values)
            .fetch(&self.pool)
            .map_ok(|r| TagId::from(r.id))
            .try_collect()
            .await?;

        let depth = match root {
            true => TagDepth::new(1, depth.children() + 1),
            false => depth,
        };

        let mut tags = fetch_tag_relatives(&mut conn, ids.iter().cloned(), depth, root).await?;
        tags.sort_unstable_by(|a, b| {
            Option::zip(ids.get_index_of(&a.id), ids.get_index_of(&b.id))
                .map(|(a, b)| Ord::cmp(&a, &b))
                .unwrap_or_else(|| Ord::cmp(&a.id, &b.id))
        });

        Ok(tags)
    }

    async fn update_by_id<T, U>(&self, id: TagId, name: Option<String>, kana: Option<String>, add_aliases: T, remove_aliases: U, depth: TagDepth) -> anyhow::Result<Tag>
    where
        T: IntoIterator<Item = String> + Send + Sync + 'static,
        U: IntoIterator<Item = String> + Send + Sync + 'static,
    {
        if id.is_root() {
            return Err(PostgresTagError::RootUpdated)?;
        }

        let mut tx = self.pool.begin().await?;
        let (sql, values) = Query::select()
            .columns([
                PostgresTag::Id,
                PostgresTag::Name,
                PostgresTag::Kana,
                PostgresTag::Aliases,
                PostgresTag::CreatedAt,
                PostgresTag::UpdatedAt,
            ])
            .from(PostgresTag::Table)
            .and_where(Expr::col(PostgresTag::Id).eq(id))
            .lock(LockType::Update)
            .build_sqlx(PostgresQueryBuilder);

        let mut tag: Tag = sqlx::query_as_with::<_, PostgresTagRow, _>(&sql, values)
            .fetch_optional(&mut tx)
            .await?
            .map(Into::into)
            .context(TagError::NotFound(id))?;

        let name = name.unwrap_or(tag.name);
        let kana = kana.unwrap_or(tag.kana);
        let aliases = {
            tag.aliases.add_all(add_aliases);
            tag.aliases.remove_all(remove_aliases);
            tag.aliases
        };

        let (sql, values) = Query::update()
            .table(PostgresTag::Table)
            .value(PostgresTag::Name, name)
            .value(PostgresTag::Kana, kana)
            .value(PostgresTag::Aliases, ArrayExpr::val(aliases)?)
            .value(PostgresTag::UpdatedAt, Expr::current_timestamp())
            .and_where(Expr::col(PostgresTag::Id).eq(id))
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut tx).await?;

        let mut relatives = fetch_tag_relatives(&mut tx, [tag.id], depth, false).await?;
        let tag = relatives.pop().context(TagError::NotFound(tag.id))?;

        tx.commit().await?;
        Ok(tag)
    }

    async fn attach_by_id(&self, id: TagId, parent_id: TagId, depth: TagDepth) -> anyhow::Result<Tag> {
        if id.is_root() {
            return Err(PostgresTagError::RootAttached)?;
        }

        let mut tx = self.pool.begin().await?;
        detach_parent(&mut tx, id).await?;
        attach_parent(&mut tx, id, parent_id).await?;

        let mut relatives = fetch_tag_relatives(&mut tx, [id], depth, false).await?;
        let tag = relatives.pop().context(TagError::NotFound(id))?;

        tx.commit().await?;
        Ok(tag)
    }

    async fn detach_by_id(&self, id: TagId, depth: TagDepth) -> anyhow::Result<Tag> {
        if id.is_root() {
            return Err(PostgresTagError::RootDetached)?;
        }

        let mut tx = self.pool.begin().await?;
        detach_parent(&mut tx, id).await?;
        attach_parent(&mut tx, id, TagId::root()).await?;

        let mut relatives = fetch_tag_relatives(&mut tx, [id], depth, false).await?;
        let tag = relatives.pop().context(TagError::NotFound(id))?;

        tx.commit().await?;
        Ok(tag)
    }

    async fn delete_by_id(&self, id: TagId, recursive: bool) -> anyhow::Result<DeleteResult> {
        if id.is_root() {
            return Err(PostgresTagError::RootDeleted)?;
        }

        let mut tx = self.pool.begin().await?;

        let (sql, values) = Query::select()
            .columns([
                PostgresTagPath::AncestorId,
                PostgresTagPath::DescendantId,
            ])
            .from(PostgresTagPath::Table)
            .and_where(Expr::col(PostgresTagPath::AncestorId).eq(id))
            .lock(LockType::Update)
            .build_sqlx(PostgresQueryBuilder);

        let ids: Vec<_> = sqlx::query_as_with::<_, PostgresTagDescendantRow, _>(&sql, values)
            .fetch(&mut tx)
            .map_ok(|r| r.descendant_id)
            .try_collect()
            .await?;

        if !recursive && ids.len() > 1 {
            return Err(PostgresTagError::ChildrenExist(ids.len() - 1))?;
        }

        let (sql, values) = Query::delete()
            .from_table(PostgresTag::Table)
            .and_where(Expr::col(PostgresTag::Id).is_in(ids))
            .build_sqlx(PostgresQueryBuilder);

        let affected = sqlx::query_with(&sql, values)
            .execute(&mut tx)
            .await?
            .rows_affected();

        tx.commit().await?;

        match affected {
            0 => Ok(DeleteResult::NotFound),
            count => Ok(DeleteResult::Deleted(count)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use chrono::NaiveDate;
    use futures::TryStreamExt;
    use pretty_assertions::{assert_eq, assert_ne};
    use sqlx::Row;
    use test_context::test_context;
    use uuid::uuid;

    use crate::{
        domain::entity::tags::AliasSet,
        infrastructure::repository::tests::DatabaseContext,
    };

    use super::*;

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn create_with_parent_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.create(
            "七森中☆生徒会",
            "ななもりちゅうせいとかい",
            &["生徒会".to_string(), "七森中生徒会".to_string()],
            Some(TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"))),
            TagDepth::new(2, 2),
        ).await.unwrap();

        let actual_id = actual.id;
        assert_eq!(actual.name, "七森中☆生徒会".to_string());
        assert_eq!(actual.kana, "ななもりちゅうせいとかい");
        assert_eq!(actual.aliases, AliasSet::new(BTreeSet::from(["生徒会".to_string(), "七森中生徒会".to_string()])));
        assert_eq!(
            actual.parent,
            Some(Box::new(Tag {
                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                name: "ゆるゆり".to_string(),
                kana: "ゆるゆり".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
            })),
        );
        assert_eq!(actual.children, Vec::new());

        let actual = sqlx::query(r#"SELECT "id", "name", "kana", "aliases" FROM "tags" WHERE "id" = $1"#)
            .bind(*actual.id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

        assert_eq!(actual.get::<&str, &str>("name"), "七森中☆生徒会");
        assert_eq!(actual.get::<&str, &str>("kana"), "ななもりちゅうせいとかい");
        assert_eq!(actual.get::<Vec<String>, &str>("aliases"), vec!["生徒会".to_string(), "七森中生徒会".to_string()]);

        let actual: Vec<_> = sqlx::query(r#"SELECT "ancestor_id", "descendant_id", "distance" FROM "tag_paths" WHERE "descendant_id" = $1 ORDER BY "distance" DESC"#)
            .bind(*actual_id)
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 3);

        assert_eq!(actual[0].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
        assert_eq!(actual[0].get::<Uuid, &str>("descendant_id"), *actual_id);
        assert_eq!(actual[0].get::<i32, &str>("distance"), 2);

        assert_eq!(actual[1].get::<Uuid, &str>("ancestor_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
        assert_eq!(actual[1].get::<Uuid, &str>("descendant_id"), *actual_id);
        assert_eq!(actual[1].get::<i32, &str>("distance"), 1);

        assert_eq!(actual[2].get::<Uuid, &str>("ancestor_id"), *actual_id);
        assert_eq!(actual[2].get::<Uuid, &str>("descendant_id"), *actual_id);
        assert_eq!(actual[2].get::<i32, &str>("distance"), 0);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn create_without_parent_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.create(
            "七森中☆生徒会",
            "ななもりちゅうせいとかい",
            &["生徒会".to_string(), "七森中生徒会".to_string()],
            None,
            TagDepth::new(2, 2),
        ).await.unwrap();

        let actual_id = actual.id;
        assert_eq!(actual.name, "七森中☆生徒会".to_string());
        assert_eq!(actual.kana, "ななもりちゅうせいとかい");
        assert_eq!(actual.aliases, AliasSet::new(BTreeSet::from(["生徒会".to_string(), "七森中生徒会".to_string()])));
        assert_eq!(actual.parent, None);
        assert_eq!(actual.children, Vec::new());

        let actual = sqlx::query(r#"SELECT "id", "name", "kana", "aliases" FROM "tags" WHERE "id" = $1"#)
            .bind(*actual.id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

        assert_eq!(actual.get::<&str, &str>("name"), "七森中☆生徒会");
        assert_eq!(actual.get::<&str, &str>("kana"), "ななもりちゅうせいとかい");
        assert_eq!(actual.get::<Vec<String>, &str>("aliases"), vec!["生徒会".to_string(), "七森中生徒会".to_string()]);

        let actual: Vec<_> = sqlx::query(r#"SELECT "ancestor_id", "descendant_id", "distance" FROM "tag_paths" WHERE "descendant_id" = $1 ORDER BY "distance" DESC"#)
            .bind(*actual_id)
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 2);

        assert_eq!(actual[0].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
        assert_eq!(actual[0].get::<Uuid, &str>("descendant_id"), *actual_id);
        assert_eq!(actual[0].get::<i32, &str>("distance"), 1);

        assert_eq!(actual[1].get::<Uuid, &str>("ancestor_id"), *actual_id);
        assert_eq!(actual[1].get::<Uuid, &str>("descendant_id"), *actual_id);
        assert_eq!(actual[1].get::<i32, &str>("distance"), 0);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_ids_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_ids(
            [
                TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
            ],
            TagDepth::new(2, 2),
        ).await.unwrap();

        assert_eq!(actual, vec![
            Tag {
                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                name: "赤座あかり".to_string(),
                kana: "あかざあかり".to_string(),
                aliases: AliasSet::default(),
                parent: Some(Box::new(Tag {
                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                    name: "七森中☆ごらく部".to_string(),
                    kana: "ななもりちゅうごらくぶ".to_string(),
                    aliases: AliasSet::default(),
                    parent: Some(Box::new(Tag {
                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                        name: "ゆるゆり".to_string(),
                        kana: "ゆるゆり".to_string(),
                        aliases: AliasSet::default(),
                        parent: None,
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                    })),
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                })),
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                name: "魔女っ娘ミラクるん".to_string(),
                kana: "まじょっこミラクるん".to_string(),
                aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                parent: Some(Box::new(Tag {
                    id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    name: "ゆるゆり".to_string(),
                    kana: "ゆるゆり".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                })),
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                name: "ゆるゆり".to_string(),
                kana: "ゆるゆり".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: vec![
                    Tag {
                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                        name: "七森中☆ごらく部".to_string(),
                        kana: "ななもりちゅうごらくぶ".to_string(),
                        aliases: AliasSet::default(),
                        parent: None,
                        children: vec![
                            Tag {
                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                name: "赤座あかり".to_string(),
                                kana: "あかざあかり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            },
                            Tag {
                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                name: "歳納京子".to_string(),
                                kana: "としのうきょうこ".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            },
                            Tag {
                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                name: "船見結衣".to_string(),
                                kana: "ふなみゆい".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                            },
                            Tag {
                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                name: "吉川ちなつ".to_string(),
                                kana: "よしかわちなつ".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                            },
                        ],
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                    },
                    Tag {
                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                        name: "魔女っ娘ミラクるん".to_string(),
                        kana: "まじょっこミラクるん".to_string(),
                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                        parent: None,
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                    },
                ],
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_name_or_alias_like_with_name_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_name_or_alias_like("り", TagDepth::new(2, 2)).await.unwrap();

        assert_eq!(actual, vec![
            Tag {
                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                name: "赤座あかり".to_string(),
                kana: "あかざあかり".to_string(),
                aliases: AliasSet::default(),
                parent: Some(Box::new(Tag {
                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                    name: "七森中☆ごらく部".to_string(),
                    kana: "ななもりちゅうごらくぶ".to_string(),
                    aliases: AliasSet::default(),
                    parent: Some(Box::new(Tag {
                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                        name: "ゆるゆり".to_string(),
                        kana: "ゆるゆり".to_string(),
                        aliases: AliasSet::default(),
                        parent: None,
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                    })),
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                })),
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                name: "ゆるゆり".to_string(),
                kana: "ゆるゆり".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: vec![
                    Tag {
                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                        name: "七森中☆ごらく部".to_string(),
                        kana: "ななもりちゅうごらくぶ".to_string(),
                        aliases: AliasSet::default(),
                        parent: None,
                        children: vec![
                            Tag {
                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                name: "赤座あかり".to_string(),
                                kana: "あかざあかり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            },
                            Tag {
                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                name: "歳納京子".to_string(),
                                kana: "としのうきょうこ".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            },
                            Tag {
                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                name: "船見結衣".to_string(),
                                kana: "ふなみゆい".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                            },
                            Tag {
                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                name: "吉川ちなつ".to_string(),
                                kana: "よしかわちなつ".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                            },
                        ],
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                    },
                    Tag {
                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                        name: "魔女っ娘ミラクるん".to_string(),
                        kana: "まじょっこミラクるん".to_string(),
                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                        parent: None,
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                    },
                ],
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_name_or_alias_like_with_alias_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_name_or_alias_like("げ", TagDepth::new(2, 2)).await.unwrap();

        assert_eq!(actual, vec![
            Tag {
                id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                name: "鈴仙・優曇華院・イナバ".to_string(),
                kana: "れいせん・うどんげいん・いなば".to_string(),
                aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                parent: Some(Box::new(Tag {
                    id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                    name: "東方Project".to_string(),
                    kana: "とうほうProject".to_string(),
                    aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                })),
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_by_name_or_alias_like_with_name_and_alias_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_by_name_or_alias_like("ん", TagDepth::new(2, 2)).await.unwrap();

        assert_eq!(actual, vec![
            Tag {
                id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                name: "鈴仙・優曇華院・イナバ".to_string(),
                kana: "れいせん・うどんげいん・いなば".to_string(),
                aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                parent: Some(Box::new(Tag {
                    id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                    name: "東方Project".to_string(),
                    kana: "とうほうProject".to_string(),
                    aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                })),
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                name: "魔女っ娘ミラクるん".to_string(),
                kana: "まじょっこミラクるん".to_string(),
                aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                parent: Some(Box::new(Tag {
                    id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    name: "ゆるゆり".to_string(),
                    kana: "ゆるゆり".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                })),
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_depth_and_root_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            TagDepth::new(2, 2),
            true,
            None,
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Tag {
                id: TagId::from(uuid!("9b47af74-f034-4973-b284-704ab3e18b68")),
                name: "アークナイツ".to_string(),
                kana: "アークナイツ".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 13)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 10)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("74303485-40fb-466d-a93f-0cba46f6f43c")),
                name: "原神".to_string(),
                kana: "げんしん".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 12)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 13)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                name: "東方Project".to_string(),
                kana: "とうほうProject".to_string(),
                aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                parent: None,
                children: vec![
                    Tag {
                        id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                        name: "古明地こいし".to_string(),
                        kana: "こめいじこいし".to_string(),
                        aliases: AliasSet::default(),
                        parent: None,
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                    },
                    Tag {
                        id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                        name: "博麗霊夢".to_string(),
                        kana: "はくれいれいむ".to_string(),
                        aliases: AliasSet::default(),
                        parent: None,
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
                    },
                    Tag {
                        id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                        name: "フランドール・スカーレット".to_string(),
                        kana: "フランドール・スカーレット".to_string(),
                        aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                        parent: None,
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                    },
                    Tag {
                        id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                        name: "鈴仙・優曇華院・イナバ".to_string(),
                        kana: "れいせん・うどんげいん・いなば".to_string(),
                        aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                        parent: None,
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                    },
                ],
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_depth_and_root_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            TagDepth::new(2, 2),
            true,
            None,
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Tag {
                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                name: "ゆるゆり".to_string(),
                kana: "ゆるゆり".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: vec![
                    Tag {
                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                        name: "七森中☆ごらく部".to_string(),
                        kana: "ななもりちゅうごらくぶ".to_string(),
                        aliases: AliasSet::default(),
                        parent: None,
                        children: vec![
                            Tag {
                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                name: "赤座あかり".to_string(),
                                kana: "あかざあかり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            },
                            Tag {
                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                name: "歳納京子".to_string(),
                                kana: "としのうきょうこ".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            },
                            Tag {
                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                name: "船見結衣".to_string(),
                                kana: "ふなみゆい".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                            },
                            Tag {
                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                name: "吉川ちなつ".to_string(),
                                kana: "よしかわちなつ".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                            },
                        ],
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                    },
                    Tag {
                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                        name: "魔女っ娘ミラクるん".to_string(),
                        kana: "まじょっこミラクるん".to_string(),
                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                        parent: None,
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                    },
                ],
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("867fe021-a034-4b01-badb-7d56f77406b5")),
                name: "ブルーアーカイブ".to_string(),
                kana: "ブルーアーカイブ".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 9)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                name: "東方Project".to_string(),
                kana: "とうほうProject".to_string(),
                aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                parent: None,
                children: vec![
                    Tag {
                        id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                        name: "古明地こいし".to_string(),
                        kana: "こめいじこいし".to_string(),
                        aliases: AliasSet::default(),
                        parent: None,
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                    },
                    Tag {
                        id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                        name: "博麗霊夢".to_string(),
                        kana: "はくれいれいむ".to_string(),
                        aliases: AliasSet::default(),
                        parent: None,
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
                    },
                    Tag {
                        id: TagId::from(uuid!("d65c551d-5a49-4ec7-8e8b-0054e116a18d")),
                        name: "フランドール・スカーレット".to_string(),
                        kana: "フランドール・スカーレット".to_string(),
                        aliases: AliasSet::new(BTreeSet::from(["フラン".to_string()])),
                        parent: None,
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                    },
                    Tag {
                        id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                        name: "鈴仙・優曇華院・イナバ".to_string(),
                        kana: "れいせん・うどんげいん・いなば".to_string(),
                        aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                        parent: None,
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                    },
                ],
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_depth_and_root_after_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            TagDepth::new(2, 2),
            true,
            Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Tag {
                id: TagId::from(uuid!("867fe021-a034-4b01-badb-7d56f77406b5")),
                name: "ブルーアーカイブ".to_string(),
                kana: "ブルーアーカイブ".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 9)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                name: "ゆるゆり".to_string(),
                kana: "ゆるゆり".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: vec![
                    Tag {
                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                        name: "七森中☆ごらく部".to_string(),
                        kana: "ななもりちゅうごらくぶ".to_string(),
                        aliases: AliasSet::default(),
                        parent: None,
                        children: vec![
                            Tag {
                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                name: "赤座あかり".to_string(),
                                kana: "あかざあかり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            },
                            Tag {
                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                name: "歳納京子".to_string(),
                                kana: "としのうきょうこ".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            },
                            Tag {
                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                name: "船見結衣".to_string(),
                                kana: "ふなみゆい".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                            },
                            Tag {
                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                name: "吉川ちなつ".to_string(),
                                kana: "よしかわちなつ".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                            },
                        ],
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                    },
                    Tag {
                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                        name: "魔女っ娘ミラクるん".to_string(),
                        kana: "まじょっこミラクるん".to_string(),
                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                        parent: None,
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                    },
                ],
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_depth_and_root_after_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            TagDepth::new(2, 2),
            true,
            Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Tag {
                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                name: "ゆるゆり".to_string(),
                kana: "ゆるゆり".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: vec![
                    Tag {
                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                        name: "七森中☆ごらく部".to_string(),
                        kana: "ななもりちゅうごらくぶ".to_string(),
                        aliases: AliasSet::default(),
                        parent: None,
                        children: vec![
                            Tag {
                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                name: "赤座あかり".to_string(),
                                kana: "あかざあかり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            },
                            Tag {
                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                name: "歳納京子".to_string(),
                                kana: "としのうきょうこ".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            },
                            Tag {
                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                name: "船見結衣".to_string(),
                                kana: "ふなみゆい".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                            },
                            Tag {
                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                name: "吉川ちなつ".to_string(),
                                kana: "よしかわちなつ".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                            },
                        ],
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                    },
                    Tag {
                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                        name: "魔女っ娘ミラクるん".to_string(),
                        kana: "まじょっこミラクるん".to_string(),
                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                        parent: None,
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                    },
                ],
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("867fe021-a034-4b01-badb-7d56f77406b5")),
                name: "ブルーアーカイブ".to_string(),
                kana: "ブルーアーカイブ".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 9)).unwrap(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_depth_and_root_before_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            TagDepth::new(2, 2),
            true,
            None,
            Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Tag {
                id: TagId::from(uuid!("9b47af74-f034-4973-b284-704ab3e18b68")),
                name: "アークナイツ".to_string(),
                kana: "アークナイツ".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 13)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 10)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("74303485-40fb-466d-a93f-0cba46f6f43c")),
                name: "原神".to_string(),
                kana: "げんしん".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 12)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 13)).unwrap(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_depth_and_root_before_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            TagDepth::new(2, 2),
            true,
            None,
            Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Tag {
                id: TagId::from(uuid!("74303485-40fb-466d-a93f-0cba46f6f43c")),
                name: "原神".to_string(),
                kana: "げんしん".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 12)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 13)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("9b47af74-f034-4973-b284-704ab3e18b68")),
                name: "アークナイツ".to_string(),
                kana: "アークナイツ".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 13)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 10)).unwrap(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_depth_and_no_root_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            TagDepth::new(2, 2),
            false,
            None,
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Tag {
                id: TagId::from(uuid!("9b47af74-f034-4973-b284-704ab3e18b68")),
                name: "アークナイツ".to_string(),
                kana: "アークナイツ".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 13)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 10)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                name: "赤座あかり".to_string(),
                kana: "あかざあかり".to_string(),
                aliases: AliasSet::default(),
                parent: Some(Box::new(Tag {
                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                    name: "七森中☆ごらく部".to_string(),
                    kana: "ななもりちゅうごらくぶ".to_string(),
                    aliases: AliasSet::default(),
                    parent: Some(Box::new(Tag {
                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                        name: "ゆるゆり".to_string(),
                        kana: "ゆるゆり".to_string(),
                        aliases: AliasSet::default(),
                        parent: None,
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                    })),
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                })),
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("74303485-40fb-466d-a93f-0cba46f6f43c")),
                name: "原神".to_string(),
                kana: "げんしん".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 12)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 13)).unwrap(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_depth_and_no_root_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            TagDepth::new(2, 2),
            false,
            None,
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Tag {
                id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                name: "鈴仙・優曇華院・イナバ".to_string(),
                kana: "れいせん・うどんげいん・いなば".to_string(),
                aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                parent: Some(Box::new(Tag {
                    id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                    name: "東方Project".to_string(),
                    kana: "とうほうProject".to_string(),
                    aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                })),
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                name: "吉川ちなつ".to_string(),
                kana: "よしかわちなつ".to_string(),
                aliases: AliasSet::default(),
                parent: Some(Box::new(Tag {
                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                    name: "七森中☆ごらく部".to_string(),
                    kana: "ななもりちゅうごらくぶ".to_string(),
                    aliases: AliasSet::default(),
                    parent: Some(Box::new(Tag {
                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                        name: "ゆるゆり".to_string(),
                        kana: "ゆるゆり".to_string(),
                        aliases: AliasSet::default(),
                        parent: None,
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                    })),
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                })),
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                name: "ゆるゆり".to_string(),
                kana: "ゆるゆり".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: vec![
                    Tag {
                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                        name: "七森中☆ごらく部".to_string(),
                        kana: "ななもりちゅうごらくぶ".to_string(),
                        aliases: AliasSet::default(),
                        parent: None,
                        children: vec![
                            Tag {
                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                name: "赤座あかり".to_string(),
                                kana: "あかざあかり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            },
                            Tag {
                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                name: "歳納京子".to_string(),
                                kana: "としのうきょうこ".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            },
                            Tag {
                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                name: "船見結衣".to_string(),
                                kana: "ふなみゆい".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                            },
                            Tag {
                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                name: "吉川ちなつ".to_string(),
                                kana: "よしかわちなつ".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                            },
                        ],
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                    },
                    Tag {
                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                        name: "魔女っ娘ミラクるん".to_string(),
                        kana: "まじょっこミラクるん".to_string(),
                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                        parent: None,
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                    },
                ],
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_depth_and_no_root_after_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            TagDepth::new(2, 2),
            false,
            Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Tag {
                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                name: "歳納京子".to_string(),
                kana: "としのうきょうこ".to_string(),
                aliases: AliasSet::default(),
                parent: Some(Box::new(Tag {
                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                    name: "七森中☆ごらく部".to_string(),
                    kana: "ななもりちゅうごらくぶ".to_string(),
                    aliases: AliasSet::default(),
                    parent: Some(Box::new(Tag {
                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                        name: "ゆるゆり".to_string(),
                        kana: "ゆるゆり".to_string(),
                        aliases: AliasSet::default(),
                        parent: None,
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                    })),
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                })),
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                name: "七森中☆ごらく部".to_string(),
                kana: "ななもりちゅうごらくぶ".to_string(),
                aliases: AliasSet::default(),
                parent: Some(Box::new(Tag {
                    id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                    name: "ゆるゆり".to_string(),
                    kana: "ゆるゆり".to_string(),
                    aliases: AliasSet::default(),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                })),
                children: vec![
                    Tag {
                        id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                        name: "赤座あかり".to_string(),
                        kana: "あかざあかり".to_string(),
                        aliases: AliasSet::default(),
                        parent: None,
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                    },
                    Tag {
                        id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                        name: "歳納京子".to_string(),
                        kana: "としのうきょうこ".to_string(),
                        aliases: AliasSet::default(),
                        parent: None,
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                    },
                    Tag {
                        id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                        name: "船見結衣".to_string(),
                        kana: "ふなみゆい".to_string(),
                        aliases: AliasSet::default(),
                        parent: None,
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                    },
                    Tag {
                        id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                        name: "吉川ちなつ".to_string(),
                        kana: "よしかわちなつ".to_string(),
                        aliases: AliasSet::default(),
                        parent: None,
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                    },
                ],
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                name: "博麗霊夢".to_string(),
                kana: "はくれいれいむ".to_string(),
                aliases: AliasSet::default(),
                parent: Some(Box::new(Tag {
                    id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                    name: "東方Project".to_string(),
                    kana: "とうほうProject".to_string(),
                    aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                })),
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_depth_and_no_root_after_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            TagDepth::new(2, 2),
            false,
            Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Tag {
                id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                name: "鈴仙・優曇華院・イナバ".to_string(),
                kana: "れいせん・うどんげいん・いなば".to_string(),
                aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                parent: Some(Box::new(Tag {
                    id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                    name: "東方Project".to_string(),
                    kana: "とうほうProject".to_string(),
                    aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                })),
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                name: "吉川ちなつ".to_string(),
                kana: "よしかわちなつ".to_string(),
                aliases: AliasSet::default(),
                parent: Some(Box::new(Tag {
                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                    name: "七森中☆ごらく部".to_string(),
                    kana: "ななもりちゅうごらくぶ".to_string(),
                    aliases: AliasSet::default(),
                    parent: Some(Box::new(Tag {
                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                        name: "ゆるゆり".to_string(),
                        kana: "ゆるゆり".to_string(),
                        aliases: AliasSet::default(),
                        parent: None,
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                    })),
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                })),
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                name: "ゆるゆり".to_string(),
                kana: "ゆるゆり".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: vec![
                    Tag {
                        id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                        name: "七森中☆ごらく部".to_string(),
                        kana: "ななもりちゅうごらくぶ".to_string(),
                        aliases: AliasSet::default(),
                        parent: None,
                        children: vec![
                            Tag {
                                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                                name: "赤座あかり".to_string(),
                                kana: "あかざあかり".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            },
                            Tag {
                                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                                name: "歳納京子".to_string(),
                                kana: "としのうきょうこ".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                            },
                            Tag {
                                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                                name: "船見結衣".to_string(),
                                kana: "ふなみゆい".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                            },
                            Tag {
                                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                                name: "吉川ちなつ".to_string(),
                                kana: "よしかわちなつ".to_string(),
                                aliases: AliasSet::default(),
                                parent: None,
                                children: Vec::new(),
                                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
                            },
                        ],
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                    },
                    Tag {
                        id: TagId::from(uuid!("d1a302b5-7b49-44be-9019-ac337077786a")),
                        name: "魔女っ娘ミラクるん".to_string(),
                        kana: "まじょっこミラクるん".to_string(),
                        aliases: AliasSet::new(BTreeSet::from(["ミラクるん".to_string()])),
                        parent: None,
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
                    },
                ],
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_depth_and_no_root_before_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            TagDepth::new(2, 2),
            false,
            None,
            Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Tag {
                id: TagId::from(uuid!("9b47af74-f034-4973-b284-704ab3e18b68")),
                name: "アークナイツ".to_string(),
                kana: "アークナイツ".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 13)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 10)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                name: "赤座あかり".to_string(),
                kana: "あかざあかり".to_string(),
                aliases: AliasSet::default(),
                parent: Some(Box::new(Tag {
                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                    name: "七森中☆ごらく部".to_string(),
                    kana: "ななもりちゅうごらくぶ".to_string(),
                    aliases: AliasSet::default(),
                    parent: Some(Box::new(Tag {
                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                        name: "ゆるゆり".to_string(),
                        kana: "ゆるゆり".to_string(),
                        aliases: AliasSet::default(),
                        parent: None,
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                    })),
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                })),
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("74303485-40fb-466d-a93f-0cba46f6f43c")),
                name: "原神".to_string(),
                kana: "げんしん".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 12)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 13)).unwrap(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_depth_and_no_root_before_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            TagDepth::new(2, 2),
            false,
            None,
            Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Tag {
                id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                name: "古明地こいし".to_string(),
                kana: "こめいじこいし".to_string(),
                aliases: AliasSet::default(),
                parent: Some(Box::new(Tag {
                    id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                    name: "東方Project".to_string(),
                    kana: "とうほうProject".to_string(),
                    aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                    parent: None,
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
                })),
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("74303485-40fb-466d-a93f-0cba46f6f43c")),
                name: "原神".to_string(),
                kana: "げんしん".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 12)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 13)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                name: "赤座あかり".to_string(),
                kana: "あかざあかり".to_string(),
                aliases: AliasSet::default(),
                parent: Some(Box::new(Tag {
                    id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                    name: "七森中☆ごらく部".to_string(),
                    kana: "ななもりちゅうごらくぶ".to_string(),
                    aliases: AliasSet::default(),
                    parent: Some(Box::new(Tag {
                        id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                        name: "ゆるゆり".to_string(),
                        kana: "ゆるゆり".to_string(),
                        aliases: AliasSet::default(),
                        parent: None,
                        children: Vec::new(),
                        created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                        updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
                    })),
                    children: Vec::new(),
                    created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                    updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
                })),
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_no_depth_and_root_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            TagDepth::new(0, 0),
            true,
            None,
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Tag {
                id: TagId::from(uuid!("9b47af74-f034-4973-b284-704ab3e18b68")),
                name: "アークナイツ".to_string(),
                kana: "アークナイツ".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 13)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 10)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("74303485-40fb-466d-a93f-0cba46f6f43c")),
                name: "原神".to_string(),
                kana: "げんしん".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 12)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 13)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                name: "東方Project".to_string(),
                kana: "とうほうProject".to_string(),
                aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_no_depth_and_root_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            TagDepth::new(0, 0),
            true,
            None,
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Tag {
                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                name: "ゆるゆり".to_string(),
                kana: "ゆるゆり".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("867fe021-a034-4b01-badb-7d56f77406b5")),
                name: "ブルーアーカイブ".to_string(),
                kana: "ブルーアーカイブ".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 9)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
                name: "東方Project".to_string(),
                kana: "とうほうProject".to_string(),
                aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_no_depth_and_root_after_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            TagDepth::new(0, 0),
            true,
            Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Tag {
                id: TagId::from(uuid!("867fe021-a034-4b01-badb-7d56f77406b5")),
                name: "ブルーアーカイブ".to_string(),
                kana: "ブルーアーカイブ".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 9)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                name: "ゆるゆり".to_string(),
                kana: "ゆるゆり".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_no_depth_and_root_after_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            TagDepth::new(0, 0),
            true,
            Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Tag {
                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                name: "ゆるゆり".to_string(),
                kana: "ゆるゆり".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("867fe021-a034-4b01-badb-7d56f77406b5")),
                name: "ブルーアーカイブ".to_string(),
                kana: "ブルーアーカイブ".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 11)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 9)).unwrap(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_no_depth_and_root_before_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            TagDepth::new(0, 0),
            true,
            None,
            Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Tag {
                id: TagId::from(uuid!("9b47af74-f034-4973-b284-704ab3e18b68")),
                name: "アークナイツ".to_string(),
                kana: "アークナイツ".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 13)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 10)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("74303485-40fb-466d-a93f-0cba46f6f43c")),
                name: "原神".to_string(),
                kana: "げんしん".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 12)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 13)).unwrap(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_no_depth_and_root_before_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            TagDepth::new(0, 0),
            true,
            None,
            Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Tag {
                id: TagId::from(uuid!("74303485-40fb-466d-a93f-0cba46f6f43c")),
                name: "原神".to_string(),
                kana: "げんしん".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 12)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 13)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("9b47af74-f034-4973-b284-704ab3e18b68")),
                name: "アークナイツ".to_string(),
                kana: "アークナイツ".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 13)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 10)).unwrap(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_no_depth_and_no_root_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            TagDepth::new(0, 0),
            false,
            None,
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Tag {
                id: TagId::from(uuid!("9b47af74-f034-4973-b284-704ab3e18b68")),
                name: "アークナイツ".to_string(),
                kana: "アークナイツ".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 13)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 10)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                name: "赤座あかり".to_string(),
                kana: "あかざあかり".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("74303485-40fb-466d-a93f-0cba46f6f43c")),
                name: "原神".to_string(),
                kana: "げんしん".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 12)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 13)).unwrap(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_no_depth_and_no_root_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            TagDepth::new(0, 0),
            false,
            None,
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Tag {
                id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                name: "鈴仙・優曇華院・イナバ".to_string(),
                kana: "れいせん・うどんげいん・いなば".to_string(),
                aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                name: "吉川ちなつ".to_string(),
                kana: "よしかわちなつ".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                name: "ゆるゆり".to_string(),
                kana: "ゆるゆり".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_no_depth_and_no_root_after_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            TagDepth::new(0, 0),
            false,
            Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
            None,
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Tag {
                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                name: "歳納京子".to_string(),
                kana: "としのうきょうこ".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
                name: "七森中☆ごらく部".to_string(),
                kana: "ななもりちゅうごらくぶ".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("3255874e-1035-427e-80e3-19bb7b28a3fb")),
                name: "博麗霊夢".to_string(),
                kana: "はくれいれいむ".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 10)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 11)).unwrap(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_no_depth_and_no_root_after_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            TagDepth::new(0, 0),
            false,
            Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
            None,
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Tag {
                id: TagId::from(uuid!("1157d6d9-54c5-48df-9f6c-3eba9fe38dfc")),
                name: "鈴仙・優曇華院・イナバ".to_string(),
                kana: "れいせん・うどんげいん・いなば".to_string(),
                aliases: AliasSet::new(BTreeSet::from(["うどんげ".to_string()])),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                name: "吉川ちなつ".to_string(),
                kana: "よしかわちなつ".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
                name: "ゆるゆり".to_string(),
                kana: "ゆるゆり".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_no_depth_and_no_root_before_asc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            TagDepth::new(0, 0),
            false,
            None,
            Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
            OrderDirection::Ascending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Tag {
                id: TagId::from(uuid!("9b47af74-f034-4973-b284-704ab3e18b68")),
                name: "アークナイツ".to_string(),
                kana: "アークナイツ".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 13)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 10)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                name: "赤座あかり".to_string(),
                kana: "あかざあかり".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("74303485-40fb-466d-a93f-0cba46f6f43c")),
                name: "原神".to_string(),
                kana: "げんしん".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 12)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 13)).unwrap(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_no_depth_and_no_root_before_desc_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            TagDepth::new(0, 0),
            false,
            None,
            Some(("とうほうProject".to_string(), TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")))),
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert_eq!(actual, vec![
            Tag {
                id: TagId::from(uuid!("7648d9b5-e0f0-48c2-870c-1fcd60a099de")),
                name: "古明地こいし".to_string(),
                kana: "こめいじこいし".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("74303485-40fb-466d-a93f-0cba46f6f43c")),
                name: "原神".to_string(),
                kana: "げんしん".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 12)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 3, 4).and_then(|d| d.and_hms_opt(5, 6, 13)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                name: "赤座あかり".to_string(),
                kana: "あかざあかり".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
            },
        ]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn fetch_all_with_out_of_bounds_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.fetch_all(
            TagDepth::new(2, 2),
            true,
            None,
            Some(("".to_string(), TagId::from(uuid!("00000000-0000-0000-0000-000000000000")))),
            OrderDirection::Descending,
            3,
        ).await.unwrap();

        assert!(actual.is_empty());
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn update_by_id_with_depth_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.update_by_id(
            TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
            Some("ごらく部".to_string()),
            Some("ごらくぶ".to_string()),
            ["七森中☆ごらく部".to_string()],
            [],
            TagDepth::new(2, 2),
        ).await.unwrap();

        assert_eq!(actual.name, "ごらく部".to_string());
        assert_eq!(actual.kana, "ごらくぶ".to_string());
        assert_eq!(actual.aliases, AliasSet::new(BTreeSet::from(["七森中☆ごらく部".to_string()])));
        assert_eq!(actual.parent, Some(Box::new(Tag {
            id: TagId::from(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba")),
            name: "ゆるゆり".to_string(),
            kana: "ゆるゆり".to_string(),
            aliases: AliasSet::default(),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 5)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 8)).unwrap(),
        })));
        assert_eq!(actual.children, vec![
            Tag {
                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                name: "赤座あかり".to_string(),
                kana: "あかざあかり".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                name: "歳納京子".to_string(),
                kana: "としのうきょうこ".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                name: "船見結衣".to_string(),
                kana: "ふなみゆい".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                name: "吉川ちなつ".to_string(),
                kana: "よしかわちなつ".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
            },
        ]);
        assert_eq!(actual.created_at, NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap());
        assert_ne!(actual.updated_at, NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap());

        let actual = sqlx::query(r#"SELECT "id", "name", "kana", "aliases" FROM "tags" WHERE "id" = $1"#)
            .bind(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

        assert_eq!(actual.get::<&str, &str>("name"), "ごらく部");
        assert_eq!(actual.get::<&str, &str>("kana"), "ごらくぶ");
        assert_eq!(actual.get::<Vec<String>, &str>("aliases"), vec!["七森中☆ごらく部".to_string()]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn update_by_id_without_depth_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.update_by_id(
            TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
            Some("ごらく部".to_string()),
            Some("ごらくぶ".to_string()),
            ["七森中☆ごらく部".to_string()],
            [],
            TagDepth::new(0, 0),
        ).await.unwrap();

        assert_eq!(actual.name, "ごらく部".to_string());
        assert_eq!(actual.kana, "ごらくぶ".to_string());
        assert_eq!(actual.aliases, AliasSet::new(BTreeSet::from(["七森中☆ごらく部".to_string()])));
        assert_eq!(actual.parent, None);
        assert_eq!(actual.children, Vec::new());
        assert_eq!(actual.created_at, NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap());
        assert_ne!(actual.updated_at, NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap());

        let actual = sqlx::query(r#"SELECT "id", "name", "kana", "aliases" FROM "tags" WHERE "id" = $1"#)
            .bind(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap();

        assert_eq!(actual.get::<&str, &str>("name"), "ごらく部");
        assert_eq!(actual.get::<&str, &str>("kana"), "ごらくぶ");
        assert_eq!(actual.get::<Vec<String>, &str>("aliases"), vec!["七森中☆ごらく部".to_string()]);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn update_by_id_fails(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.update_by_id(
            TagId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            None,
            None,
            [],
            [],
            TagDepth::new(0, 0),
        ).await;

        assert!(actual.is_err());
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn update_by_id_root_fails(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.update_by_id(
            TagId::from(uuid!("00000000-0000-0000-0000-000000000000")),
            None,
            None,
            [],
            [],
            TagDepth::new(0, 0),
        ).await;

        assert!(actual.is_err());
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn attach_by_id_with_depth_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.attach_by_id(
            TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
            TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
            TagDepth::new(2, 2),
        ).await.unwrap();

        assert_eq!(actual.name, "七森中☆ごらく部".to_string());
        assert_eq!(actual.kana, "ななもりちゅうごらくぶ".to_string());
        assert_eq!(actual.aliases, AliasSet::default());
        assert_eq!(actual.parent, Some(Box::new(Tag {
            id: TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
            name: "東方Project".to_string(),
            kana: "とうほうProject".to_string(),
            aliases: AliasSet::new(BTreeSet::from(["東方".to_string()])),
            parent: None,
            children: Vec::new(),
            created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 10)).unwrap(),
        })));
        assert_eq!(actual.children, vec![
            Tag {
                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                name: "赤座あかり".to_string(),
                kana: "あかざあかり".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                name: "歳納京子".to_string(),
                kana: "としのうきょうこ".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                name: "船見結衣".to_string(),
                kana: "ふなみゆい".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                name: "吉川ちなつ".to_string(),
                kana: "よしかわちなつ".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
            },
        ]);
        assert_eq!(actual.created_at, NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap());
        assert_eq!(actual.updated_at, NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap());

        let actual: Vec<_> = sqlx::query(r#"SELECT "ancestor_id", "descendant_id", "distance" FROM "tag_paths" WHERE "descendant_id" IN ($1, $2, $3, $4, $5, $6, $7) ORDER BY "descendant_id", "distance" DESC"#)
            .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
            .bind(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"))
            .bind(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"))
            .bind(uuid!("991a287c-e77d-456f-94b4-293334674d0e"))
            .bind(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"))
            .bind(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"))
            .bind(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"))
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 23);

        assert_eq!(actual[0].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
        assert_eq!(actual[0].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
        assert_eq!(actual[0].get::<i32, &str>("distance"), 3);

        assert_eq!(actual[1].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
        assert_eq!(actual[1].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
        assert_eq!(actual[1].get::<i32, &str>("distance"), 2);

        assert_eq!(actual[2].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[2].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
        assert_eq!(actual[2].get::<i32, &str>("distance"), 1);

        assert_eq!(actual[3].get::<Uuid, &str>("ancestor_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
        assert_eq!(actual[3].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
        assert_eq!(actual[3].get::<i32, &str>("distance"), 0);

        assert_eq!(actual[4].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
        assert_eq!(actual[4].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
        assert_eq!(actual[4].get::<i32, &str>("distance"), 3);

        assert_eq!(actual[5].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
        assert_eq!(actual[5].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
        assert_eq!(actual[5].get::<i32, &str>("distance"), 2);

        assert_eq!(actual[6].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[6].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
        assert_eq!(actual[6].get::<i32, &str>("distance"), 1);

        assert_eq!(actual[7].get::<Uuid, &str>("ancestor_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
        assert_eq!(actual[7].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
        assert_eq!(actual[7].get::<i32, &str>("distance"), 0);

        assert_eq!(actual[8].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
        assert_eq!(actual[8].get::<Uuid, &str>("descendant_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
        assert_eq!(actual[8].get::<i32, &str>("distance"), 1);

        assert_eq!(actual[9].get::<Uuid, &str>("ancestor_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
        assert_eq!(actual[9].get::<Uuid, &str>("descendant_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
        assert_eq!(actual[9].get::<i32, &str>("distance"), 0);

        assert_eq!(actual[10].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
        assert_eq!(actual[10].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
        assert_eq!(actual[10].get::<i32, &str>("distance"), 3);

        assert_eq!(actual[11].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
        assert_eq!(actual[11].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
        assert_eq!(actual[11].get::<i32, &str>("distance"), 2);

        assert_eq!(actual[12].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[12].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
        assert_eq!(actual[12].get::<i32, &str>("distance"), 1);

        assert_eq!(actual[13].get::<Uuid, &str>("ancestor_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
        assert_eq!(actual[13].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
        assert_eq!(actual[13].get::<i32, &str>("distance"), 0);

        assert_eq!(actual[14].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
        assert_eq!(actual[14].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
        assert_eq!(actual[14].get::<i32, &str>("distance"), 3);

        assert_eq!(actual[15].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
        assert_eq!(actual[15].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
        assert_eq!(actual[15].get::<i32, &str>("distance"), 2);

        assert_eq!(actual[16].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[16].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
        assert_eq!(actual[16].get::<i32, &str>("distance"), 1);

        assert_eq!(actual[17].get::<Uuid, &str>("ancestor_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
        assert_eq!(actual[17].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
        assert_eq!(actual[17].get::<i32, &str>("distance"), 0);

        assert_eq!(actual[18].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
        assert_eq!(actual[18].get::<Uuid, &str>("descendant_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[18].get::<i32, &str>("distance"), 2);

        assert_eq!(actual[19].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
        assert_eq!(actual[19].get::<Uuid, &str>("descendant_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[19].get::<i32, &str>("distance"), 1);

        assert_eq!(actual[20].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[20].get::<Uuid, &str>("descendant_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[20].get::<i32, &str>("distance"), 0);

        assert_eq!(actual[21].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
        assert_eq!(actual[21].get::<Uuid, &str>("descendant_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
        assert_eq!(actual[21].get::<i32, &str>("distance"), 1);

        assert_eq!(actual[22].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
        assert_eq!(actual[22].get::<Uuid, &str>("descendant_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
        assert_eq!(actual[22].get::<i32, &str>("distance"), 0);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn attach_by_id_without_depth_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.attach_by_id(
            TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
            TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
            TagDepth::new(0, 0),
        ).await.unwrap();

        assert_eq!(actual.name, "七森中☆ごらく部".to_string());
        assert_eq!(actual.kana, "ななもりちゅうごらくぶ".to_string());
        assert_eq!(actual.aliases, AliasSet::default());
        assert_eq!(actual.parent, None);
        assert_eq!(actual.children, Vec::new());
        assert_eq!(actual.created_at, NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap());
        assert_eq!(actual.updated_at, NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap());

        let actual: Vec<_> = sqlx::query(r#"SELECT "ancestor_id", "descendant_id", "distance" FROM "tag_paths" WHERE "descendant_id" IN ($1, $2, $3, $4, $5, $6, $7) ORDER BY "descendant_id", "distance" DESC"#)
            .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
            .bind(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"))
            .bind(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"))
            .bind(uuid!("991a287c-e77d-456f-94b4-293334674d0e"))
            .bind(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"))
            .bind(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"))
            .bind(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"))
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 23);

        assert_eq!(actual[0].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
        assert_eq!(actual[0].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
        assert_eq!(actual[0].get::<i32, &str>("distance"), 3);

        assert_eq!(actual[1].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
        assert_eq!(actual[1].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
        assert_eq!(actual[1].get::<i32, &str>("distance"), 2);

        assert_eq!(actual[2].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[2].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
        assert_eq!(actual[2].get::<i32, &str>("distance"), 1);

        assert_eq!(actual[3].get::<Uuid, &str>("ancestor_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
        assert_eq!(actual[3].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
        assert_eq!(actual[3].get::<i32, &str>("distance"), 0);

        assert_eq!(actual[4].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
        assert_eq!(actual[4].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
        assert_eq!(actual[4].get::<i32, &str>("distance"), 3);

        assert_eq!(actual[5].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
        assert_eq!(actual[5].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
        assert_eq!(actual[5].get::<i32, &str>("distance"), 2);

        assert_eq!(actual[6].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[6].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
        assert_eq!(actual[6].get::<i32, &str>("distance"), 1);

        assert_eq!(actual[7].get::<Uuid, &str>("ancestor_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
        assert_eq!(actual[7].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
        assert_eq!(actual[7].get::<i32, &str>("distance"), 0);

        assert_eq!(actual[8].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
        assert_eq!(actual[8].get::<Uuid, &str>("descendant_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
        assert_eq!(actual[8].get::<i32, &str>("distance"), 1);

        assert_eq!(actual[9].get::<Uuid, &str>("ancestor_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
        assert_eq!(actual[9].get::<Uuid, &str>("descendant_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
        assert_eq!(actual[9].get::<i32, &str>("distance"), 0);

        assert_eq!(actual[10].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
        assert_eq!(actual[10].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
        assert_eq!(actual[10].get::<i32, &str>("distance"), 3);

        assert_eq!(actual[11].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
        assert_eq!(actual[11].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
        assert_eq!(actual[11].get::<i32, &str>("distance"), 2);

        assert_eq!(actual[12].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[12].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
        assert_eq!(actual[12].get::<i32, &str>("distance"), 1);

        assert_eq!(actual[13].get::<Uuid, &str>("ancestor_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
        assert_eq!(actual[13].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
        assert_eq!(actual[13].get::<i32, &str>("distance"), 0);

        assert_eq!(actual[14].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
        assert_eq!(actual[14].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
        assert_eq!(actual[14].get::<i32, &str>("distance"), 3);

        assert_eq!(actual[15].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
        assert_eq!(actual[15].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
        assert_eq!(actual[15].get::<i32, &str>("distance"), 2);

        assert_eq!(actual[16].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[16].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
        assert_eq!(actual[16].get::<i32, &str>("distance"), 1);

        assert_eq!(actual[17].get::<Uuid, &str>("ancestor_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
        assert_eq!(actual[17].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
        assert_eq!(actual[17].get::<i32, &str>("distance"), 0);

        assert_eq!(actual[18].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
        assert_eq!(actual[18].get::<Uuid, &str>("descendant_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[18].get::<i32, &str>("distance"), 2);

        assert_eq!(actual[19].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
        assert_eq!(actual[19].get::<Uuid, &str>("descendant_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[19].get::<i32, &str>("distance"), 1);

        assert_eq!(actual[20].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[20].get::<Uuid, &str>("descendant_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[20].get::<i32, &str>("distance"), 0);

        assert_eq!(actual[21].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
        assert_eq!(actual[21].get::<Uuid, &str>("descendant_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
        assert_eq!(actual[21].get::<i32, &str>("distance"), 1);

        assert_eq!(actual[22].get::<Uuid, &str>("ancestor_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
        assert_eq!(actual[22].get::<Uuid, &str>("descendant_id"), uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb"));
        assert_eq!(actual[22].get::<i32, &str>("distance"), 0);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn attach_by_id_root_fails(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.attach_by_id(
            TagId::from(uuid!("00000000-0000-0000-0000-000000000000")),
            TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
            TagDepth::new(0, 0),
        ).await;

        assert!(actual.is_err());
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn attach_by_id_non_existing_fails(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.attach_by_id(
            TagId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            TagId::from(uuid!("fe81a56d-165b-446d-aebb-ca59e5acf3cb")),
            TagDepth::new(0, 0),
        ).await;

        assert!(actual.is_err());
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn detach_by_id_with_depth_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.detach_by_id(
            TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
            TagDepth::new(2, 2),
        ).await.unwrap();

        assert_eq!(actual.name, "七森中☆ごらく部".to_string());
        assert_eq!(actual.kana, "ななもりちゅうごらくぶ".to_string());
        assert_eq!(actual.aliases, AliasSet::default());
        assert_eq!(actual.parent, None);
        assert_eq!(actual.children, vec![
            Tag {
                id: TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")),
                name: "赤座あかり".to_string(),
                kana: "あかざあかり".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 9)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6")),
                name: "歳納京子".to_string(),
                kana: "としのうきょうこ".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707")),
                name: "船見結衣".to_string(),
                kana: "ふなみゆい".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 7)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 7)).unwrap(),
            },
            Tag {
                id: TagId::from(uuid!("991a287c-e77d-456f-94b4-293334674d0e")),
                name: "吉川ちなつ".to_string(),
                kana: "よしかわちなつ".to_string(),
                aliases: AliasSet::default(),
                parent: None,
                children: Vec::new(),
                created_at: NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 8)).unwrap(),
                updated_at: NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 6)).unwrap(),
            },
        ]);
        assert_eq!(actual.created_at, NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap());
        assert_eq!(actual.updated_at, NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap());

        let actual: Vec<_> = sqlx::query(r#"SELECT "ancestor_id", "descendant_id", "distance" FROM "tag_paths" WHERE "descendant_id" IN ($1, $2, $3, $4, $5, $6) ORDER BY "descendant_id", "distance" DESC"#)
            .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
            .bind(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"))
            .bind(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"))
            .bind(uuid!("991a287c-e77d-456f-94b4-293334674d0e"))
            .bind(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"))
            .bind(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"))
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 16);

        assert_eq!(actual[0].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
        assert_eq!(actual[0].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
        assert_eq!(actual[0].get::<i32, &str>("distance"), 2);

        assert_eq!(actual[1].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[1].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
        assert_eq!(actual[1].get::<i32, &str>("distance"), 1);

        assert_eq!(actual[2].get::<Uuid, &str>("ancestor_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
        assert_eq!(actual[2].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
        assert_eq!(actual[2].get::<i32, &str>("distance"), 0);

        assert_eq!(actual[3].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
        assert_eq!(actual[3].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
        assert_eq!(actual[3].get::<i32, &str>("distance"), 2);

        assert_eq!(actual[4].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[4].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
        assert_eq!(actual[4].get::<i32, &str>("distance"), 1);

        assert_eq!(actual[5].get::<Uuid, &str>("ancestor_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
        assert_eq!(actual[5].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
        assert_eq!(actual[5].get::<i32, &str>("distance"), 0);

        assert_eq!(actual[6].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
        assert_eq!(actual[6].get::<Uuid, &str>("descendant_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
        assert_eq!(actual[6].get::<i32, &str>("distance"), 1);

        assert_eq!(actual[7].get::<Uuid, &str>("ancestor_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
        assert_eq!(actual[7].get::<Uuid, &str>("descendant_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
        assert_eq!(actual[7].get::<i32, &str>("distance"), 0);

        assert_eq!(actual[8].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
        assert_eq!(actual[8].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
        assert_eq!(actual[8].get::<i32, &str>("distance"), 2);

        assert_eq!(actual[9].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[9].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
        assert_eq!(actual[9].get::<i32, &str>("distance"), 1);

        assert_eq!(actual[10].get::<Uuid, &str>("ancestor_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
        assert_eq!(actual[10].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
        assert_eq!(actual[10].get::<i32, &str>("distance"), 0);

        assert_eq!(actual[11].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
        assert_eq!(actual[11].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
        assert_eq!(actual[11].get::<i32, &str>("distance"), 2);

        assert_eq!(actual[12].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[12].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
        assert_eq!(actual[12].get::<i32, &str>("distance"), 1);

        assert_eq!(actual[13].get::<Uuid, &str>("ancestor_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
        assert_eq!(actual[13].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
        assert_eq!(actual[13].get::<i32, &str>("distance"), 0);

        assert_eq!(actual[14].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
        assert_eq!(actual[14].get::<Uuid, &str>("descendant_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[14].get::<i32, &str>("distance"), 1);

        assert_eq!(actual[15].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[15].get::<Uuid, &str>("descendant_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[15].get::<i32, &str>("distance"), 0);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn detach_by_id_without_depth_succeeds(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.detach_by_id(
            TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")),
            TagDepth::new(0, 0),
        ).await.unwrap();

        assert_eq!(actual.name, "七森中☆ごらく部".to_string());
        assert_eq!(actual.kana, "ななもりちゅうごらくぶ".to_string());
        assert_eq!(actual.aliases, AliasSet::default());
        assert_eq!(actual.parent, None);
        assert_eq!(actual.children, Vec::new());
        assert_eq!(actual.created_at, NaiveDate::from_ymd_opt(2022, 1, 2).and_then(|d| d.and_hms_opt(3, 4, 6)).unwrap());
        assert_eq!(actual.updated_at, NaiveDate::from_ymd_opt(2022, 2, 3).and_then(|d| d.and_hms_opt(4, 5, 9)).unwrap());

        let actual: Vec<_> = sqlx::query(r#"SELECT "ancestor_id", "descendant_id", "distance" FROM "tag_paths" WHERE "descendant_id" IN ($1, $2, $3, $4, $5, $6) ORDER BY "descendant_id", "distance" DESC"#)
            .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
            .bind(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"))
            .bind(uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"))
            .bind(uuid!("991a287c-e77d-456f-94b4-293334674d0e"))
            .bind(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"))
            .bind(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"))
            .fetch(&ctx.pool)
            .try_collect()
            .await
            .unwrap();

        assert_eq!(actual.len(), 16);

        assert_eq!(actual[0].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
        assert_eq!(actual[0].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
        assert_eq!(actual[0].get::<i32, &str>("distance"), 2);

        assert_eq!(actual[1].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[1].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
        assert_eq!(actual[1].get::<i32, &str>("distance"), 1);

        assert_eq!(actual[2].get::<Uuid, &str>("ancestor_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
        assert_eq!(actual[2].get::<Uuid, &str>("descendant_id"), uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"));
        assert_eq!(actual[2].get::<i32, &str>("distance"), 0);

        assert_eq!(actual[3].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
        assert_eq!(actual[3].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
        assert_eq!(actual[3].get::<i32, &str>("distance"), 2);

        assert_eq!(actual[4].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[4].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
        assert_eq!(actual[4].get::<i32, &str>("distance"), 1);

        assert_eq!(actual[5].get::<Uuid, &str>("ancestor_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
        assert_eq!(actual[5].get::<Uuid, &str>("descendant_id"), uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"));
        assert_eq!(actual[5].get::<i32, &str>("distance"), 0);

        assert_eq!(actual[6].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
        assert_eq!(actual[6].get::<Uuid, &str>("descendant_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
        assert_eq!(actual[6].get::<i32, &str>("distance"), 1);

        assert_eq!(actual[7].get::<Uuid, &str>("ancestor_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
        assert_eq!(actual[7].get::<Uuid, &str>("descendant_id"), uuid!("744b7274-371b-4790-8f5a-df4d76e983ba"));
        assert_eq!(actual[7].get::<i32, &str>("distance"), 0);

        assert_eq!(actual[8].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
        assert_eq!(actual[8].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
        assert_eq!(actual[8].get::<i32, &str>("distance"), 2);

        assert_eq!(actual[9].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[9].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
        assert_eq!(actual[9].get::<i32, &str>("distance"), 1);

        assert_eq!(actual[10].get::<Uuid, &str>("ancestor_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
        assert_eq!(actual[10].get::<Uuid, &str>("descendant_id"), uuid!("991a287c-e77d-456f-94b4-293334674d0e"));
        assert_eq!(actual[10].get::<i32, &str>("distance"), 0);

        assert_eq!(actual[11].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
        assert_eq!(actual[11].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
        assert_eq!(actual[11].get::<i32, &str>("distance"), 2);

        assert_eq!(actual[12].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[12].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
        assert_eq!(actual[12].get::<i32, &str>("distance"), 1);

        assert_eq!(actual[13].get::<Uuid, &str>("ancestor_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
        assert_eq!(actual[13].get::<Uuid, &str>("descendant_id"), uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"));
        assert_eq!(actual[13].get::<i32, &str>("distance"), 0);

        assert_eq!(actual[14].get::<Uuid, &str>("ancestor_id"), uuid!("00000000-0000-0000-0000-000000000000"));
        assert_eq!(actual[14].get::<Uuid, &str>("descendant_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[14].get::<i32, &str>("distance"), 1);

        assert_eq!(actual[15].get::<Uuid, &str>("ancestor_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[15].get::<Uuid, &str>("descendant_id"), uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"));
        assert_eq!(actual[15].get::<i32, &str>("distance"), 0);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn detach_by_id_root_fails(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.detach_by_id(
            TagId::from(uuid!("00000000-0000-0000-0000-000000000000")),
            TagDepth::new(0, 0),
        ).await;

        assert!(actual.is_err());
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn detach_by_id_non_existing_fails(ctx: &DatabaseContext) {
        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.detach_by_id(
            TagId::from(uuid!("11111111-1111-1111-1111-111111111111")),
            TagDepth::new(0, 0),
        ).await;

        assert!(actual.is_err());
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn delete_by_id_root_with_recursive_fails(ctx: &DatabaseContext) {
        let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" = $1"#)
            .bind(uuid!("00000000-0000-0000-0000-000000000000"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap()
            .get(0);

        assert_eq!(actual, 1);

        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.delete_by_id(TagId::from(uuid!("00000000-0000-0000-0000-000000000000")), true).await;

        assert!(actual.is_err());

        let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" = $1"#)
            .bind(uuid!("00000000-0000-0000-0000-000000000000"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap()
            .get(0);

        assert_eq!(actual, 1);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn delete_by_id_root_without_recursive_fails(ctx: &DatabaseContext) {
        let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" = $1"#)
            .bind(uuid!("00000000-0000-0000-0000-000000000000"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap()
            .get(0);

        assert_eq!(actual, 1);

        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.delete_by_id(TagId::from(uuid!("00000000-0000-0000-0000-000000000000")), false).await;

        assert!(actual.is_err());

        let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" = $1"#)
            .bind(uuid!("00000000-0000-0000-0000-000000000000"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap()
            .get(0);

        assert_eq!(actual, 1);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn delete_by_id_node_with_recursive_succeeds(ctx: &DatabaseContext) {
        let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" IN ($1, $2, $3, $4, $5)"#)
            .bind(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"))
            .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
            .bind(uuid!("991a287c-e77d-456f-94b4-293334674d0e"))
            .bind(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"))
            .bind(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap()
            .get(0);

        assert_eq!(actual, 5);

        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.delete_by_id(TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")), true).await.unwrap();

        assert_eq!(actual, DeleteResult::Deleted(5));

        let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" IN ($1, $2, $3, $4, $5)"#)
            .bind(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"))
            .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
            .bind(uuid!("991a287c-e77d-456f-94b4-293334674d0e"))
            .bind(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"))
            .bind(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap()
            .get(0);

        assert_eq!(actual, 0);

        let actual = repository.delete_by_id(TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")), true).await.unwrap();

        assert_eq!(actual, DeleteResult::NotFound);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn delete_by_id_node_without_recursive_fails(ctx: &DatabaseContext) {
        let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" IN ($1, $2, $3, $4, $5)"#)
            .bind(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"))
            .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
            .bind(uuid!("991a287c-e77d-456f-94b4-293334674d0e"))
            .bind(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"))
            .bind(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap()
            .get(0);

        assert_eq!(actual, 5);

        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository
            .delete_by_id(TagId::from(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60")), false)
            .await
            .unwrap_err()
            .downcast::<PostgresTagError>()
            .unwrap();

        assert_eq!(actual, PostgresTagError::ChildrenExist(4));

        let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" IN ($1, $2, $3, $4, $5)"#)
            .bind(uuid!("e8d32062-0185-43e8-a27d-6ca707d7dd60"))
            .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
            .bind(uuid!("991a287c-e77d-456f-94b4-293334674d0e"))
            .bind(uuid!("69c4860d-24d0-41f6-a3ab-ac07dea5abd6"))
            .bind(uuid!("a2a6c29d-18d0-47b1-a324-88e93c267707"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap()
            .get(0);

        assert_eq!(actual, 5);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn delete_by_id_leaf_with_recursive_succeeds(ctx: &DatabaseContext) {
        let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" = $1"#)
            .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap()
            .get(0);

        assert_eq!(actual, 1);

        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.delete_by_id(TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")), true).await.unwrap();

        assert_eq!(actual, DeleteResult::Deleted(1));

        let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" = $1"#)
            .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap()
            .get(0);

        assert_eq!(actual, 0);

        let actual = repository.delete_by_id(TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")), true).await.unwrap();

        assert_eq!(actual, DeleteResult::NotFound);
    }

    #[test_context(DatabaseContext)]
    #[tokio::test]
    #[cfg_attr(not(feature = "test-postgres"), ignore)]
    async fn delete_by_id_leaf_without_recursive_succeeds(ctx: &DatabaseContext) {
        let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" = $1"#)
            .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap()
            .get(0);

        assert_eq!(actual, 1);

        let repository = PostgresTagsRepository::new(ctx.pool.clone());
        let actual = repository.delete_by_id(TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")), false).await.unwrap();

        assert_eq!(actual, DeleteResult::Deleted(1));

        let actual: i64 = sqlx::query(r#"SELECT COUNT(*) FROM "tags" WHERE "id" = $1"#)
            .bind(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5"))
            .fetch_one(&ctx.pool)
            .await
            .unwrap()
            .get(0);

        assert_eq!(actual, 0);

        let actual = repository.delete_by_id(TagId::from(uuid!("12c4101e-722f-4172-9fe2-7862ebbc8fc5")), false).await.unwrap();

        assert_eq!(actual, DeleteResult::NotFound);
    }
}
