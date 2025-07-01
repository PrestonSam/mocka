use pest_derive::Parser;
use pest::{iterators::Pairs, Parser};
use lang_packer_model::generic_utils::DropRules;

use crate::mockadoc::model::MockadocError;


#[derive(Parser)]
#[grammar = "mockadoc/parser/parser.pest"]
pub struct MockadocParser;

pub fn parse_mockadoc(code: &str) -> Result<Pairs<'_, Rule>, MockadocError> {
    MockadocParser::parse(Rule::body, code)
        .map_err(|err| {
            println!("{}", &err.to_string());
            MockadocError::from(Box::from(err))
        })
}

impl DropRules for Rule {
    fn get_drop_rules(&self) -> Vec<Self> {
        vec![ /* TODO drop rules */ ]
    }
}