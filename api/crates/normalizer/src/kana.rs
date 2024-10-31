use std::borrow::Cow;

use phf::{Map, phf_map};

static SOUND_MARKS: Map<char, &Map<char, &str>> = phf_map! {
    '゙' => &VOICED,
    '゚' => &SEMI_VOICED,
    '゛' => &VOICED,
    '゜' => &SEMI_VOICED,
    'ﾞ' => &VOICED_FULLWIDTH,
    'ﾟ' => &SEMI_VOICED_FULLWIDTH,
};

static VOICED: Map<char, &str> = phf_map! {
    'う' => "ゔ",
    'か' => "が",
    'き' => "ぎ",
    'く' => "ぐ",
    'け' => "げ",
    'こ' => "ご",
    'さ' => "ざ",
    'し' => "じ",
    'す' => "ず",
    'せ' => "ぜ",
    'そ' => "ぞ",
    'た' => "だ",
    'ち' => "ぢ",
    'つ' => "づ",
    'て' => "で",
    'と' => "ど",
    'は' => "ば",
    'ひ' => "び",
    'ふ' => "ぶ",
    'へ' => "べ",
    'ほ' => "ぼ",
    'ゝ' => "ゞ",
    'ウ' => "ヴ",
    'カ' => "ガ",
    'キ' => "ギ",
    'ク' => "グ",
    'ケ' => "ゲ",
    'コ' => "ゴ",
    'サ' => "ザ",
    'シ' => "ジ",
    'ス' => "ズ",
    'セ' => "ゼ",
    'ソ' => "ゾ",
    'タ' => "ダ",
    'チ' => "ヂ",
    'ツ' => "ヅ",
    'テ' => "デ",
    'ト' => "ド",
    'ハ' => "バ",
    'ヒ' => "ビ",
    'フ' => "ブ",
    'ヘ' => "ベ",
    'ホ' => "ボ",
    'ワ' => "ヷ",
    'ヰ' => "ヸ",
    'ヱ' => "ヹ",
    'ヲ' => "ヺ",
    'ヽ' => "ヾ",
    'ｳ' => "ｳﾞ",
    'ｶ' => "ｶﾞ",
    'ｷ' => "ｷﾞ",
    'ｸ' => "ｸﾞ",
    'ｹ' => "ｹﾞ",
    'ｺ' => "ｺﾞ",
    'ｻ' => "ｻﾞ",
    'ｼ' => "ｼﾞ",
    'ｽ' => "ｽﾞ",
    'ｾ' => "ｾﾞ",
    'ｿ' => "ｿﾞ",
    'ﾀ' => "ﾀﾞ",
    'ﾁ' => "ﾁﾞ",
    'ﾂ' => "ﾂﾞ",
    'ﾃ' => "ﾃﾞ",
    'ﾄ' => "ﾄﾞ",
    'ﾊ' => "ﾊﾞ",
    'ﾋ' => "ﾋﾞ",
    'ﾌ' => "ﾌﾞ",
    'ﾍ' => "ﾍﾞ",
    'ﾎ' => "ﾎﾞ",
    'ﾜ' => "ﾜﾞ",
    'ｦ' => "ｦﾞ",
};

static VOICED_FULLWIDTH: Map<char, &str> = phf_map! {
    'う' => "ゔ",
    'か' => "が",
    'き' => "ぎ",
    'く' => "ぐ",
    'け' => "げ",
    'こ' => "ご",
    'さ' => "ざ",
    'し' => "じ",
    'す' => "ず",
    'せ' => "ぜ",
    'そ' => "ぞ",
    'た' => "だ",
    'ち' => "ぢ",
    'つ' => "づ",
    'て' => "で",
    'と' => "ど",
    'は' => "ば",
    'ひ' => "び",
    'ふ' => "ぶ",
    'へ' => "べ",
    'ほ' => "ぼ",
    'ゝ' => "ゞ",
    'ウ' => "ヴ",
    'カ' => "ガ",
    'キ' => "ギ",
    'ク' => "グ",
    'ケ' => "ゲ",
    'コ' => "ゴ",
    'サ' => "ザ",
    'シ' => "ジ",
    'ス' => "ズ",
    'セ' => "ゼ",
    'ソ' => "ゾ",
    'タ' => "ダ",
    'チ' => "ヂ",
    'ツ' => "ヅ",
    'テ' => "デ",
    'ト' => "ド",
    'ハ' => "バ",
    'ヒ' => "ビ",
    'フ' => "ブ",
    'ヘ' => "ベ",
    'ホ' => "ボ",
    'ワ' => "ヷ",
    'ヰ' => "ヸ",
    'ヱ' => "ヹ",
    'ヲ' => "ヺ",
    'ヽ' => "ヾ",
};

static SEMI_VOICED: Map<char, &str> = phf_map! {
    'は' => "ぱ",
    'ひ' => "ぴ",
    'ふ' => "ぷ",
    'へ' => "ぺ",
    'ほ' => "ぽ",
    'ハ' => "パ",
    'ヒ' => "ピ",
    'フ' => "プ",
    'ヘ' => "ペ",
    'ホ' => "ポ",
    'ﾊ' => "ﾊﾟ",
    'ﾋ' => "ﾋﾟ",
    'ﾌ' => "ﾌﾟ",
    'ﾍ' => "ﾍﾟ",
    'ﾎ' => "ﾎﾟ",
};

static SEMI_VOICED_FULLWIDTH: Map<char, &str> = phf_map! {
    'は' => "ぱ",
    'ひ' => "ぴ",
    'ふ' => "ぷ",
    'へ' => "ぺ",
    'ほ' => "ぽ",
    'ハ' => "パ",
    'ヒ' => "ピ",
    'フ' => "プ",
    'ヘ' => "ペ",
    'ホ' => "ポ",
};

pub(crate) fn normalize(text: &str) -> Cow<str> {
    text.char_indices().fold(Cow::Borrowed(""), |s, (i, c)| {
        match SOUND_MARKS.get(&c).and_then(|v| s.chars().last().and_then(|c| v.get(&c))) {
            Some(v) => {
                let mut s = match s {
                    Cow::Borrowed(_) => {
                        let mut s = String::with_capacity(text.len());
                        s.push_str(&text[..i]);
                        s
                    },
                    Cow::Owned(s) => s,
                };
                s.pop();
                s.push_str(v);
                s.into()
            },
            None => {
                match s {
                    Cow::Borrowed(_) => text[..i + c.len_utf8()].into(),
                    Cow::Owned(mut s) => {
                        s.push(c);
                        s.into()
                    },
                }
            },
        }
    })
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use pretty_assertions::assert_matches;

    use super::*;

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
}
