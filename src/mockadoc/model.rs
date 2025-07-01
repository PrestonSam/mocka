use thiserror::Error;

use crate::{mockagen::MockagenError, utils::error::LanguageError};

use super::{evaluator::model::EvaluationError, packer::model::PackingError, parser::Rule};


#[derive(Debug, Error)]
pub enum MockadocError {
    #[error("failed to parse mockadoc file")]
    ParsingError(#[from] Box<pest::error::Error<Rule>>),

    #[error("failed to pack mockadoc file")]
    PackingError(PackingError),

    // #[error("failed to pack mockadoc file")]
    #[error("{0}")]
    PackingError2(#[from] lang_packer_model::generic_utils::PackingError<Rule>),

    #[error("{0}")]
    EvaluationError(#[from] EvaluationError),
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
