use std::fmt::Debug;

use crate::utils::error::LanguageError;

use super::{evaluator::model::EvaluationError, packer::model::PackingError, parser::Rule};


#[derive(Debug)]
pub enum MockagenErrorVariant {
    PackingError(PackingError),
    ParsingError(Box<pest::error::Error<Rule>>),
    EvaluationError(EvaluationError),
}

#[derive(Debug)]
pub struct MockagenError { // It might be worth dropping the struct and only using MockagenErrorVariant
    error: MockagenErrorVariant,
}

impl LanguageError for MockagenError {
    type Rule = Rule;
    type PackingError = PackingError;
    type EvaluationError = EvaluationError;

    fn from_parsing_err(error: pest::error::Error<Rule>) -> Self {
        MockagenError {
            error: MockagenErrorVariant::ParsingError(Box::from(error))
        }
    }

    fn from_packing_err(error: PackingError) -> Self {
        MockagenError {
            error: MockagenErrorVariant::PackingError(error)
        }
    }

    fn from_eval_err(error: EvaluationError) -> Self {
        MockagenError {
            error: MockagenErrorVariant::EvaluationError(error)
        }
    }
}

impl From<PackingError> for MockagenErrorVariant {
    fn from(value: PackingError) -> Self {
        MockagenErrorVariant::PackingError(value)
    }
}

impl From<pest::error::Error<Rule>> for MockagenErrorVariant {
    fn from(value: pest::error::Error<Rule>) -> Self {
        MockagenErrorVariant::ParsingError(Box::from(value))
    }
}

impl From<EvaluationError> for MockagenErrorVariant {
    fn from(value: EvaluationError) -> Self {
        MockagenErrorVariant::EvaluationError(value)
    }
}
