use crate::mockadoc::{evaluator::evaluate_mockadoc, packer::pack_mockadoc, parser::parse_mockadoc};

pub use self::model::MockadocError;

mod model;
mod parser;
mod packer;
mod evaluator;
mod utils;


pub fn run_mockadoc(code: &str) -> Result<(), MockadocError> {
    let pairs = parse_mockadoc(code)?;
    dbg!(&pairs);
    panic!("whoops");
    let packed = pack_mockadoc(pairs)?;
    dbg!(&packed);
    let evaluation = evaluate_mockadoc(packed)?;
    dbg!(&evaluation);

    todo!()
}   
