use std::borrow::Cow;

use pretty_assertions::{assert_eq, assert_matches};

use crate::{Normalizer, NormalizerInterface};

mod kana;
mod unicode;

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
