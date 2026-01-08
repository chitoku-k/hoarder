use std::convert::Infallible;

use axum::{extract::OptionalFromRequestParts, http::request::Parts};
use headers::{HeaderMapExt, IfMatch, IfModifiedSince, IfNoneMatch};

pub mod error;
pub mod server;
pub mod service;

#[derive(Debug)]
pub enum Precondition {
    IfNoneMatch(IfNoneMatch),
    IfMatch(IfMatch),
    IfModifiedSince(IfModifiedSince),
}

impl<S> OptionalFromRequestParts<S> for Precondition
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Option<Self>, Self::Rejection> {
        if let Some(if_none_match) = parts.headers.typed_get() {
            return Ok(Some(Precondition::IfNoneMatch(if_none_match)));
        }

        if let Some(if_match) = parts.headers.typed_get() {
            return Ok(Some(Precondition::IfMatch(if_match)));
        }

        if let Some(if_modified_since) = parts.headers.typed_get() {
            return Ok(Some(Precondition::IfModifiedSince(if_modified_since)));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests;
