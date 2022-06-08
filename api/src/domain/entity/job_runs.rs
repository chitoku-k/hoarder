use chrono::NaiveDateTime;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JobRun {
    pub phase: String,
    pub message: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
