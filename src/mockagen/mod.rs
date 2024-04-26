use self::{evaluator::evaluate_mockagen, model::Error, parser::parse_mockagen, packer::pack_mockagen};

mod model;
mod utils;
mod parser;
mod packer;
mod evaluator;

pub fn run_mockagen(code: &str) -> Result<(), Error> {
    let pairs = parse_mockagen(code)?;
    // dbg!(&pairs);
    // todo!();
    let statements = pack_mockagen(pairs)?;
    let evaluation = evaluate_mockagen(statements);
    dbg!(&evaluation);
    
    Ok(())
}