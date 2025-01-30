use std::borrow::Cow;

use pretty_assertions::assert_matches;

use crate::unicode::normalize;

#[test]
fn normalized() {
    let actual = normalize("normalized: άÅÅ神神兔兔");
    assert_matches!(actual, Cow::Borrowed(s) if s == "normalized: άÅÅ神神兔兔");
}

#[test]
fn denormalized() {
    let actual = normalize("denormalized: άάÅÅ神神兔兔");
    assert_matches!(actual, Cow::Owned(s) if s == "denormalized: άάÅÅ神神兔兔");
}
