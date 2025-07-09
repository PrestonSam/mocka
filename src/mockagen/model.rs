use std::fmt::Debug;

use thiserror::Error;

use crate::utils::error::LanguageError;
use super::{evaluator::model::EvaluationError, parser::Rule};

pub type PackingError2
    = lang_packer_model::generic_utils::PackingError<Rule>;

// #[derive(Debug)]
// pub enum MockagenErrorVariant {
//     PackingError2(PackingError2),
//     ParsingError(Box<pest::error::Error<Rule>>),
//     EvaluationError(EvaluationError),
// }

#[derive(Error, Debug)]
pub enum MockagenError {
    #[error("{0}")]
    ParsingError(#[from] pest::error::Error<Rule>),

    #[error("{0}")]
    PackingError2(#[from] PackingError2),

    #[error("{0}")]
    EvaluationError(#[from] EvaluationError),
}

// impl LanguageError for MockagenError {
//     type Rule = Rule;
//     type PackingError = PackingError2;
//     type EvaluationError = EvaluationError;

//     fn from_parsing_err(error: pest::error::Error<Rule>) -> Self {
//         MockagenError::from(Box::from(error))
//     }

//     fn from_packing_err(error: PackingError2) -> Self {
//         MockagenError::from(error)
//     }

//     fn from_eval_err(error: EvaluationError) -> Self {
//         MockagenError::from(error)
//     }
// }

// impl From<PackingError2> for MockagenErrorVariant {
//     fn from(value: PackingError2) -> Self {
//         MockagenErrorVariant::PackingError2(value)
//     }
// }

// impl From<pest::error::Error<Rule>> for MockagenErrorVariant {
//     fn from(value: pest::error::Error<Rule>) -> Self {
//         MockagenErrorVariant::ParsingError(Box::from(value))
//     }
// }

// impl From<EvaluationError> for MockagenErrorVariant {
//     fn from(value: EvaluationError) -> Self {
//         MockagenErrorVariant::EvaluationError(value)
//     }
// }

