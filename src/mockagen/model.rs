use std::fmt::Debug;

use lang_packer_model::generic_utils::PackingError;
use thiserror::Error;

use super::{evaluator::model::EvaluationError, parser::Rule};

#[derive(Error, Debug)]
pub enum MockagenError {
    #[error("{0}")]
    ParsingError(#[from] pest::error::Error<Rule>),

    #[error("{0}")]
    PackingError2(#[from] PackingError<Rule>),

    #[error("{0}")]
    EvaluationError(#[from] EvaluationError),
}
