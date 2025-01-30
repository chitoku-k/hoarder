use std::borrow::Cow;

use pretty_assertions::assert_matches;

use crate::kana::normalize;

#[test]
fn normalized() {
    let actual = normalize("normalized: うゔウヴｳﾞはぱハパﾊﾟ");
    assert_matches!(actual, Cow::Borrowed(s) if s == "normalized: うゔウヴｳﾞはぱハパﾊﾟ");
}

#[test]
fn denormalized() {
    let actual = normalize("denormalized: うゔゔう゛うﾞウヴヴウ゛ウﾞｳｳ゙ｳ゛ｳﾞはぱぱは゜はﾟハパパハ゜ハﾟﾊﾊ゚ﾊ゜ﾊﾟ");
    assert_matches!(actual, Cow::Owned(s) if s == "denormalized: うゔゔゔゔウヴヴヴヴｳｳﾞｳﾞｳﾞはぱぱぱぱハパパパパﾊﾊﾟﾊﾟﾊﾟ");
}
