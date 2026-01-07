use std::time::SystemTime;

use axum::http::{Error, HeaderMap, HeaderName, HeaderValue, response::Builder};
use chrono::{DateTime, Utc};
use headers::{self, Header, HeaderMapExt};

pub(crate) trait ResponseBuilderExt
where
    Self: Sized,
{
    fn header_opt<K, V>(self, key: K, value: Option<V>) -> Self
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<Error>;
}

impl ResponseBuilderExt for Builder {
    fn header_opt<K, V>(self, key: K, value: Option<V>) -> Self
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<Error>,
    {
        if let Some(value) = value {
            self.header(key, value)
        } else {
            self
        }
    }
}

trait IntoHeaderValue<H>
where
    Self: Into<H>,
    H: Header,
{
    fn into_header_value(self) -> HeaderValue {
        let mut map = HeaderMap::new();
        map.typed_insert(self.into());
        map.remove(H::name()).unwrap()
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct HttpContentLength(pub u64);

impl IntoHeaderValue<headers::ContentLength> for HttpContentLength {}

impl From<HttpContentLength> for headers::ContentLength {
    fn from(value: HttpContentLength) -> Self {
        Self(value.0)
    }
}

impl From<HttpContentLength> for HeaderValue {
    fn from(value: HttpContentLength) -> Self {
        value.into_header_value()
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct HttpETag(pub u64, pub Option<DateTime<Utc>>);

impl IntoHeaderValue<headers::ETag> for HttpETag {}

impl From<HttpETag> for headers::ETag {
    fn from(value: HttpETag) -> Self {
        let etag = match value {
            HttpETag(size, Some(updated_at)) => format!(r#""{size:x}-{:x}""#, updated_at.timestamp_micros()),
            HttpETag(size, None) => format!(r#""{size:x}""#),
        };

        etag.parse().unwrap()
    }
}

impl From<HttpETag> for HeaderValue {
    fn from(value: HttpETag) -> Self {
        value.into_header_value()
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct HttpLastModified(pub DateTime<Utc>);

impl IntoHeaderValue<headers::LastModified> for HttpLastModified {}

impl From<HttpLastModified> for headers::LastModified {
    fn from(value: HttpLastModified) -> Self {
        Self::from(SystemTime::from(value.0))
    }
}

impl From<HttpLastModified> for HeaderValue {
    fn from(value: HttpLastModified) -> Self {
        value.into_header_value()
    }
}
