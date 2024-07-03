use pest_derive::Parser;
use pest::{iterators::Pairs, Parser};

use crate::mockadoc::model::MockadocError;


#[derive(Parser)]
#[grammar = "mockadoc/parser/parser.pest"]
pub struct MockadocParser;

pub fn parse_mockadoc(code: &str) -> Result<Pairs<'_, Rule>, MockadocError> {
    MockadocParser::parse(Rule::body, code)
        .map_err(MockadocError::from)
}
