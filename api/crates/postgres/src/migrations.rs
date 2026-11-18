use domain::error::{Error, Result};

use icu_locale_core::LanguageIdentifier;
use sqlx::Postgres;
use sqlx_migrator::{migrator, vec_box};

mod v1;
mod v2;
mod v3;
mod v4;
mod v5;
mod v6;
mod v7;
mod v8;
mod v9;
mod v10;
mod v11;

pub struct Migrator(migrator::Migrator<Postgres>);

impl Migrator {
    pub fn new(langid: LanguageIdentifier) -> Result<Self> {
        let mut migrator = migrator::Migrator::default();
        migrator.add_migrations(vec_box![
            v1::V1Migration::new(),
            v2::V2Migration::new((v1::V1Migration::APP, v1::V1Migration::NAME)),
            v3::V3Migration::new((v2::V2Migration::APP, v2::V2Migration::NAME)),
            v4::V4Migration::new((v3::V3Migration::APP, v3::V3Migration::NAME)),
            v5::V5Migration::new((v4::V4Migration::APP, v4::V4Migration::NAME)),
            v6::V6Migration::new((v5::V5Migration::APP, v5::V5Migration::NAME)),
            v7::V7Migration::new((v6::V6Migration::APP, v6::V6Migration::NAME)),
            v8::V8Migration::new((v7::V7Migration::APP, v7::V7Migration::NAME)),
            v9::V9Migration::new((v8::V8Migration::APP, v8::V8Migration::NAME)),
            v10::V10Migration::new((v9::V9Migration::APP, v9::V9Migration::NAME), langid.clone()),
            v11::V11Migration::new((v10::V10Migration::APP, v10::V10Migration::NAME), langid),
        ]).map_err(Error::other)?;

        Ok(Self(migrator))
    }

    pub fn as_inner(&self) -> &migrator::Migrator<Postgres> {
        &self.0
    }
}
