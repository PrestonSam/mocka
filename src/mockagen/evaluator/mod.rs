use crate::mockagen::{evaluator::{evaluator::Evaluate, model::Bindings}, packer::packer::Body, MockagenError};

mod generators;
mod evaluator;
pub mod model;
pub use generators::Generator2;

pub fn evaluate_mockagen(body: Body) -> Result<Bindings, MockagenError> {
    let Body(maybe_includes, definitions, _) = body;

    // maybe_includes.map(|m| m.0.iter().map(|m| m.))
    // TODO handle includes

    definitions.into_iter()
        .try_fold(Bindings::new(), |bindings, def| def.evaluate(bindings))
        .map_err(MockagenError::from)
}
