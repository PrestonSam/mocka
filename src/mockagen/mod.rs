use crate::mockagen::{evaluator::model::Bindings, parser::parse_mockagen};

use self::{evaluator::evaluate_mockagen, packer::pack_mockagen};

mod model;
mod utils;
mod parser;
mod packer;
mod evaluator;

pub use model::MockagenError;
pub use evaluator::model::OutValue;
pub use evaluator::Generator2;
pub use evaluator::model::Context;

pub fn run_mockagen(code: &str) -> Result<Bindings, MockagenError> {
    parse_mockagen(code)
        .and_then(pack_mockagen)
        .and_then(evaluate_mockagen)
}
