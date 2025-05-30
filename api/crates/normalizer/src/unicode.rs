use std::borrow::Cow;

use icu_normalizer::ComposingNormalizerBorrowed;

const NFC: ComposingNormalizerBorrowed<'_> = ComposingNormalizerBorrowed::new_nfc();

fn should_normalize(c: char) -> bool {
    // https://developer.apple.com/library/archive/qa/qa1173/_index.html
    !('\u{2000}'..='\u{2FFF}').contains(&c)
        && !('\u{F900}'..='\u{FAFF}').contains(&c)
        && !('\u{2F800}'..='\u{2FAFF}').contains(&c)
}

pub(crate) fn normalize(text: &str) -> Cow<str> {
    let (s, last_match) = text.match_indices(should_normalize).fold((Cow::Borrowed(""), 0), |(s, last_match), (i, m)| {
        let s = if !NFC.is_normalized(m) {
            let mut s = match s {
                Cow::Borrowed(_) => {
                    let mut s = String::with_capacity(text.len());
                    s.push_str(&text[..i]);
                    s
                },
                Cow::Owned(mut s) => {
                    s.push_str(&text[last_match..i]);
                    s
                },
            };
            NFC.normalize_to(m, &mut s).unwrap();
            s.into()
        } else {
            match s {
                Cow::Borrowed(_) => text[..i + m.len()].into(),
                Cow::Owned(mut s) => {
                    s.push_str(&text[last_match..i + m.len()]);
                    s.into()
                },
            }
        };
        (s, i + m.len())
    });

    match s {
        Cow::Borrowed(_) => text.into(),
        Cow::Owned(mut s) => {
            s.push_str(&text[last_match..]);
            s.into()
        },
    }
}
