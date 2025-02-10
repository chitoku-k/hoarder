use strum::EnumIs;

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

#[derive(Clone, Copy, Debug, EnumIs, Eq, PartialEq)]
pub enum DeleteResult {
    NotFound,
    Deleted(u64),
}
