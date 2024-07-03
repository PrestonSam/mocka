use pest_derive::Parser;
use pest::{iterators::Pairs, Parser};

use crate::{mockagen::MockagenError, utils::error::LanguageError};


#[derive(Parser)]
#[grammar = "mockagen/parser/parser.pest"]
pub struct MockagenParser;

pub fn parse_mockagen(code: &str) -> Result<Pairs<'_, Rule>, MockagenError> {
    MockagenParser::parse(Rule::body, code)
        .map_err(MockagenError::from_parsing_err)
}
