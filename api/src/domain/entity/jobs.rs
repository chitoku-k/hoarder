use chrono::NaiveDateTime;
use derive_more::{Deref, Display, From};
use uuid::Uuid;

use crate::domain::entity::job_runs::JobRun;

#[derive(Clone, Copy, Debug, Default, Deref, Display, Eq, From, PartialEq)]
pub struct JobId(Uuid);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Job {
    pub id: JobId,
    pub content: serde_json::Value,
    pub runs: Vec<JobRun>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
