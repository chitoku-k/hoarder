pub mod external_services;
pub mod media;
pub mod objects;
pub mod replicas;
pub mod sources;
pub mod tag_types;
pub mod tags;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Order {
    Ascending,
    Descending,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Forward,
    Backward,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeleteResult {
    NotFound,
    Deleted(u64),
}

impl DeleteResult {
    pub const fn is_not_found(&self) -> bool {
        matches!(self, DeleteResult::NotFound)
    }

    pub const fn is_deleted(&self) -> bool {
        matches!(self, DeleteResult::Deleted(_))
    }
}
