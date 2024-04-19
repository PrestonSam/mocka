use mockagen::run_mockagen;

mod mockagen;
mod mockadoc;


fn main() {
    let file = std::fs::read_to_string("debug-data/generic-event-types.mkg").unwrap();
    let output = run_mockagen(&file);

    output.map_err(|err| dbg!(err));
}
