use derive_more::derive::Constructor;
use lazy_regex::regex_captures_iter;

pub trait QueryParserInterface: Send + Sync + 'static {
    fn parse<'a>(&self, query: &'a str) -> Vec<&'a str>;
}

#[derive(Clone, Constructor)]
pub struct QueryParser;

impl QueryParserInterface for QueryParser {
    fn parse<'a>(&self, query: &'a str) -> Vec<&'a str> {
        regex_captures_iter!(r#""(.*?)"|([^\u0020\u3000]+)"#, query)
            .flat_map(|c| c.iter().flatten().nth(1).map(|m| m.as_str()))
            .collect()
    }
}

#[cfg(test)]
mod tests;
