use axum::http::{Error, HeaderName, HeaderValue, response::Builder};

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
