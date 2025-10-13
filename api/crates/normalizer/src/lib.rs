use std::borrow::Cow;

use derive_more::derive::Constructor;

mod kana;
mod unicode;

pub trait NormalizerInterface: Send + Sync + 'static {
    fn normalize<T>(&self, text: T) -> String
    where
        T: Into<String> + 'static,
    {
        let text = text.into();
        match self.normalize_str(&text) {
            Cow::Borrowed(s) if s.len() == text.len() => text,
            s => s.into_owned(),
        }
    }

    fn normalize_str<'a>(&self, text: &'a str) -> Cow<'a, str>;
}

#[derive(Clone, Constructor)]
pub struct Normalizer;

impl NormalizerInterface for Normalizer {
    fn normalize_str<'a>(&self, text: &'a str) -> Cow<'a, str> {
        let kana = kana::normalize(text);
        let unicode = unicode::normalize(&kana);
        match unicode {
            Cow::Borrowed(s) if s.len() == kana.len() => kana,
            s => s.into_owned().into(),
        }
    }
}

#[cfg(test)]
mod tests;
