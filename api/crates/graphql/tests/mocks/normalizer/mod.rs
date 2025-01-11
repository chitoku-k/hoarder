use std::borrow::Cow;

use normalizer::NormalizerInterface;

mockall::mock! {
    pub NormalizerInterface {}

    impl NormalizerInterface for NormalizerInterface {
        fn normalize_str(&self, text: &str) -> Cow<'static, str>;
    }
}
