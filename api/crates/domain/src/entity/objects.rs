use derive_more::Constructor;

#[derive(Clone, Constructor, Debug, Eq, PartialEq)]
pub struct Entry {
    pub name: String,
    pub path: String,
    pub kind: Kind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    Container,
    Object,
    Unknown,
}
