use std::str::FromStr;

use pest::iterators::{Pair, Pairs};

use crate::mockagen::{model::{AnnotatedPairs, Error, PackingError, Providence, RuleData, Value}, parser::Rule};

use super::{error::{make_error_from_pair, make_error_from_providence, make_no_match_found_error_many}, parsing::is_not_tab};


pub fn unpack_range<'a, T>(rule: Rule, make_values: fn(T, T) -> Value, apairs: AnnotatedPairs<'a>) -> Result<Value, Error>
where T: FromStr, PackingError<'a>: From<T::Err>
{
    match get_rules_arr_from_pairs(apairs.pairs)? {
        [ Some(RuleData { rule: rl, inner: AnnotatedPairs { providence: provl @ Providence { src: lower_bound, .. }, .. } })
        , Some(RuleData { rule: ru, inner: AnnotatedPairs { providence: provr @ Providence { src: upper_bound, .. }, .. } })
        ] if rl == rule && ru == rule =>
        {
            let lower_bound = lower_bound.parse::<T>()
                .map_err(|err| make_error_from_providence(provl, PackingError::from(err)))?;
            let upper_bound = upper_bound.parse::<T>()
                .map_err(|err| make_error_from_providence(provr, PackingError::from(err)))?;

            Ok(make_values(lower_bound, upper_bound))
        }

        [ Some(rule_data), tail @ .. ] =>
            make_no_match_found_error_many(rule_data, tail)?,

        [ .. ] =>
            Err(make_error_from_providence(apairs.providence, PackingError::ASTPackerEmptyInner))?,
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

pub fn into_array_varied_length<'a, const N: usize>(pairs: impl Iterator<Item = Pair<'a, Rule>>) -> Result<[Option<Pair<'a, Rule>>; N], Error<'a>> {
    pairs.map(Some)
        .chain(std::iter::repeat(None))
        .take(N)
        .collect::<Vec<Option<Pair<'a, Rule>>>>()
        .try_into()
        .map_err(|rules: Vec<Option<Pair<'a, Rule>>>| { // TODO fix this
            let pair = rules.first().unwrap().as_ref().unwrap().clone();

            make_error_from_pair(&pair, PackingError::ArrayCastError(rules))
        })
}

pub fn get_rules_arr_from_pairs<const N: usize>(pairs: Pairs<'_, Rule>) -> Result<[Option<RuleData>; N], Error> {
    let rule_datas = into_array_varied_length(pairs.filter(is_not_tab))?
        .map(|option| option.map(RuleData::from));

    Ok(rule_datas)
}
