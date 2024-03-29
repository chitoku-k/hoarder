use domain::{
    entity::objects::{Entry, EntryUrl},
    error::{Error, ErrorKind, Result},
};
use percent_encoding::{percent_decode_str, utf8_percent_encode, AsciiSet, CONTROLS};

pub mod filesystem;

pub(crate) trait StorageEntry {
    fn into_entry(self) -> Entry;
}

pub(crate) trait StorageEntryUrl: TryFrom<EntryUrl, Error = Error> {
    const URL_PREFIX: &'static str;

    const RFC3986_PATH: &'static AsciiSet = &CONTROLS
        .add(b' ')
        .add(b'"')
        .add(b'#')
        .add(b'%')
        .add(b'<')
        .add(b'>')
        .add(b'?')
        .add(b'^')
        .add(b'`')
        .add(b'{')
        .add(b'}');

    fn from_path_str(path: &str) -> Result<Self> {
        let url = format!(
            "{}{}",
            Self::URL_PREFIX,
            utf8_percent_encode(path, Self::RFC3986_PATH),
        );

        Self::try_from(EntryUrl::from(url))
    }

    fn to_path_string(url: &str) -> Result<String> {
        let path = url
            .strip_prefix(Self::URL_PREFIX)
            .ok_or_else(|| ErrorKind::ObjectUrlUnsupported { url: url.to_string() })?;

        let path = percent_decode_str(path)
            .decode_utf8()
            .map_err(|e| Error::new(ErrorKind::ObjectPathInvalid, e))?
            .to_string();

        Ok(path)
    }

    fn into_url(self) -> EntryUrl;
}
