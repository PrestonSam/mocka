use crate::mockagen::parser::parse_mockagen2;

use self::{evaluator::evaluate_mockagen, packer::pack_mockagen};

mod model;
mod utils;
mod parser;
mod packer;
mod evaluator;

pub use model::MockagenError;
pub use evaluator::model::OutValue;

pub fn run_mockagen(code: &str) -> Result<(), MockagenError> {
    let pairs = parse_mockagen2(code)?;
    let body = pack_mockagen(pairs)?;

    let evaluation = evaluate_mockagen(body);

    // Ok(GeneratorSet::new(evaluation))
    todo!("switched this off to get a compilation while transitioning to new parser and packer")
}
