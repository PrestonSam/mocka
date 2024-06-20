use crate::utils::error::LanguageError;

use super::{evaluator::model::EvaluationError, packer::model::PackingError, parser::Rule};


#[derive(Debug)]
pub enum MockadocError {
    ParsingError(Box<pest::error::Error<Rule>>),
    PackingError(PackingError),
    EvaluationError(EvaluationError),
}

impl From<pest::error::Error<Rule>> for MockadocError {
    fn from(value: pest::error::Error<Rule>) -> Self {
        MockadocError::ParsingError(Box::from(value))
    }
}

impl LanguageError for MockadocError {
    type Rule = Rule;
    type PackingError = PackingError;
    type EvaluationError = EvaluationError;

    fn from_parsing_err(error: pest::error::Error<Rule>) -> Self {
        MockadocError::ParsingError(Box::from(error))
    }

    fn from_packing_err(error: PackingError) -> Self {
        MockadocError::PackingError(error)
    }

    fn from_eval_err(error: EvaluationError) -> Self {
        MockadocError::EvaluationError(error)
    }
}