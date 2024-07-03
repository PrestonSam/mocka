use crate::mockagen::parser::parse_mockagen;

use self::{evaluator::evaluate_mockagen, packer::pack_mockagen};

mod model;
mod utils;
mod parser;
mod packer;
mod evaluator;

pub use model::MockagenError;
pub use evaluator::model::{GeneratorSet, ColumnGenerator, OutValue};

pub fn run_mockagen(code: &str) -> Result<GeneratorSet, MockagenError> {
    let pairs = parse_mockagen(code)?;
    let statements = pack_mockagen(pairs)?;
    let evaluation = evaluate_mockagen(statements);

    Ok(GeneratorSet::new(evaluation))
}
