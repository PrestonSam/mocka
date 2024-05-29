use crate::mockagen::{debug::write_tsv, evaluator::model::GeneratorSet};

use self::{evaluator::evaluate_mockagen, packer::pack_mockagen, parser::parse_mockagen};

mod model;
mod utils;
mod parser;
mod packer;
mod evaluator;
mod debug;

pub use model::MockagenError;

pub fn run_mockagen(code: &str) -> Result<(), MockagenError> {
    let pairs = parse_mockagen(code)?;
    let statements = pack_mockagen(pairs)?;
    dbg!(&statements);
    let evaluation = evaluate_mockagen(statements);
    dbg!(&evaluation);

    let gen_set = GeneratorSet::new(evaluation);

    write_tsv(&gen_set, 1_000)
}
