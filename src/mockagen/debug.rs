use std::fs::File;

use csv::WriterBuilder;

use super::{evaluator::model::GeneratorSet, MockagenError};


pub fn write_tsv(gen_set: &GeneratorSet, line_count: usize) -> Result<(), MockagenError> {
    let file = File::create("debug.tsv").unwrap();
    let mut wtr = WriterBuilder::new().delimiter(b'\t').from_writer(file);

    for _ in 0..line_count {
        wtr.serialize(gen_set.generate_row()?).unwrap();
    }

    Ok(())
}
