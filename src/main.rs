use mockagen::run_mockagen;

mod mockagen;
mod mockadoc;
mod error;


fn main() -> Result<(), crate::error::Error> {
    let file = std::fs::read_to_string("debug-data/debug.mkg").unwrap();
    let output = run_mockagen(&file);

    dbg!(&output);

    output.map(|_| ())
        .map_err(|err| crate::error::Error::MockagenError(err))
}
