use std::collections::HashMap;

use self::{generators::make_definition_gen, model::ValueContext};

use super::model::{Error, Statement};

mod generators;
pub mod model;

pub fn evaluate_mockagen<'a>(statements: Vec<Statement>) -> Result<(), Error> {
    let def_gens: Vec<_> = statements.into_iter().flat_map(|statement| {
        match statement {
            Statement::Include(includes) => todo!(),
            Statement::Definition(definition) => {
                make_definition_gen(definition)
            }
        }
    }).collect();

    // TODO you should have lazy functions that have not yet evaluated their values. A value is evaluated once per row
    let mut context: ValueContext = HashMap::new();
    

    todo!()
}