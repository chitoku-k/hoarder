use domain::error::{Error, Result};

use sqlx::Postgres;
use sqlx_migrator::{migrator::{self, Info}, vec_box};

mod v1;
mod v2;
mod v3;
mod v4;
mod v5;
mod v6;
mod v7;
mod v8;
mod v9;

pub struct Migrator(migrator::Migrator<Postgres>);

impl Migrator {
    pub fn new() -> Result<Self> {
        let mut migrator = migrator::Migrator::new();
        migrator.add_migrations(vec_box![
            v1::V1Migration,
            v2::V2Migration,
            v3::V3Migration,
            v4::V4Migration,
            v5::V5Migration,
            v6::V6Migration,
            v7::V7Migration,
            v8::V8Migration,
            v9::V9Migration,
        ]).map_err(Error::other)?;

        Ok(Self(migrator))
    }

    pub fn into_boxed_migrator(self) -> Box<migrator::Migrator<Postgres>> {
        Box::new(self.0)
    }
}
