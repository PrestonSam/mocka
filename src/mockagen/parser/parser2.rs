use pest_derive::Parser;
use pest::{iterators::Pairs, Parser};
use token_packer::generic_utils::DropRules;

use crate::{mockagen::MockagenError, utils::error::LanguageError};


#[derive(Parser)]
#[grammar = "mockagen/parser/parser2.pest"]
pub struct MockagenParser;

pub fn parse_mockagen2(code: &str) -> Result<Pairs<'_, Rule>, MockagenError> {
    Ok(MockagenParser::parse(Rule::body, code).unwrap()) // TODO switch error handling back on again
        // .map_err(MockagenError::from_parsing_err)
}

impl DropRules for Rule {
    fn get_drop_rules(&self) -> Vec<Self> {
        vec![]
    }
}