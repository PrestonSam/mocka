use mockagen::run_mockagen;

use crate::{mockadoc::run_mockadoc, mockagen::{Context, Generator2}};

mod mockagen;
mod mockadoc;
mod error;
mod utils;

fn mockagen() -> Result<(), crate::error::Error> {
    let file = std::fs::read_to_string("debug_data/debug.mkg").unwrap();
    let output = run_mockagen(&file);

    match output {
        Ok(bindings) => {
            let mut context: Context = bindings.into();
            println!("{:?}", context.get_value("region"));
            println!("{:?}", context.get_value("full-name"));
        },
        Err(err) => println!("{}",err),
    };
    todo!()
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
    let _ = mockagen()
        .inspect_err(|e| println!("{e}"));
    Ok(())

    // mockadoc()
}
