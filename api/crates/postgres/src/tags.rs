use std::{
    cell::RefCell,
    collections::HashSet,
    rc::{Rc, Weak},
};

use chrono::{DateTime, Utc};
use cow_utils::CowUtils;
use derive_more::{Constructor, From, Into};
use domain::{
    entity::tags::{Tag, TagDepth, TagId},
    error::{Error, ErrorKind, Result},
    repository::{self, tags::TagsRepository, DeleteResult},
};
use futures::TryStreamExt;
use ordermap::{OrderMap, OrderSet};
use sea_query::{extension::postgres::PgExpr, Asterisk, BinOper, Cond, Expr, Iden, JoinType, LikeExpr, LockType, Order, PostgresQueryBuilder, Query, SelectStatement};
use sea_query_binder::SqlxBinder;
use sqlx::{Acquire, FromRow, PgPool, Postgres, Transaction, PgConnection, Row};

use crate::{
    expr::array::ArrayExpr,
    sea_query_uuid_value,
};

#[derive(Clone, Constructor)]
pub struct PostgresTagsRepository {
    pool: PgPool,
}

#[derive(Clone, Debug, From, Into)]
pub(crate) struct PostgresTagId(TagId);

#[derive(Debug, FromRow)]
struct PostgresTagRow {
    id: PostgresTagId,
    name: String,
    kana: String,
    aliases: Vec<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct PostgresTagDescendantRow {
    descendant_id: PostgresTagId,
    distance: i32,
}

#[derive(Debug, FromRow)]
struct PostgresTagRelativeRow {
    distance: i32,

    ancestor_id: PostgresTagId,
    ancestor_name: String,
    ancestor_kana: String,
    ancestor_aliases: Vec<String>,
    ancestor_created_at: DateTime<Utc>,
    ancestor_updated_at: DateTime<Utc>,

    descendant_id: PostgresTagId,
    descendant_name: String,
    descendant_kana: String,
    descendant_aliases: Vec<String>,
    descendant_created_at: DateTime<Utc>,
    descendant_updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Default)]
struct TagRelation {
    id: TagId,
    name: String,
    kana: String,
    aliases: Vec<String>,
    parent: Weak<RefCell<Self>>,
    children: Vec<Rc<RefCell<Self>>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Iden)]
pub(crate) enum PostgresTag {
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
pub(crate) enum PostgresTagPath {
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

sea_query_uuid_value!(PostgresTagId, TagId);

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
                id: row.ancestor_id.into(),
                name: row.ancestor_name,
                kana: row.ancestor_kana,
                aliases: row.ancestor_aliases,
                created_at: row.ancestor_created_at,
                updated_at: row.ancestor_updated_at,
                ..Default::default()
            },
            TagRelation {
                id: row.descendant_id.into(),
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

impl FromIterator<PostgresTagRelativeRow> for OrderMap<TagId, Rc<RefCell<TagRelation>>> {
    fn from_iter<T>(rows: T) -> Self
    where
        T: IntoIterator<Item = PostgresTagRelativeRow>,
    {
        let mut tags = OrderMap::new();

        for tag in rows {
            let (distance, ancestor, descendant) = tag.into();
            match distance {
                0 => {
                    let ancestor_id = ancestor.id;
                    let ancestor = Rc::new(RefCell::new(ancestor));
                    tags.insert(ancestor_id, ancestor);
                },
                _ => {
                    {
                        let ancestor = tags.get(&ancestor.id).and_then(|r| r.try_borrow_mut().ok());
                        let descendant = tags.get(&descendant.id);

                        if let (Some(mut ancestor), Some(descendant)) = (ancestor, descendant) {
                            ancestor.children.push(descendant.clone());
                        }
                    }
                    {
                        let ancestor = tags.get(&ancestor.id);
                        let descendant = tags.get(&descendant.id).and_then(|r| r.try_borrow_mut().ok());

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
    let relation = Rc::unwrap_or_clone(rc).into_inner();

    let parent = depth
        .has_parent()
        .then_some(relation.parent)
        .and_then(|parent| parent.upgrade())
        .map(|relation| extract(relation, TagDepth::new(depth.parent() - 1, 0)))
        .map(Box::new);

    let children = depth
        .has_children()
        .then_some(relation.children)
        .into_iter()
        .flatten()
        .map(|relation| extract(relation, TagDepth::new(0, depth.children() - 1)))
        .collect();

    Tag {
        id: relation.id,
        name: relation.name,
        kana: relation.kana,
        aliases: relation.aliases.into(),
        parent,
        children,
        created_at: relation.created_at,
        updated_at: relation.updated_at,
    }
}

pub async fn fetch_tag_relatives<T>(conn: &mut PgConnection, ids: T, depth: TagDepth, root: bool) -> Result<Vec<Tag>>
where
    T: IntoIterator<Item = TagId>,
{
    let ids: HashSet<_> = ids.into_iter().collect();

    const TAG_ANCESTORS: &str = "tag_ancestors";
    const TAG_DESCENDANTS: &str = "tag_descendants";

    let relatives = {
        let conditions = Cond::any()
            .add_option(depth.has_parent().then(|| {
                Expr::col(PostgresTagPath::DescendantId).in_subquery(
                    Query::select()
                        .from(PostgresTagPath::Table)
                        .column(PostgresTagPath::AncestorId)
                        .and_where(Expr::col(PostgresTagPath::Distance).lte(depth.parent()))
                        .and_where(Expr::col(PostgresTagPath::DescendantId).is_in(ids.iter().cloned().map(PostgresTagId::from)))
                        .take()
                    )
            }))
            .add_option(depth.has_children().then(|| {
                Expr::col(PostgresTagPath::AncestorId).in_subquery(
                    Query::select()
                        .from(PostgresTagPath::Table)
                        .column(PostgresTagPath::DescendantId)
                        .and_where(Expr::col(PostgresTagPath::Distance).lte(depth.children()))
                        .and_where(Expr::col(PostgresTagPath::AncestorId).is_in(ids.iter().cloned().map(PostgresTagId::from)))
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
        .expr_as(Expr::col((TAG_ANCESTORS, PostgresTag::Id)), PostgresTagRelation::AncestorId)
        .expr_as(Expr::col((TAG_ANCESTORS, PostgresTag::Name)), PostgresTagRelation::AncestorName)
        .expr_as(Expr::col((TAG_ANCESTORS, PostgresTag::Kana)), PostgresTagRelation::AncestorKana)
        .expr_as(Expr::col((TAG_ANCESTORS, PostgresTag::Aliases)), PostgresTagRelation::AncestorAliases)
        .expr_as(Expr::col((TAG_ANCESTORS, PostgresTag::CreatedAt)), PostgresTagRelation::AncestorCreatedAt)
        .expr_as(Expr::col((TAG_ANCESTORS, PostgresTag::UpdatedAt)), PostgresTagRelation::AncestorUpdatedAt)
        .expr_as(Expr::col((TAG_DESCENDANTS, PostgresTag::Id)), PostgresTagRelation::DescendantId)
        .expr_as(Expr::col((TAG_DESCENDANTS, PostgresTag::Name)), PostgresTagRelation::DescendantName)
        .expr_as(Expr::col((TAG_DESCENDANTS, PostgresTag::Kana)), PostgresTagRelation::DescendantKana)
        .expr_as(Expr::col((TAG_DESCENDANTS, PostgresTag::Aliases)), PostgresTagRelation::DescendantAliases)
        .expr_as(Expr::col((TAG_DESCENDANTS, PostgresTag::CreatedAt)), PostgresTagRelation::DescendantCreatedAt)
        .expr_as(Expr::col((TAG_DESCENDANTS, PostgresTag::UpdatedAt)), PostgresTagRelation::DescendantUpdatedAt)
        .from(PostgresTagPath::Table)
        .join_as(
            JoinType::InnerJoin,
            PostgresTag::Table,
            TAG_ANCESTORS,
            Expr::col((TAG_ANCESTORS, PostgresTag::Id))
                .equals((PostgresTagPath::Table, PostgresTagPath::AncestorId)),
        )
        .join_as(
            JoinType::InnerJoin,
            PostgresTag::Table,
            TAG_DESCENDANTS,
            Expr::col((TAG_DESCENDANTS, PostgresTag::Id))
                .equals((PostgresTagPath::Table, PostgresTagPath::DescendantId)),
        )
        .and_where(Expr::col(PostgresTagPath::Distance).lte(1i32))
        .order_by(PostgresTagPath::Distance, Order::Asc)
        .order_by((TAG_ANCESTORS, PostgresTag::Kana), Order::Asc)
        .order_by((TAG_DESCENDANTS, PostgresTag::Kana), Order::Asc)
        .cond_where(relatives)
        .build_sqlx(PostgresQueryBuilder);

    let rows: Vec<_> = sqlx::query_as_with::<_, PostgresTagRelativeRow, _>(&sql, values)
        .fetch(&mut *conn)
        .try_collect()
        .await
        .map_err(Error::other)?;

    let mut relations: OrderMap<_, _> = rows.into_iter().collect();
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
            Cond::all()
                .add(Expr::col(PostgresTagPath::DescendantId).eq(PostgresTagId::from(id)))
                .add(Expr::col(PostgresTagPath::AncestorId).ne(Expr::col(PostgresTagPath::DescendantId)))
        )
        .take()
}

fn descendant_relations(id: TagId) -> SelectStatement {
    const TAG_PATH_ANCESTORS: &str = "tag_path_ancestors";
    const TAG_PATH_DESCENDANTS: &str = "tag_path_descendants";

    Query::select()
        .expr(Expr::col((TAG_PATH_ANCESTORS, PostgresTagPath::AncestorId)))
        .expr(Expr::col((TAG_PATH_DESCENDANTS, PostgresTagPath::DescendantId)))
        .expr(Expr::col((TAG_PATH_ANCESTORS, PostgresTagPath::Distance))
            .add(Expr::col((TAG_PATH_DESCENDANTS, PostgresTagPath::Distance))))
        .from_as(PostgresTagPath::Table, TAG_PATH_ANCESTORS)
        .join_as(
            JoinType::InnerJoin,
            PostgresTagPath::Table,
            TAG_PATH_DESCENDANTS,
            Expr::col((TAG_PATH_ANCESTORS, PostgresTagPath::DescendantId))
                .equals((TAG_PATH_DESCENDANTS, PostgresTagPath::AncestorId)),
        )
        .cond_where(
            Cond::all()
                .add(Expr::col((TAG_PATH_ANCESTORS, PostgresTagPath::DescendantId)).eq(PostgresTagId::from(id)))
                .add(Expr::col((TAG_PATH_DESCENDANTS, PostgresTagPath::AncestorId)).eq(PostgresTagId::from(id)))
                .add(Expr::col((TAG_PATH_ANCESTORS, PostgresTagPath::AncestorId)).ne(PostgresTagId::from(id)))
                .add(Expr::col((TAG_PATH_DESCENDANTS, PostgresTagPath::DescendantId)).ne(PostgresTagId::from(id)))
        )
        .take()
}

async fn attach_parent(tx: &mut Transaction<'_, Postgres>, id: TagId, parent_id: TagId) -> Result<()> {
    let mut tx = tx.begin().await.map_err(Error::other)?;

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
                    Expr::val(PostgresTagId::from(id)).into(),
                    Expr::col(PostgresTagPath::Distance).add(1i32),
                ])
                .from(PostgresTagPath::Table)
                .and_where(Expr::col(PostgresTagPath::DescendantId).eq(PostgresTagId::from(parent_id)))
                .take()
        )
        .map_err(Error::other)?
        .build_sqlx(PostgresQueryBuilder);

    match sqlx::query_with(&sql, values).execute(&mut *tx).await {
        Ok(_) => (),
        Err(sqlx::Error::Database(e)) if e.is_foreign_key_violation() => return Err(ErrorKind::TagNotFound { id })?,
        Err(e) => return Err(Error::other(e)),
    }

    let (sql, values) = Query::insert()
        .into_table(PostgresTagPath::Table)
        .columns([
            PostgresTagPath::AncestorId,
            PostgresTagPath::DescendantId,
            PostgresTagPath::Distance,
        ])
        .select_from(descendant_relations(id))
        .map_err(Error::other)?
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(&mut *tx)
        .await
        .map_err(Error::other)?;

    tx.commit().await.map_err(Error::other)?;
    Ok(())
}

async fn detach_parent(tx: &mut Transaction<'_, Postgres>, id: TagId) -> Result<()> {
    let mut tx = tx.begin().await.map_err(Error::other)?;

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

    sqlx::query_with(&sql, values)
        .execute(&mut *tx)
        .await
        .map_err(Error::other)?;

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

    sqlx::query_with(&sql, values)
        .execute(&mut *tx)
        .await
        .map_err(Error::other)?;

    tx.commit().await.map_err(Error::other)?;
    Ok(())
}

impl TagsRepository for PostgresTagsRepository {
    #[tracing::instrument(skip_all)]
    async fn create<T>(&self, name: &str, kana: &str, aliases: T, parent_id: Option<TagId>, depth: TagDepth) -> Result<Tag>
    where
        T: Iterator<Item = String> + Send,
    {
        let mut tx = self.pool.begin().await.map_err(Error::other)?;

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
                aliases.collect::<Vec<_>>().into(),
            ])
            .map_err(Error::other)?
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
            .fetch_one(&mut *tx)
            .await
            .map_err(Error::other)?
            .into();

        let (sql, values) = Query::insert()
            .into_table(PostgresTagPath::Table)
            .columns([
                PostgresTagPath::AncestorId,
                PostgresTagPath::DescendantId,
                PostgresTagPath::Distance,
            ])
            .values([
                PostgresTagId::from(tag.id).into(),
                PostgresTagId::from(tag.id).into(),
                0.into(),
            ])
            .map_err(Error::other)?
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&mut *tx)
            .await
            .map_err(Error::other)?;

        let parent_id = parent_id.unwrap_or_default();
        attach_parent(&mut tx, tag.id, parent_id).await?;

        let mut relatives = fetch_tag_relatives(&mut tx, [tag.id], depth, false).await?;
        let tag = relatives.pop().ok_or(ErrorKind::TagNotFound { id: tag.id })?;

        tx.commit().await.map_err(Error::other)?;
        Ok(tag)
    }

    #[tracing::instrument(skip_all)]
    async fn fetch_by_ids<T>(&self, ids: T, depth: TagDepth) -> Result<Vec<Tag>>
    where
        T: Iterator<Item = TagId> + Send,
    {
        let mut conn = self.pool.acquire().await.map_err(Error::other)?;

        let tags = fetch_tag_relatives(&mut conn, ids, depth, false).await?;
        Ok(tags)
    }

    #[tracing::instrument(skip_all)]
    async fn fetch_by_name_or_alias_like(&self, name_or_alias_like: &str, depth: TagDepth) -> Result<Vec<Tag>> {
        let mut conn = self.pool.acquire().await.map_err(Error::other)?;

        const TAGS_ALIASES: &str = "tags_aliases";
        const ALIAS: &str = "alias";

        let name_or_alias_like = format!(
            "%{}%",
            name_or_alias_like
                .cow_replace('\\', "\\\\")
                .cow_replace('%', "\\%")
                .cow_replace('_', "\\_"),
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
                    .expr_as(ArrayExpr::unnest(Expr::col(PostgresTag::Aliases)), ALIAS)
                    .from(PostgresTag::Table)
                    .take(),
                TAGS_ALIASES,
                Expr::col((TAGS_ALIASES, PostgresTag::Id)).equals((PostgresTag::Table, PostgresTag::Id)),
            )
            .cond_where(
                Cond::any()
                    .add(Expr::col(PostgresTag::Name).ilike(LikeExpr::new(name_or_alias_like.clone())))
                    .add(Expr::col(PostgresTag::Kana).ilike(LikeExpr::new(name_or_alias_like.clone())))
                    .add(Expr::col(ALIAS).ilike(LikeExpr::new(name_or_alias_like))),
            )
            .build_sqlx(PostgresQueryBuilder);

        let ids: Vec<_> = sqlx::query_as_with::<_, PostgresTagRow, _>(&sql, values)
            .fetch(&mut *conn)
            .map_ok(|r| TagId::from(r.id))
            .try_collect()
            .await
            .map_err(Error::other)?;

        let tags = fetch_tag_relatives(&mut conn, ids, depth, false).await?;
        Ok(tags)
    }

    #[tracing::instrument(skip_all)]
    async fn fetch_all(&self, depth: TagDepth, root: bool, cursor: Option<(String, TagId)>, order: repository::Order, direction: repository::Direction, limit: u64) -> Result<Vec<Tag>> {
        let mut conn = self.pool.acquire().await.map_err(Error::other)?;

        let (comparison, order, rev) = match (order, direction) {
            (repository::Order::Ascending, repository::Direction::Forward) => (BinOper::GreaterThan, Order::Asc, false),
            (repository::Order::Ascending, repository::Direction::Backward) => (BinOper::SmallerThan, Order::Desc, true),
            (repository::Order::Descending, repository::Direction::Forward) => (BinOper::SmallerThan, Order::Desc, false),
            (repository::Order::Descending, repository::Direction::Backward) => (BinOper::GreaterThan, Order::Asc, true),
        };

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
            .and_where(Expr::col((PostgresTag::Table, PostgresTag::Id)).ne(PostgresTagId::from(TagId::root())))
            .and_where_option(
                cursor.map(|(kana, tag_id)| {
                    Expr::tuple([
                        Expr::col((PostgresTag::Table, PostgresTag::Kana)).into(),
                        Expr::col((PostgresTag::Table, PostgresTag::Id)).into(),
                    ]).binary(comparison, Expr::tuple([
                        Expr::value(kana),
                        Expr::value(PostgresTagId::from(tag_id)),
                    ]))
                })
            )
            .order_by((PostgresTag::Table, PostgresTag::Kana), order.clone())
            .order_by((PostgresTag::Table, PostgresTag::Id), order)
            .limit(limit);

        if root {
            query
                .join(
                    JoinType::InnerJoin,
                    PostgresTagPath::Table,
                    Expr::col((PostgresTagPath::Table, PostgresTagPath::DescendantId))
                        .equals((PostgresTag::Table, PostgresTag::Id)),
                )
                .and_where(Expr::col((PostgresTagPath::Table, PostgresTagPath::AncestorId)).eq(PostgresTagId::from(TagId::root())))
                .and_where(Expr::col((PostgresTagPath::Table, PostgresTagPath::Distance)).eq(1));
        }

        let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
        let ids: OrderSet<_> = sqlx::query_as_with::<_, PostgresTagRow, _>(&sql, values)
            .fetch(&self.pool)
            .map_ok(|r| TagId::from(r.id))
            .try_collect()
            .await
            .map_err(Error::other)?;

        let depth = match root {
            true => TagDepth::new(1, depth.children() + 1),
            false => depth,
        };

        let mut tags = fetch_tag_relatives(&mut conn, ids.iter().cloned(), depth, root).await?;
        tags.sort_unstable_by(|a, b| {
            let ord = Option::zip(ids.get_index_of(&a.id), ids.get_index_of(&b.id))
                .map(|(a, b)| Ord::cmp(&a, &b))
                .unwrap_or_else(|| Ord::cmp(&a.id, &b.id));
            if rev {
                ord.reverse()
            } else {
                ord
            }
        });

        Ok(tags)
    }

    #[tracing::instrument(skip_all)]
    async fn update_by_id<T, U>(&self, id: TagId, name: Option<String>, kana: Option<String>, add_aliases: T, remove_aliases: U, depth: TagDepth) -> Result<Tag>
    where
        T: Iterator<Item = String> + Send,
        U: Iterator<Item = String> + Send,
    {
        if id.is_root() {
            return Err(ErrorKind::TagUpdatingRoot)?;
        }

        let mut tx = self.pool.begin().await.map_err(Error::other)?;

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
            .and_where(Expr::col(PostgresTag::Id).eq(PostgresTagId::from(id)))
            .lock(LockType::Update)
            .build_sqlx(PostgresQueryBuilder);

        let mut tag = match sqlx::query_as_with::<_, PostgresTagRow, _>(&sql, values).fetch_one(&mut *tx).await {
            Ok(row) => Tag::from(row),
            Err(sqlx::Error::RowNotFound) => return Err(ErrorKind::TagNotFound { id })?,
            Err(e) => return Err(Error::other(e)),
        };

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
            .value(PostgresTag::Aliases, Vec::from(aliases))
            .value(PostgresTag::UpdatedAt, Expr::current_timestamp())
            .and_where(Expr::col(PostgresTag::Id).eq(PostgresTagId::from(id)))
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&mut *tx)
            .await
            .map_err(Error::other)?;

        let mut relatives = fetch_tag_relatives(&mut tx, [tag.id], depth, false).await?;
        let tag = relatives.pop().ok_or(ErrorKind::TagNotFound { id: tag.id })?;

        tx.commit().await.map_err(Error::other)?;
        Ok(tag)
    }

    #[tracing::instrument(skip_all)]
    async fn attach_by_id(&self, id: TagId, parent_id: TagId, depth: TagDepth) -> Result<Tag> {
        if id.is_root() || parent_id.is_root() {
            return Err(ErrorKind::TagAttachingRoot)?;
        }

        if id == parent_id {
            return Err(ErrorKind::TagAttachingToItself { id })?;
        }

        let mut tx = self.pool.begin().await.map_err(Error::other)?;

        let (sql, values) = Query::select()
            .expr(Expr::col(Asterisk).count())
            .from(PostgresTagPath::Table)
            .and_where(Expr::col(PostgresTagPath::AncestorId).eq(PostgresTagId::from(id)))
            .and_where(Expr::col(PostgresTagPath::DescendantId).eq(PostgresTagId::from(parent_id)))
            .build_sqlx(PostgresQueryBuilder);

        let count: i64 = sqlx::query_with(&sql, values)
            .fetch_one(&mut *tx)
            .await
            .and_then(|r| r.try_get(0))
            .map_err(Error::other)?;

        if count > 0 {
            return Err(ErrorKind::TagAttachingToDescendant { id })?;
        }

        detach_parent(&mut tx, id).await?;
        attach_parent(&mut tx, id, parent_id).await?;

        let mut relatives = fetch_tag_relatives(&mut tx, [id], depth, false).await?;
        let tag = relatives.pop().ok_or(ErrorKind::TagNotFound { id })?;

        tx.commit().await.map_err(Error::other)?;
        Ok(tag)
    }

    #[tracing::instrument(skip_all)]
    async fn detach_by_id(&self, id: TagId, depth: TagDepth) -> Result<Tag> {
        if id.is_root() {
            return Err(ErrorKind::TagDetachingRoot)?;
        }

        let mut tx = self.pool.begin().await.map_err(Error::other)?;

        detach_parent(&mut tx, id).await?;
        attach_parent(&mut tx, id, TagId::root()).await?;

        let mut relatives = fetch_tag_relatives(&mut tx, [id], depth, false).await?;
        let tag = relatives.pop().ok_or(ErrorKind::TagNotFound { id })?;

        tx.commit().await.map_err(Error::other)?;
        Ok(tag)
    }

    #[tracing::instrument(skip_all)]
    async fn delete_by_id(&self, id: TagId, recursive: bool) -> Result<DeleteResult> {
        if id.is_root() {
            return Err(ErrorKind::TagDeletingRoot)?;
        }

        let mut tx = self.pool.begin().await.map_err(Error::other)?;

        let (sql, values) = Query::select()
            .columns([
                PostgresTagPath::DescendantId,
                PostgresTagPath::Distance,
            ])
            .from(PostgresTagPath::Table)
            .and_where(Expr::col(PostgresTagPath::AncestorId).eq(PostgresTagId::from(id)))
            .order_by(PostgresTagPath::AncestorId, Order::Asc)
            .order_by(PostgresTagPath::DescendantId, Order::Asc)
            .lock(LockType::Update)
            .build_sqlx(PostgresQueryBuilder);

        let (ids, children) = sqlx::query_as_with::<_, PostgresTagDescendantRow, _>(&sql, values)
            .fetch(&mut *tx)
            .try_fold((Vec::new(), Vec::new()), |(mut ids, mut children), row| async move {
                ids.push(row.descendant_id.clone());
                if row.distance == 1 {
                    children.push(TagId::from(row.descendant_id));
                }
                Ok((ids, children))
            })
            .await
            .map_err(Error::other)?;

        if !recursive && !children.is_empty() {
            return Err(ErrorKind::TagChildrenExist { id, children })?;
        }

        let (sql, values) = Query::delete()
            .from_table(PostgresTag::Table)
            .and_where(Expr::col(PostgresTag::Id).is_in(ids))
            .build_sqlx(PostgresQueryBuilder);

        let affected = sqlx::query_with(&sql, values)
            .execute(&mut *tx)
            .await
            .map_err(Error::other)?
            .rows_affected();

        tx.commit().await.map_err(Error::other)?;

        match affected {
            0 => Ok(DeleteResult::NotFound),
            count => Ok(DeleteResult::Deleted(count)),
        }
    }
}
