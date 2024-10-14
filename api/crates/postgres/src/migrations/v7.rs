use async_trait::async_trait;
use sea_query::{ColumnDef, PostgresQueryBuilder, Query, Table};
use sqlx::{PgConnection, Postgres};
use sqlx_migrator::{error::Error, migration::Migration, operation::Operation, vec_box};

use crate::{migrations::State, tag_types::PostgresTagType};

pub(super) struct V7Migration;

impl Migration<Postgres, State> for V7Migration {
    fn app(&self) -> &str {
        "hoarder"
    }

    fn name(&self) -> &str {
        "tag_types_kana"
    }

    fn parents(&self) -> Vec<Box<dyn Migration<Postgres, State>>> {
        vec_box![]
    }

    fn operations(&self) -> Vec<Box<dyn Operation<Postgres, State>>> {
        vec_box![TagTypeKanaOperation]
    }
}

struct TagTypeKanaOperation;

#[async_trait]
impl Operation<Postgres, State> for TagTypeKanaOperation {
    async fn up(&self, connection: &mut PgConnection, _state: &State) -> Result<(), Error> {
        let sql = Table::alter()
            .table(PostgresTagType::Table)
            .add_column(ColumnDef::new(PostgresTagType::Kana).text())
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Query::update()
            .table(PostgresTagType::Table)
            .value(PostgresTagType::Kana, "")
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        let sql = Table::alter()
            .table(PostgresTagType::Table)
            .modify_column(ColumnDef::new(PostgresTagType::Kana).text().not_null())
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        Ok(())
    }

    async fn down(&self, connection: &mut PgConnection, _state: &State) -> Result<(), Error> {
        let sql = Table::alter()
            .table(PostgresTagType::Table)
            .drop_column(PostgresTagType::Kana)
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql).execute(&mut *connection).await?;

        Ok(())
    }
}
