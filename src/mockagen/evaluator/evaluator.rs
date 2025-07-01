use itertools::Itertools;

use crate::mockagen::{evaluator::model::{Bindings, Result}, packer::packer::{AssignIds, Definition, Identifier, MultiValDef, Names, NestedClauses, NestedDefinition, SingleDefinition, SingleValDef}};

pub trait Evaluate {
    // Worth exchanging the returned vec for an iterator and then just call flatten a lot.
    // Actually, I think I might just modify &mut Bindings and return nothing
    // Perhaps I'll just return a result to capture any errors?
    fn evaluate(self, bindings: Bindings) -> Result<Bindings>;
}

impl Evaluate for SingleValDef {
    fn evaluate(self, mut bindings: Bindings) -> Result<Bindings> {
        let Self(Identifier(id), value) = self;
        bindings.add(id, value.into())?;
        Ok(bindings)
    }
}

impl Evaluate for MultiValDef {
    fn evaluate(self, mut bindings: Bindings) -> Result<Bindings> {
        let Self(Identifier(id), _, values) = self;
        bindings.add(id, values.into())?;
        Ok(bindings)
    }
}

impl Evaluate for SingleDefinition {
    fn evaluate(self, bindings: Bindings) -> Result<Bindings> {
        match self {
            SingleDefinition::SingleVal(single) => single.evaluate(bindings),
            SingleDefinition::MultiVal(multi) => multi.evaluate(bindings),
        }
    }
}

impl Evaluate for NestedDefinition {
    fn evaluate(self, bindings: Bindings) -> Result<Bindings> {
        let Self(maybe_using_ids, AssignIds(Names(assign_ids)), nested_clauses) = self;
        // This can be broken into one assign statement (which would look similar to multi.evaluate) followed by lots of using context define x statements
        // So let's pair everything up first

        // Using A, B, C Define D E F
        // Becomes

        // Using A, B, C, Define D
        // Using A, B, C, D Define E
        // Using A, B, C, D, E Define F

        // So I'm seeing lots of slices right?
        // We're ALWAYS including the using ids, so that's easy. I don't need to consider those too hard
        // I should be iterating along the assign ids and taking x values, then consuming the next value after that
        // Leading up to the final value.
        // Considering that I'm incrementing each time, I wonder if it's possible to ignore indexes
        // I think instead it's probably worth just using a range based on the length of the vector.
        // (..assign_ids.len()).map(|len| maybe_using_ids.iter().flatten().chain(assign_ids.take(len)))

        // Every item in this iterator produces an iterator containing a definition and all of its dependencies
        let statements = (0..assign_ids.len())
            .map(|len| maybe_using_ids.iter().flat_map(|i| i.0.0.iter()).chain(assign_ids.iter().take(len)));

        maybe_using_ids.unwrap();
        todo!()
    }
}

impl Evaluate for Definition {
    fn evaluate(self, bindings: Bindings) -> Result<Bindings> {
        match self {
            Definition::Single(single) => single.evaluate(bindings),
            Definition::Nested(nested) => nested.evaluate(bindings),
        }
    }
}
