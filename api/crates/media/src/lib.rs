use application::service::media::MediaURLFactoryInterface;
use derive_more::Constructor;

#[derive(Constructor)]
pub struct FileMediaURLFactory {
    root_url: String,
}

#[derive(Constructor)]
pub struct NoopMediaURLFactory;

impl FileMediaURLFactory {
    const URL_PREFIX: &'static str = "file://";
}

impl MediaURLFactoryInterface for FileMediaURLFactory {
    fn public_url(&self, original_url: &str) -> Option<String> {
        original_url
            .strip_prefix(Self::URL_PREFIX)
            .map(|s| format!("{}{}", &self.root_url, s))
    }
}

impl MediaURLFactoryInterface for NoopMediaURLFactory {
    fn public_url(&self, _: &str) -> Option<String> {
        None
    }
}
