use crate::mockagen::model::{Value, WeightedValue};

use super::model::{Definition, Error, Statement};

mod generators;
pub mod model;

pub fn evaluate_mockagen<'a>(statements: Vec<Statement>) -> Result<(), Error<'a>> {
    match statements.first().unwrap() {
        Statement::Definition(definition) => {
            match definition {
                Definition::SingleDefinition { identifier, values } => {
                    dbg!(&identifier);
                    match values.first().unwrap() {
                        WeightedValue { weight: Some(weight), value } => {
                            dbg!(&weight);
                            match value {
                                Value::Literal(literal) => {
                                    dbg!(&literal);
                                    todo!()
                                }
                                Value::IntegerRange(low, high) => {
                                    dbg!(&low, &high);
                                    todo!()
                                }
                                _ => todo!()
                            }
                        }
                        _ => todo!()
                    }
                }
                _ => todo!()
            }
        }
        _ => todo!()
    }
}