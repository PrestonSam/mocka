use std::{cmp::Ordering, fmt::Debug};

use chrono::NaiveDate;
use indexmap::IndexMap;
use itertools::Itertools;

use crate::mockagen::model::Error;

#[derive(Debug)]
pub enum EvaluationError {
    MissingIdentifier(String),
}

#[derive(Clone, Debug)]
pub enum OutValue {
    String(String),
    I64(i64),
    F64(f64),
    NaiveDate(NaiveDate),
}

impl ToString for OutValue {
    fn to_string(&self) -> String {
        match self {
            Self::String(str) => str.to_string(),
            Self::I64(i64) => i64.to_string(),
            Self::F64(f64) => f64.to_string(),
            Self::NaiveDate(date) => date.to_string(), // TODO Might want to check what this one looks like 
        }
    }
}

pub type ValueContext<'a> = IndexMap<&'a str, OutValue>;

pub type Generator = Box<dyn Fn(&ValueContext) -> Result<OutValue, Error>>;

pub struct DefGen {
    pub id: String,
    pub gen: Generator,
    pub dependencies: Vec<String>,
}

impl Debug for DefGen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefGen")
            .field("id", &self.id)
            .field("dependencies", &self.dependencies)
            .finish()
    }
}

#[derive(Debug)]
pub struct GeneratorSet {
    def_gens: Vec<DefGen>,
}

impl GeneratorSet {
    fn new(mut def_gens: Vec<DefGen>) -> GeneratorSet {
        def_gens.sort_by(|l, r|
            if r.dependencies.contains(&l.id) {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        );

        GeneratorSet { def_gens }
    }

    fn generate_row(&self) -> Result<Vec<OutValue>, Error> {
        self.def_gens.iter()
            .fold(Ok(IndexMap::<&str, OutValue>::new()), |idx_map_rslt, def_gen| {
                let mut idx_map = idx_map_rslt?;

                let out_val = (*def_gen.gen)(&idx_map)?;
                idx_map.insert(&def_gen.id, out_val);

                Ok(idx_map)
            })
            .map(|idx_map| idx_map.into_values().collect_vec())
    }  
}
