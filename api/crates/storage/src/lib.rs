use domain::{entity::objects::{Entry, EntryUrl}, error::Error};

pub mod filesystem;

pub(crate) trait StorageEntry {
    fn into_entry(self) -> Entry;
}

pub(crate) trait StorageEntryUrl: TryFrom<EntryUrl, Error = Error> {
    const URL_PREFIX: &'static str;

    fn into_url(self) -> EntryUrl;
}

#[cfg(test)]
mod tests;
