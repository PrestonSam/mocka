use pest_derive::Parser;
use pest::{iterators::Pairs, Parser};
use lang_packer_model::generic_utils::DropRules;

use crate::mockagen::MockagenError;


#[derive(Parser)]
#[grammar = "mockagen/parser/parser.pest"]
pub struct MockagenParser;

pub fn parse_mockagen(code: &str) -> Result<Pairs<'_, Rule>, MockagenError> {
    MockagenParser::parse(Rule::body, code)
        .map_err(Into::into)
}

impl DropRules for Rule {
    fn get_drop_rules(&self) -> Vec<Self> {
        vec![]
    }
}