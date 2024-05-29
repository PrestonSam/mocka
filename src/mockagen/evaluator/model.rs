use std::{cmp::Ordering, fmt::Debug};

use chrono::NaiveDate;
use indexmap::IndexMap;
use itertools::Itertools;
use serde::{Serialize, Serializer};

use crate::mockagen::model::{MockagenError, NestedAssignNode, TerminalAssignNode, WeightedValue};

#[derive(Debug)]
pub enum EvaluationError {
    MissingIdentifier(String),
    NoMatchBranchForValue(OutValue),
}

#[derive(Clone, Debug)]
pub enum OutValue {
    String(String),
    I64(i64),
    F64(f64),
    NaiveDate(NaiveDate),
}

impl Serialize for OutValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        match self {
            OutValue::NaiveDate(date) => serializer.serialize_str(&date.format("%Y-%m-%d").to_string()),
            OutValue::String(str) => serializer.serialize_str(str),
            OutValue::F64(f64) => serializer.serialize_f64(*f64),
            OutValue::I64(i64) => serializer.serialize_i64(*i64),
        }
    }
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

pub type Generator = Box<dyn Fn(&ValueContext) -> Result<OutValue, MockagenError>>;

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
    pub fn new(mut def_gens: Vec<DefGen>) -> Self {
        def_gens.sort_by(|l, r|
            if r.dependencies.contains(&l.id) {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        );

        GeneratorSet { def_gens }
    }

    pub fn only(def_gens: Vec<DefGen>, ids: Vec<String>) -> Self {
        let filtered_ids = def_gens.into_iter()
            .filter(|def_gen| ids.contains(&def_gen.id))
            .collect();

        GeneratorSet::new(filtered_ids)
    }

    pub fn generate_row(&self) -> Result<Vec<OutValue>, MockagenError> {
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

pub struct MaybeWeighted<T> {
    pub weight: Option<f64>,
    pub value: T
}

impl From<WeightedValue> for MaybeWeighted<WeightedValue> {
    fn from(value: WeightedValue) -> Self {
        MaybeWeighted {
            weight: value.weight,
            value,
        }
    }
}

impl From<NestedAssignNode> for MaybeWeighted<NestedAssignNode> {
    fn from(value: NestedAssignNode) -> Self {
        MaybeWeighted {
            weight: value.weight,
            value,
        }
    }
}

impl From<TerminalAssignNode> for MaybeWeighted<TerminalAssignNode> {
    fn from(value: TerminalAssignNode) -> Self {
        MaybeWeighted {
            weight: value.weight,
            value,
        }
    }
}

pub struct WeightedT<T> {
    pub weight: f64,
    pub value: T
}
