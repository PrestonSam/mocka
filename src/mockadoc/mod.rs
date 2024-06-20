use crate::mockadoc::{packer::pack_mockadoc, parser::parse_mockadoc};

pub use self::model::MockadocError;

mod model;
mod parser;
mod packer;
mod evaluator;
mod utils;


pub fn run_mockadoc(code: &str) -> Result<(), MockadocError> {
    let pairs = parse_mockadoc(code)?;
    let packed = pack_mockadoc(pairs);
    dbg!(packed);
    // let evaluation = evaluate_mockadoc(statements);
    // dbg!(&evaluation);

    todo!()
}   
