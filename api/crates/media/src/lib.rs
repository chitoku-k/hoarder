use std::borrow::Cow;

use application::service::media::MediaURLFactoryInterface;
use derive_more::Constructor;

pub use regex::Regex;

pub mod parser;

#[derive(Constructor)]
pub struct RegexMediaURLFactory {
    rewrite_from: Regex,
    rewrite_to: String,
}

#[derive(Constructor)]
pub struct NoopMediaURLFactory;

impl MediaURLFactoryInterface for RegexMediaURLFactory {
    fn rewrite_original_url(&self, original_url: String) -> String {
        match self.rewrite_from.replace(&original_url, &self.rewrite_to) {
            Cow::Borrowed(b) if b.len() == original_url.len() => original_url,
            Cow::Borrowed(b) => b.to_string(),
            Cow::Owned(o) => o,
        }
    }
}

impl MediaURLFactoryInterface for NoopMediaURLFactory {
    fn rewrite_original_url(&self, original_url: String) -> String {
        original_url
    }
}
