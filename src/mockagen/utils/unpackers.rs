use std::str::FromStr;

use pest::iterators::{Pair, Pairs};

use crate::mockagen::{model::{Error, PackingError, Providence, SyntaxChildren, SyntaxTree, Value}, parser::Rule};

use super::error::{make_error_from_pair, make_error_from_providence, make_no_array_match_found_error};


pub fn unpack_range<'a, T>(rule: Rule, make_values: fn(T, T) -> Value, trees: Vec<SyntaxTree<'a>>) -> Result<Value, Error>
where T: FromStr, PackingError<'a>: From<T::Err>
{
    match vec_into_array_varied_length(trees)? {
        [ Some((rule_lower, providence_lower @ Providence { src: lower_bound, .. }, None))
        , Some((rule_upper, providence_upper @ Providence { src: upper_bound, .. }, None))
        ] if rule_lower == rule && rule_upper == rule =>
        {
            let lower_bound = lower_bound.parse::<T>()
                .map_err(|err| make_error_from_providence(providence_lower, PackingError::from(err)))?;
            let upper_bound = upper_bound.parse::<T>()
                .map_err(|err| make_error_from_providence(providence_upper, PackingError::from(err)))?;

            Ok(make_values(lower_bound, upper_bound))
        }

        nodes =>
            make_no_array_match_found_error(nodes),
    }
}

pub fn into_array<const N: usize>(pairs: Pairs<'_, Rule>) -> Result<[Pair<'_, Rule>; N], Error> {
    pairs.take(N)
        .collect::<Vec<Pair<'_, Rule>>>()
        .try_into()
        .map_err(|rules: Vec<Pair<'_, Rule>>| {
            let pair = rules.first().unwrap().clone();

            make_error_from_pair(&pair, PackingError::PairsCountMismatch(rules))
        })
}

pub fn vec_into_array_varied_length<const N: usize>(vec: Vec<SyntaxTree>) -> Result<[Option<(Rule, Providence, Option<SyntaxChildren>)>; N], Error> {
    vec.into_iter()
        .filter(|tree| tree.token.rule != Rule::TAB)
        .map(|tree| Some((tree.token.rule, tree.token.providence, tree.children)))
        .chain(std::iter::repeat(None))
        .take(N)
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|vec: Vec<Option<(Rule, Providence, Option<SyntaxChildren>)>>| {
            let providence = vec.first().unwrap().as_ref().unwrap().1.clone(); // TODO fix this

            make_error_from_providence(providence, PackingError::SyntaxChildrenArrayCastError(vec))
        })
}
