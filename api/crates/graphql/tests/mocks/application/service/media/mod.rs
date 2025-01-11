use application::service::media::MediaURLFactoryInterface;

mockall::mock! {
    pub MediaURLFactoryInterface {}

    impl MediaURLFactoryInterface for MediaURLFactoryInterface {
        fn public_url(&self, original_url: &str) -> Option<String>;
    }
}
