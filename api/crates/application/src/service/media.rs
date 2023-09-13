#[cfg_attr(feature = "test-mock", mockall::automock)]
pub trait MediaURLFactoryInterface: Send + Sync + 'static {
    fn rewrite_original_url(&self, original_url: String) -> String;
}
