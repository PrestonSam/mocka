use std::fmt::Debug;

use crate::utils::error::LanguageError;
use super::{evaluator::model::EvaluationError, packer::model::PackingError, parser::{Rule, Rule2}};

pub type PackingError2
    = token_packer::generic_utils::PackingError<Rule2>;

#[derive(Debug)]
pub enum MockagenErrorVariant {
    PackingError(PackingError),
    PackingError2(PackingError2),
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

impl From<PackingError2> for MockagenError {
    fn from(value: PackingError2) -> Self {
        MockagenError {
            error: MockagenErrorVariant::from(value),
        }
    }
}

impl From<PackingError> for MockagenErrorVariant {
    fn from(value: PackingError) -> Self {
        MockagenErrorVariant::PackingError(value)
    }
}

impl From<PackingError2> for MockagenErrorVariant {
    fn from(value: PackingError2) -> Self {
        MockagenErrorVariant::PackingError2(value)
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
