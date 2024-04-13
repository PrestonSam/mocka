use pest_derive::Parser;
use pest::{iterators::Pairs, Parser};

use super::model::Error;

#[derive(Parser)]
#[grammar = "mockagen/parser.pest"]
pub struct MockagenParser;

pub fn parse_mockagen(code: &str) -> Result<Pairs<'_, Rule>, Error> {
    MockagenParser::parse(Rule::body, code)
        .map_err(Error::from)
}
