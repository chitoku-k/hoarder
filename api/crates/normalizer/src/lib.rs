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

#[derive(Constructor)]
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
mod tests {
    use std::borrow::Cow;

    use pretty_assertions::{assert_eq, assert_matches};

    use super::*;

    #[test]
    fn normalized() {
        let normalizer = Normalizer::new();

        let original = "normalized: うゔウヴｳﾞはぱハパﾊﾟάÅÅ神神兔兔".to_string();
        let expected = "normalized: うゔウヴｳﾞはぱハパﾊﾟάÅÅ神神兔兔";
        let ptr = original.as_ptr();

        let actual = normalizer.normalize_str(&original);
        assert_matches!(actual, Cow::Borrowed(s) if s == expected);

        let actual = normalizer.normalize(original);
        assert_eq!(actual, expected);
        assert_eq!(actual.as_ptr(), ptr);
    }

    #[test]
    fn denormalized() {
        let normalizer = Normalizer::new();

        let original = "denormalized: うゔゔう゛うﾞウヴヴウ゛ウﾞｳｳ゙ｳ゛ｳﾞはぱぱは゜はﾟハパパハ゜ハﾟﾊﾊ゚ﾊ゜ﾊﾟάάÅÅ神神兔兔".to_string();
        let expected = "denormalized: うゔゔゔゔウヴヴヴヴｳｳﾞｳﾞｳﾞはぱぱぱぱハパパパパﾊﾊﾟﾊﾟﾊﾟάάÅÅ神神兔兔";
        let ptr = original.as_ptr();

        let actual = normalizer.normalize_str(&original);
        assert_matches!(actual, Cow::Owned(s) if s == expected);

        let actual = normalizer.normalize(original);
        assert_eq!(actual, expected);
        assert_ne!(actual.as_ptr(), ptr);
    }
}
