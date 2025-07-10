use thiserror::Error;

use super::{evaluator::model::EvaluationError, parser::Rule};


#[derive(Debug, Error)]
pub enum MockadocError {
    #[error("failed to parse mockadoc file")]
    ParsingError(#[from] Box<pest::error::Error<Rule>>),

    #[error("{0}")]
    PackingError(#[from] lang_packer_model::generic_utils::PackingError<Rule>),

    #[error("{0}")]
    EvaluationError(#[from] EvaluationError),
}
