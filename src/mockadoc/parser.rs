use pest_derive::Parser;
use pest::{iterators::Pairs, Parser};

use super::model::Error;


#[derive(Parser)]
#[grammar = "mockadoc/parser.pest"]
pub struct MockadocParser;

pub fn parse_mockadoc(code: &str) -> Result<Pairs<'_, Rule>, Error> {
    MockadocParser::parse(Rule::body, code)
        .map_err(Error::from)
}
