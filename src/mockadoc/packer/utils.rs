use std::iter::once;

use crate::{mockadoc::parser::Rule, utils::packing::Providence};

use super::{error::reformat_rule_matcher_vec, model::{PackingError, PackingErrorVariant, SyntaxChildren, SyntaxTree}};

#[derive(Debug)]
pub enum FirstAndRest<T> {
    Both(T, Vec<T>),
    OnlyFirst(T),
    Neither,
}

pub fn vec_first_and_rest<T>(vec: Vec<T>) -> FirstAndRest<T> {
    let mut iter = vec.into_iter();

    match iter.next() {
        Some(first) => 
            match iter.next() {
                Some(second_value) => FirstAndRest::Both(first, once(second_value).chain(iter).collect()),
                None => FirstAndRest::OnlyFirst(first)
            }
        None =>
            FirstAndRest::Neither,
    }
}

pub fn vec_into_array_varied_length<const N: usize>(vec: Vec<SyntaxTree>) -> Result<[Option<(Rule, Providence, Option<SyntaxChildren>)>; N], PackingError> {
    vec.into_iter()
        .map(|tree| Some((tree.token.rule, tree.token.providence, tree.children)))
        .chain(std::iter::repeat(None))
        .take(N)
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|vec: Vec<Option<(Rule, Providence, Option<SyntaxChildren>)>>| {
            let (providence, reformatted_vec) = reformat_rule_matcher_vec(vec);

            PackingError::new(PackingErrorVariant::SyntaxChildrenArrayCastError(reformatted_vec))
                .with_providence(providence)
        })
}
