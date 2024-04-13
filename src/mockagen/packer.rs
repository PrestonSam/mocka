use pest::iterators::{Pair, Pairs};

use crate::mockagen::utils::error::{make_error_from_providence, make_no_match_found_error_many};

use super::{
    model::{AnnotatedPairs, DefNode, Definition, Error, PackingError, Providence, RuleData, Statement, Value, Weight, WeightedValue},
    parser::Rule,
    utils::{error::{make_empty_inner_error, make_no_match_found_error_single}, unpackers::{get_rules_arr_from_pairs, into_array, unpack_range}}
};

fn unpack_string_literal<'a>(pair: Pair<'a, Rule>) -> Result<String, Error<'a>> {
    let [ string_content ] = into_array(pair.into_inner())?;

    Ok(string_content.as_str().to_string())
}

fn parse_value_type<'a>(rule_data: RuleData<'a>) -> Result<Value, Error<'a>> {
    let RuleData { rule, inner: apairs } = rule_data;

    match rule {
        Rule::integer_value =>
            unpack_range(Rule::INTEGER_LITERAL, Value::IntegerRange, apairs),

        Rule::real_value =>
            unpack_range(Rule::REAL_LITERAL, Value::RealRange, apairs),

        Rule::string_value =>
            unpack_range(Rule::INTEGER_LITERAL, Value::StringRange, apairs),

        Rule::timestamp_date_value =>
            unpack_range(Rule::DATE_LITERAL, Value::DateRange, apairs),

        Rule::any_value =>
            Ok(Value::Any),

        Rule::literal_value => {
            let [ string_literal ] = into_array(apairs.pairs)?;
            
            Ok(Value::Literal(unpack_string_literal(string_literal)?))
        },

        Rule::identifier_value =>
            Ok(Value::Identifier(apairs.providence.src.to_string())),

        Rule::join_value =>
            Ok(Value::Join(apairs.pairs
                .map(|pair| parse_value_type(RuleData::from(pair)))
                .collect::<Result<Vec<_>, Error>>()?)
            ),

        rule =>
            Err(make_error_from_providence(apairs.providence, PackingError::NoRuleFound(rule)))
    }
}

fn parse_value_from_inner<'a>(apairs: AnnotatedPairs<'a>) -> Result<Value, Error<'a>> {
    let [ pair ] = into_array(apairs.pairs)?;

    parse_value_type(RuleData::from(pair))
}

fn parse_matchers<'a>(apairs: AnnotatedPairs<'a>) -> Result<Vec<Value>, Error<'a>> {
    match get_rules_arr_from_pairs(apairs.pairs)? {
        [ Some(RuleData { rule: Rule::value, inner, .. }) ] =>
            Ok(vec![ parse_value_from_inner(inner)? ]),

        [ Some(RuleData { rule: Rule::matcher_set, inner, .. }) ] =>
            inner.pairs.map(|pair| parse_value_from_inner(AnnotatedPairs::from(pair))).collect(),

        [ .. ] =>
            make_empty_inner_error(apairs.providence),
    }
}

fn parse_weight<'a>(apairs: AnnotatedPairs<'a>) -> Result<f64, Error<'a>> {
    let [ percentage_pair ] = into_array(apairs.pairs)?;
    let src = percentage_pair.as_str();
    let weight = src.parse::<f64>()
        .map_err(|err| make_error_from_providence(apairs.providence, PackingError::from(err)))?;
    
    Ok(weight)
}

fn parse_maybe_weighted<'a>(apairs: AnnotatedPairs<'a>) -> Result<(Option<Weight>, RuleData<'a>), Error<'a>> {
    match get_rules_arr_from_pairs(apairs.pairs)? {
        [ Some(RuleData { rule: Rule::WEIGHT, inner: weight_pairs, .. })
        , Some(value)
        ] =>
            Ok((Some(parse_weight(weight_pairs)?), value)),

        [ Some(value), None ] =>
            Ok((None, value)),

        [ Some(rule_data), tail @ .. ] =>
            make_no_match_found_error_many(rule_data, tail),

        [ .. ] =>
            make_empty_inner_error(apairs.providence),
    }
}

fn parse_weighted_value<'a>(apairs: AnnotatedPairs<'a>) -> Result<WeightedValue, Error<'a>> {
    let (maybe_weight, value_rule) = parse_maybe_weighted(apairs)?;

    Ok(WeightedValue {
        weight: maybe_weight,
        value: parse_value_from_inner(value_rule.inner)?
    })
}

fn parse_weighted_value_or_set<'a>(rule_data: RuleData<'a>) -> Result<Vec<WeightedValue>, Error<'a>> {
    match rule_data {
        RuleData { rule: Rule::value, inner, .. } =>
            Ok(vec![ WeightedValue { weight: None, value: parse_value_from_inner(inner)? } ]),

        RuleData { rule: Rule::value_set, inner, .. } =>
            inner.pairs
                .map(|pair| parse_weighted_value(AnnotatedPairs::from(pair)))
                .collect::<Result<Vec<WeightedValue>, Error>>(),

        rule_data =>
            make_no_match_found_error_single(rule_data)
    }
}

fn parse_values<'a>(apairs: RuleData<'a>) -> Result<Vec<WeightedValue>, Error<'a>> {
    match apairs {
        RuleData { rule: Rule::values, inner } => {
            let [ pair ] = into_array(inner.pairs)?;

            parse_weighted_value_or_set(RuleData::from(pair))
        },

        rule_data =>
            make_no_match_found_error_single(rule_data)
    }
}

fn parse_single_definition<'a>(apairs: AnnotatedPairs<'a>) -> Result<Definition, Error<'a>> {
    match get_rules_arr_from_pairs(apairs.pairs)? {
        [ Some(RuleData { rule: Rule::IDENTIFIER, inner: AnnotatedPairs { providence: Providence { src: id, .. }, .. } })
        , Some(rule_data)
        ] =>
            Ok(Definition::SingleDefinition { identifier: id.to_string(), values: parse_weighted_value_or_set(rule_data)? }),

        [ Some(rule_data), tail @ .. ] =>
            make_no_match_found_error_many(rule_data, tail),

        [ .. ] =>
            make_empty_inner_error(apairs.providence),
    }
}

fn parse_names<'a>(apairs: AnnotatedPairs<'a>) -> Result<Vec<String>, Error<'a>> {
    apairs.pairs.map(|pair| {
        match RuleData::from(pair) {
            RuleData { rule: Rule::IDENTIFIER, inner: AnnotatedPairs { providence: Providence { src, .. }, .. } } =>
                Ok(src.to_string()),
            
            rule_data =>
                make_no_match_found_error_single(rule_data)
        }
    }).collect()
}

fn parse_children(apairs: AnnotatedPairs) -> Result<Option<Vec<DefNode>>, Error> {
    apairs.pairs
        .peek()
        .map(|_| apairs.pairs.map(parse_branch).collect())
        .transpose()
}

fn parse_branch(pair: Pair<'_, Rule>) -> Result<DefNode, Error> {
    match RuleData::from(pair) {
        RuleData { rule: Rule::r#match, inner } => {
             match get_rules_arr_from_pairs(inner.pairs)? {
                [ Some(RuleData { rule: Rule::matchers, inner: matchers })
                , Some(RuleData { rule: Rule::match_sub_assignments | Rule::match_sub_matches, inner: child_pairs })
                ] =>
                    Ok(DefNode::Match {
                        matchers: parse_matchers(matchers)?,
                        children: parse_children(child_pairs)?
                    }),

                [ Some(rule_data), tail @ .. ] =>
                    make_no_match_found_error_many(rule_data, tail),

                [ .. ] =>
                    make_empty_inner_error(inner.providence),
            }
        }

        RuleData { rule: Rule::assign, inner } => {
            match get_rules_arr_from_pairs(inner.pairs)? {
                [ Some(RuleData { rule: Rule::weighted_values, inner: weighted_value_pairs, .. })
                , Some(RuleData { rule: Rule::sub_assignments, inner: sub_assignments_pairs, .. })
                ] => {
                    let (maybe_weight, values_rule) = parse_maybe_weighted(weighted_value_pairs)?;

                    Ok(DefNode::Assign { 
                        weight: maybe_weight,
                        values: parse_values(values_rule)?,
                        children: parse_children(sub_assignments_pairs)?,
                    })
                }

                [ Some(rule_data), tail @ .. ] =>
                    make_no_match_found_error_many(rule_data, tail),

                [ .. ] =>
                    make_empty_inner_error(inner.providence),
            }
        }
        
        rule_data =>
            make_no_match_found_error_single(rule_data)
    }
}

fn parse_nested_definition<'a>(apairs: AnnotatedPairs<'a>) -> Result<Definition, Error<'a>> {
    let (maybe_using_ids, assign_ids, nested_clauses) = match get_rules_arr_from_pairs(apairs.pairs)? {
        [ Some(RuleData { rule: Rule::using_ids, inner: using_ids, .. })
        , Some(RuleData { rule: Rule::assign_ids, inner: assign_ids, .. })
        , Some(RuleData { rule: Rule::nested_clauses, inner: nested_clauses, .. })
        ] =>
            (Some(parse_names(using_ids)?), assign_ids, nested_clauses),

        [ Some(RuleData { rule: Rule::assign_ids, inner: assign_ids, .. })
        , Some(RuleData { rule: Rule::nested_clauses, inner: nested_clauses, .. })
        , None
        ] =>
            (None, assign_ids, nested_clauses),

        [ Some(rule_data), tail @ .. ] =>
            make_no_match_found_error_many(rule_data, tail)?,

        [ .. ] =>
            make_empty_inner_error(apairs.providence)?,
    };


    let assign_ids = parse_names(assign_ids)?;
    let branches = nested_clauses.pairs
        .map(parse_branch)
        .collect::<Result<Vec<_>, Error>>()?;

    Ok(Definition::NestedDefinition {
        using_ids: maybe_using_ids,
        identifiers: assign_ids,
        branches
    })
}

pub fn pack_mockagen(pairs: Pairs<'_, Rule>) -> Result<Vec<Statement>, Error> {
    pairs.map_while(|pair|
        match RuleData::from(pair) {
            RuleData { rule: Rule::include_statement, inner, .. } =>
                Some(inner.pairs
                    .map(unpack_string_literal)
                    .collect::<Result<Vec<String>, Error>>()
                    .map(Statement::Include)
                ),

            RuleData { rule: Rule::single_definition, inner, ..  } =>
                Some(parse_single_definition(inner).map(Statement::Definition)),

            RuleData { rule: Rule::nested_definition, inner } =>
                Some(parse_nested_definition(inner).map(Statement::Definition)),

            RuleData { rule: Rule::EOI, .. } =>
                None,

            rule_data =>
                Some(make_no_match_found_error_single(rule_data))
        }
    ).collect()
}
