
pub trait LanguageError {
    type Rule;
    type PackingError;
    type EvaluationError;

    fn from_parsing_err(error: pest::error::Error<Self::Rule>) -> Self;

    fn from_packing_err(error: Self::PackingError) -> Self;

    fn from_eval_err(error: Self::EvaluationError) -> Self;
}
