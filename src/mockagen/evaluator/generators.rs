use chrono::{Duration, NaiveDate};
use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng, Rng,
};

use crate::mockagen::{
    evaluator::model::EvaluationError,
    model::{AssignNode, DefSet, Definition, Error, HigherOrderValue, MatchExpr, MatchNode, PrimitiveValue, Value, WeightedValue, WildcardNode},
};

use super::model::{DefGen, Generator, OutValue};


fn make_date_range_gen<'a>(low: NaiveDate, high: NaiveDate) -> Generator {
    let range_size = high.signed_duration_since(low).num_days();

    Box::new(move |_| {
        Ok(OutValue::NaiveDate(low + Duration::days(thread_rng().gen_range(0..=range_size))))
    })
}

fn make_integer_range_gen<'a>(low: i64, high: i64) -> Generator {
    Box::new(move |_| Ok(OutValue::I64(thread_rng().gen_range(low..=high))))
}

fn make_real_range_gen<'a>(low: f64, high: f64) -> Generator {
    Box::new(move |_| Ok(OutValue::F64(thread_rng().gen_range(low..=high))))
}

fn make_string_range_gen<'a>(low: i64, high: i64) -> Generator {
    Box::new(move |_| {
        let length = thread_rng().gen_range(low..=high) as usize;

        Ok(OutValue::String(Alphanumeric.sample_string(&mut thread_rng(), length)))
    })
}

fn make_literal_gen<'a>(literal: String) -> Generator {
    Box::new(move |_| Ok(OutValue::String(literal.clone())))
}

fn make_identifier_gen<'a>(identifier: String) -> Generator {
    Box::new(move |context| match context.get(&identifier) {
        Some(value) =>
            Ok(value()),

        None =>
            Err(Error::from(EvaluationError::MissingIdentifier(identifier.clone()))),
    })
}

fn make_join_gen<'a>(values: Vec<Value>) -> Generator {
    let generators: Vec<Generator> = values.into_iter()
        .map(|value| get_generator_from_value(value))
        .collect();

    Box::new(move |context| {
        let output = generators.iter()
            .map(|gen| Ok::<_, Error>(gen(context)?.to_string()))
            .collect::<Result<_, _>>()?;

        Ok(OutValue::String(output))
    })
}

fn get_generator_from_primitive_value<'a>(value: PrimitiveValue) -> Generator {
    match value {
        PrimitiveValue::DateRange(low, high) => make_date_range_gen(low, high),
        PrimitiveValue::IntegerRange(low, high) => make_integer_range_gen(low, high),
        PrimitiveValue::RealRange(low, high) => make_real_range_gen(low, high),
        PrimitiveValue::StringRange(low, high) => make_string_range_gen(low, high),
        PrimitiveValue::Literal(literal) => make_literal_gen(literal),
    }
}

fn get_generator_from_higher_order_value<'a>(value: HigherOrderValue) -> Generator {
    match value {
        HigherOrderValue::Identifier(identifier) =>
            make_identifier_gen(identifier),

        HigherOrderValue::Join(values) =>
            make_join_gen(values),
    }
}

fn get_generator_from_value<'a>(value: Value) -> Generator {
    match value {
        Value::PrimitiveValue(value) =>
            get_generator_from_primitive_value(value),

        Value::HigherOrderValue(value) =>
            get_generator_from_higher_order_value(value),
    }
}

// Presumably we could just create an enum that wraps all the possible return values.
// Then we implement the ToStr trait for that.
// Then we can just return errors if you can't unpack the thing that you're looking for.
// It's definitely a lot better than returning a string that you're expected to parse again.
// And it'd be much faster and would have better quality error messages.
// Alright we'll do that, then

fn get_gens_and_weightings<'a>(wvals: Vec<WeightedValue>) -> Vec<(f64, Generator)> {
    let mut total_explicit_percentage = 0.0;
    let mut implicit_percentage_count = 0.0;

    for wval in &wvals {
        let WeightedValue { weight, value: _ } = wval;

        match weight {
            Some(weight) => total_explicit_percentage += weight,
            None => implicit_percentage_count += 1.0,
        }
    }

    let implicit_weighting = (100.0 - total_explicit_percentage) / implicit_percentage_count;

    wvals
        .into_iter()
        .map(move |wval| {
            let WeightedValue { weight, value } = wval;

            (weight.unwrap_or(implicit_weighting), get_generator_from_value(value))
        })
        .collect()
}

fn make_weighted_alternation_gen<'a>(wvals: Vec<WeightedValue>) -> Generator {
    let vals_and_weightings = get_gens_and_weightings(wvals);

    Box::new(move |context| {
        let mut chosen_percentage = thread_rng().gen_range(0.0..=100.0);

        let gen = vals_and_weightings
            .iter()
            .find_map(|(weighting, gen)| {
                chosen_percentage -= weighting;

                Option::Some(gen)
                    .filter(|_| 0.0 < chosen_percentage)
            })
            .expect("Random number generator managed to generate number outside of percentage range");

        gen(context)
    })
}

pub fn make_definition_gen<'a>(definition: Definition) -> Vec<DefGen> {
    match definition {
        Definition::NestedDefinition { using_ids, identifiers, def_set } => {
            
            todo!()
        }

        Definition::SingleDefinition { identifier, values } => {
            vec![
                DefGen {
                    id: identifier,
                    gen: make_weighted_alternation_gen(values)
                }
            ]
        }
    }
}

fn get_debug_value() -> Definition {
    Definition::NestedDefinition {
        using_ids: Some(vec![ "country".to_string() ]),
        identifiers: vec![ "region".to_string() ],
        def_set: DefSet::MatchWithWildCard {
            nodes: vec![
                MatchNode {
                    matchers: vec![ MatchExpr::Literal("United Kingdom".to_string()), ],
                    children: DefSet::Assign {
                        nodes: vec![
                            AssignNode {
                                weight: None,
                                values: vec![
                                    WeightedValue {
                                        weight: Some(17.0),
                                        value: Value::PrimitiveValue(PrimitiveValue::Literal("London".to_string())),
                                    },
                                    WeightedValue {
                                        weight: Some(10.0),
                                        value: Value::PrimitiveValue(PrimitiveValue::Literal("Manchester".to_string())),
                                    },
                                ],
                                children: None,
                            },
                        ],
                    },
                },
            ],
            wildcard_node: Box::new(WildcardNode {
                children: DefSet::Assign {
                    nodes: vec![
                        AssignNode {
                            weight: None,
                            values: vec![
                                WeightedValue {
                                    weight: None,
                                    value: Value::PrimitiveValue(PrimitiveValue::Literal("Unknown".to_string())),
                                },
                            ],
                            children: None,
                        },
                    ],
                },
            }),
        },
    }
}
