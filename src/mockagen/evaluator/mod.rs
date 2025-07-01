use crate::mockagen::{evaluator::{evaluator::{Evaluate}, model::Bindings, model::Result}, packer::packer::Body};

mod generators;
mod evaluator;
pub mod model;

pub fn evaluate_mockagen(body: Body) -> Result<Bindings> {
    let Body(maybe_includes, definitions, _) = body;

    // TODO handle includes

    definitions.into_iter()
        .try_fold(Bindings::new(), |bindings, def| def.evaluate(bindings))
}
