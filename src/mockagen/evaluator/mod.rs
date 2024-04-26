use self::{generators::make_definition_gens, model::DefGen};

use super::model::Statement;

mod generators;
pub mod model;

pub fn evaluate_mockagen(statements: Vec<Statement>) -> Vec<DefGen> {
    statements.into_iter()
        .flat_map(|statement| {
            match statement {
                Statement::Include(includes) => todo!(),
                Statement::Definition(definition) => {
                    make_definition_gens(definition)
                }
            }
        }).collect()
}
