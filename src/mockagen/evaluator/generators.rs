use std::collections::HashMap;

use chrono::{Duration, NaiveDate};
use once_cell::sync::Lazy;
use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng, Rng,
};

use crate::mockagen::{
    evaluator::model::EvaluationError,
    model::{DefNode, Definition, Error, MatchExpr, Value, WeightedValue},
};

use super::model::OutValue;

type Generator = Box<dyn Fn() -> OutValue>;

fn make_date_range_gen(low: NaiveDate, high: NaiveDate) -> Generator {
    let range_size = high.signed_duration_since(low).num_days();

    Box::new(move || {
        OutValue::NaiveDate(low + Duration::days(thread_rng().gen_range(0..=range_size)))
    })
}

fn make_integer_range_gen(low: i64, high: i64) -> Generator {
    Box::new(move || OutValue::I64(thread_rng().gen_range(low..=high)))
}

fn make_real_range_gen(low: f64, high: f64) -> Generator {
    Box::new(move || OutValue::F64(thread_rng().gen_range(low..=high)))
}

fn make_string_range_gen(low: i64, high: i64) -> Generator {
    Box::new(move || {
        let length = thread_rng().gen_range(low..=high) as usize;

        OutValue::String(Alphanumeric.sample_string(&mut thread_rng(), length))
    })
}

fn make_literal_gen(literal: String) -> Generator {
    Box::new(move || OutValue::String(literal.clone()))
}

fn make_identifier_gen<'a>(
    identifier: String,
    context: &'a HashMap<String, &Generator>,
) -> Box<dyn Fn() -> Result<OutValue, Error<'a>> + 'a> {
    // Make a lazy pointer to the value in the context
    let lazy = Lazy::new(move || context.get(&identifier).ok_or_else(|| identifier));

    Box::new(move || match lazy.as_deref() {
        Ok(fun) => Ok(fun()),
        Err(identifier) => Err(Error::from(EvaluationError::MissingIdentifier(
            identifier.clone(),
        ))),
    })
}

fn get_generator_from_value(value: Value) {
    let generator = match value {
        Value::DateRange(low, high) => make_date_range_gen(low, high),
        Value::IntegerRange(low, high) => make_integer_range_gen(low, high),
        Value::RealRange(low, high) => make_real_range_gen(low, high),
        Value::StringRange(low, high) => make_string_range_gen(low, high),
        Value::Literal(literal) => make_literal_gen(literal),
        Value::Identifier(identifier) => make_literal_gen(identifier),
        _ => panic!("SEE FIXMES UNDER THIS LINE"),
        // FIXME | 'any' is a matcher, not an assigner. I shouldn't be asked to handle it here
        // FIXME | 'join' is a higher order assigner and also shouldn't be handled here
    };
}

// Presumably we could just create an enum that wraps all the possible return values.
// Then we implement the ToStr trait for that.
// Then we can just return errors if you can't unpack the thing that you're looking for.
// It's definitely a lot better than returning a string that you're expected to parse again.
// And it'd be much faster and would have better quality error messages.
// Alright we'll do that, then

fn get_values_and_weightings(wvals: Vec<WeightedValue>) -> Vec<(f64, Value)> {
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

            (weight.unwrap_or(implicit_weighting), value)
        })
        .collect()
}

fn make_weighted_alternation_gen(wvals: Vec<WeightedValue>) -> Generator {
    let vals_and_weightings = get_values_and_weightings(wvals);

    Box::new(move || {
        let mut chosen_value = thread_rng().gen_range(0.0..=100.0);

        let maybe_value = vals_and_weightings
            .iter()
            .find_map(|(weighting, value)| {
                chosen_value -= weighting;

                Option::Some(value)
                    .filter(|_| 0.0 < chosen_value)
            })
            .expect("Random number generator managed to generate number outside of percentage range");

        maybe_value;

        todo!()
    })
}

fn make_join_gen(values: Vec<Value>) -> impl Fn() -> String {
    // You'd need to figure out what the gen functions are for these values.
    // You'd then need to generate each sub-value and then cat all their results together
    || todo!()
}

fn make_gender_gen() -> Generator {
    make_weighted_alternation_gen(vec![
        WeightedValue {
            weight: Some(30.0),
            value: Value::Literal(String::from("Male")),
        },
        WeightedValue {
            weight: Some(30.0),
            value: Value::Literal(String::from("Female")),
        },
        WeightedValue {
            weight: None,
            value: Value::Literal(String::from("Business Entity")),
        },
    ])
}

fn make_age_gen() -> Generator {
    make_integer_range_gen(18, 90)
}

fn make_payment_channel_gen(fsp_type: OutValue) -> Generator {
    let gen_1 = make_weighted_alternation_gen(vec![
        WeightedValue {
            weight: Some(23.0),
            value: Value::Literal(String::from("ATM")),
        },
        WeightedValue {
            weight: Some(13.0),
            value: Value::Literal(String::from("POS machine")),
        },
        WeightedValue {
            weight: Some(9.0),
            value: Value::Literal(String::from("Mobile banking")),
        },
        WeightedValue {
            weight: Some(20.0),
            value: Value::Literal(String::from("Internet banking")),
        },
        WeightedValue {
            weight: Some(13.0),
            value: Value::Literal(String::from("Branch")),
        },
        WeightedValue {
            weight: Some(17.0),
            value: Value::Literal(String::from("Agent")),
        },
        WeightedValue {
            weight: None,
            value: Value::Literal(String::from("Sub-branch"))
        },
    ]);

    let gen_2 = make_weighted_alternation_gen(vec![
        WeightedValue {
            weight: Some(6.0),
            value: Value::Literal(String::from("Mobile banking")),
        },
        WeightedValue {
            weight: Some(7.0),
            value: Value::Literal(String::from("Internet banking")),
        },
        WeightedValue {
            weight: None,
            value: Value::Literal(String::from("Branch")),
        },
        WeightedValue {
            weight: Some(23.0),
            value: Value::Literal(String::from("Agent")),
        }
    ]);

    match fsp_type {
        OutValue::String(str) =>
            match str.as_str() {
                "Commercial bank" => gen_1,
                "Microfinance institution" => gen_2,

                _ => todo!(),
            }
        _ => todo!(),
    }
    
}

fn get_debug_value() -> Definition {
    Definition::NestedDefinition {
        using_ids: Some(vec![ "country".to_owned() ]),
        identifiers: vec![ "region".to_owned() ],
        branches: vec![
            DefNode::Match {
                matchers: vec![ MatchExpr::Literal("United Kingdom".to_owned()) ],
                children: Some(
                    vec![
                        DefNode::Assign {
                            weight: None,
                            values: vec![
                                WeightedValue {
                                    weight: Some(17.0),
                                    value: Value::Literal("London".to_owned()),
                                },
                                WeightedValue {
                                    weight: Some(10.0),
                                    value: Value::Literal("Manchester".to_owned()),
                                },
                            ],
                            children: None,
                        },
                    ],
                ),
            },
            DefNode::Match {
                matchers: vec![ MatchExpr::Any ],
                children: Some(
                    vec![
                        DefNode::Assign {
                            weight: None,
                            values: vec![
                                WeightedValue {
                                    weight: None,
                                    value: Value::Literal("Unknown".to_owned()),
                                },
                            ],
                            children: None,
                        }
                    ]
                )
            }
        ]
    }
}
