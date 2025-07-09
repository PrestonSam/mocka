use packer::pack;

use crate::mockadoc::{parser::parse_mockadoc};

pub use self::model::MockadocError;

mod model;
mod parser;
mod packer;
mod evaluator;
mod utils;


pub fn run_mockadoc(code: &str) -> Result<(), MockadocError> {
    let pairs = parse_mockadoc(code)?;
    // dbg!(&pairs);
    let packed = pack(pairs).map_err(MockadocError::PackingError2)?;
    // dbg!(&packed);
    // let evaluation = evaluate_mockadoc(packed)?;
    // dbg!(&evaluation);

    todo!()
}   
