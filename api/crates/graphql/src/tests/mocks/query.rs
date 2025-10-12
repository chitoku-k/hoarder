use query::QueryParserInterface;

mockall::mock! {
    pub(crate) QueryParserInterface {}

    impl QueryParserInterface for QueryParserInterface {
        fn parse(&self, query: &str) -> Vec<&'static str>;
    }
}
