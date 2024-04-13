use std::iter::once;

use pest::iterators::Pair;

use crate::mockagen::{model::{AnnotatedParsingError, Error, PackingError, Providence, RuleData}, parser::Rule};

pub fn make_error_from_providence<'a>(providence: Providence<'a>, error: PackingError<'a>) -> Error<'a> {
    Error::from(AnnotatedParsingError { error, providence })
}

pub fn make_error_from_pair<'a>(pair: &Pair<'a, Rule>, error: PackingError<'a>) -> Error<'a> {
    Error::from(AnnotatedParsingError {
        error,
        providence: Providence { span: pair.as_span(), src: pair.as_str() },
    })
}


pub fn make_no_match_found_error_many<'a, T, const N: usize>(rule_data: RuleData<'a>, tail: [Option<RuleData<'a>>; N]) -> Result<T, Error<'a>> {
    let providence = rule_data.inner.providence.clone();
    let vec = once(Some(rule_data))
        .chain(tail.into_iter())
        .collect();

    Err(make_error_from_providence(providence, PackingError::ASTPackerNoMatchFound(vec)))
}

pub fn make_empty_inner_error<'a, T>(providence: Providence<'a>) -> Result<T, Error<'a>> {
    Err(make_error_from_providence(providence, PackingError::ASTPackerEmptyInner))
}

pub fn make_no_match_found_error_single<'a, T>(rule_data: RuleData<'a>) -> Result<T, Error<'a>> {
    Err(make_error_from_providence(
        rule_data.inner.providence.clone(),
        PackingError::ASTPackerNoMatchFound(vec![Some(rule_data)])
    ))
}
