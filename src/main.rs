use mockagen::{run_mockagen, GeneratorSet};

use crate::mockadoc::run_mockadoc;

mod mockagen;
mod mockadoc;
mod error;
mod utils;

fn mockagen() -> Result<GeneratorSet, crate::error::Error> {
    let file = std::fs::read_to_string("debug-data/debug.mkg").unwrap();
    let output = run_mockagen(&file);


    output.map_err(|err| crate::error::Error::MockagenError(err))
}


fn mockadoc() -> Result<(), crate::error::Error> {
    let file = std::fs::read_to_string("debug-data/debug.mkd").unwrap();
    let output = run_mockadoc(&file);

    dbg!(output);

    todo!()
}


fn main() -> Result<(), crate::error::Error> {
    // mockagen();
    mockadoc()
}
