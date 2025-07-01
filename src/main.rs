use mockadoc::MockadocError;
use mockagen::{run_mockagen};

use crate::mockadoc::run_mockadoc;

mod mockagen;
mod mockadoc;
mod error;
mod utils;

fn mockagen() -> Result<(), crate::error::Error> {
    let file = std::fs::read_to_string("debug_data/debug.mkg").unwrap();
    let output = run_mockagen(&file);

    dbg!(&output);

    output.map_err(crate::error::Error::MockagenError)
}


fn mockadoc() -> Result<(), crate::error::Error> {
    let file = std::fs::read_to_string("debug_data/new-dbg.mkd").unwrap();
    let output = run_mockadoc(&file);

    match output {
        Ok(()) => { println!("it worked"); }
        Err(err) => {
            println!("{err}");
            // dbg!(err);
        }
    }

    todo!()
}


fn main() -> Result<(), crate::error::Error> {
    // mockagen();

    mockadoc()
}
