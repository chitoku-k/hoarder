use pretty_assertions::assert_eq;

use crate::{QueryParser, QueryParserInterface};

#[test]
fn single_word() {
    let query_parser = QueryParser::new();

    let actual = query_parser.parse("keyword");
    assert_eq!(actual, vec!["keyword"]);

    let actual = query_parser.parse(r#"" keyword1  keyword2""#);
    assert_eq!(actual, vec![" keyword1  keyword2"]);
}

#[test]
fn multiple_words() {
    let query_parser = QueryParser::new();

    let actual = query_parser.parse("keyword1 keyword2   keyword3 　keyword4");
    assert_eq!(actual, vec!["keyword1", "keyword2", "keyword3", "keyword4"]);

    let actual = query_parser.parse(r#"keyword1 keyword2  " keyword3  keyword4"　keyword5"#);
    assert_eq!(actual, vec!["keyword1", "keyword2", " keyword3  keyword4", "keyword5"]);
}
