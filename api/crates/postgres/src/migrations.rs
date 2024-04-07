use sqlx::Postgres;
use sqlx_migrator::{migrator::{self, Info}, vec_box};

mod v1;
mod v2;

pub struct Migrator(migrator::Migrator<Postgres, State>);

impl Migrator {
    pub fn new() -> Self {
        let mut migrator = migrator::Migrator::new(State);
        migrator.add_migrations(vec_box![
            v1::V1Migration,
            v2::V2Migration,
        ]);

        Self(migrator)
    }

    pub fn into_boxed_migrator(self) -> Box<migrator::Migrator<Postgres, State>> {
        Box::new(self.0)
    }
}

impl Default for Migrator {
    fn default() -> Self {
        Self::new()
    }
}

pub struct State;
