use std::rc::Rc;

use itertools::Itertools;

use crate::mockagen::{evaluator::{generators::{GeneratorEnum, NestedGenerator, ValueTree}, model::{Bindings, Result}}, packer::packer::{AssignIds, Definition, Identifier, MultiValDef, Names, NestedDefinition, SingleDefinition, SingleValDef}};

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
    fn evaluate(self, mut bindings: Bindings) -> Result<Bindings> {
        let Self(maybe_using_ids, AssignIds(Names(assign_ids)), nested_clauses) = self;
        let ids = maybe_using_ids.iter()
            .flat_map(|u| u.0.0.iter())
            .chain(assign_ids.iter())
            .map(|i| i.0.as_str())
            .collect_vec();

        let value_tree = Rc::new(ValueTree::from_nested_clauses(ids.as_slice(), nested_clauses)?);
        let using_id_offset = maybe_using_ids.map(|u| u.0.0.len()).unwrap_or(0);
        let depths = using_id_offset..assign_ids.len() + using_id_offset;

        for (Identifier(id), depth) in assign_ids.into_iter().zip(depths) {
            bindings.add(id, GeneratorEnum::Nested(NestedGenerator::new(depth, value_tree.clone())))?;
        }

        Ok(bindings)
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
