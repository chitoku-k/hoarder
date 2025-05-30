pub trait MediaURLFactoryInterface: Send + Sync + 'static {
    fn public_url(&self, original_url: &str) -> Option<String>;
}
