use std::str::FromStr;

use crate::mockagen::{model::{PackingError, PackingErrorVariant, PrimitiveValue, Providence, SyntaxChildren, SyntaxTree}, parser::Rule};

use super::error::{make_error_from_providence, make_no_array_match_found_error, reformat_rule_matcher_vec};


pub fn unpack_range<T>(rule: Rule, make_values: fn(T, T) -> PrimitiveValue, trees: Vec<SyntaxTree>) -> Result<PrimitiveValue, PackingError>
where T: FromStr, PackingErrorVariant: From<T::Err>
{
    match vec_into_array_varied_length(trees)? {
        [ Some((rule_lower, providence_lower @ Providence { src: lower_bound, .. }, None))
        , Some((rule_upper, providence_upper @ Providence { src: upper_bound, .. }, None))
        ] if rule_lower == rule && rule_upper == rule =>
        {
            let lower_bound = lower_bound.parse::<T>()
                .map_err(|err| make_error_from_providence(providence_lower, PackingErrorVariant::from(err)))?;
            let upper_bound = upper_bound.parse::<T>()
                .map_err(|err| make_error_from_providence(providence_upper, PackingErrorVariant::from(err)))?;

            Ok(make_values(lower_bound, upper_bound))
        }

        nodes =>
            make_no_array_match_found_error(nodes),
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

            make_error_from_providence(providence, PackingErrorVariant::SyntaxChildrenArrayCastError(reformatted_vec))
        })
}
