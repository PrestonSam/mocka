use crate::mockagen::model::{PrimitiveValue, WeightedValue};

use super::model::{Definition, Error, Statement};

mod generators;
pub mod model;

pub fn evaluate_mockagen<'a>(statements: Vec<Statement>) -> Result<(), Error<'a>> {
    todo!()
}