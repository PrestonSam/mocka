use std::collections::HashMap;
use std::fmt::Debug;

use chrono::NaiveDate;
use indexmap::IndexMap;
use serde::{Serialize, Serializer};

use crate::mockagen::model::MockagenError;
use crate::mockagen::packer::model::{NestedAssignNode, TerminalAssignNode, WeightedValue};
use crate::utils::error::LanguageError;

#[derive(Debug)]
pub enum EvaluationError {
    MissingIdentifier(String),
    MissingIdentifiers(Vec<String>),
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

pub type ValueContext<'a>
    = IndexMap<&'a str, OutValue>;

pub type Generator
    = Box<dyn Fn(&ValueContext) -> Result<OutValue, MockagenError>>;

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
struct SelectedDefGen<'a> {
    is_dependency: bool,
    def_gen: &'a DefGen,
}

#[derive(Debug)]
pub struct ColumnGenerator<'a> {
    gens: Vec<SelectedDefGen<'a>>
}

impl<'a> ColumnGenerator<'a> {
    pub fn generate_column(&self) -> Result<Vec<OutValue>, MockagenError> {
        self.gens.iter()
            // TODO instead of producing a vector of booleans, produce a vector of the values you plan to return. This will save you a lot of time
            .fold(Ok((ValueContext::new(), vec![])), |idx_map_rslt, SelectedDefGen { is_dependency, def_gen }| {
                let (mut idx_map, mut dependency_tracker) = idx_map_rslt?;

                let out_val = (def_gen.gen)(&idx_map)?;

                idx_map.insert(&def_gen.id, out_val);
                dependency_tracker.push(*is_dependency);

                Ok((idx_map, dependency_tracker))
            })
            .map(|(idx_map, dep_tracker)|
                idx_map.into_values()
                    .zip(dep_tracker)
                    .filter(|(_, is_dep)| !(is_dep))
                    .map(|(gen, _)| gen)
                    .collect()
            )
    }  
}

#[derive(Debug)]
pub struct GeneratorSet {
    def_gens: HashMap<String, DefGen>,
}

impl GeneratorSet {
    pub fn new(def_gens: Vec<DefGen>) -> Self {
        let map = def_gens.into_iter()
            .map(|def_gen| (def_gen.id.clone(), def_gen))
            .collect::<HashMap<_, _>>();

        GeneratorSet { def_gens: map }
    }

    pub fn merge(&mut self, other: GeneratorSet) {
        self.def_gens.extend(other.def_gens.into_iter())
    }


    pub fn make_column_generator<'a>(&'a self, ids: Vec<String>) -> Result<ColumnGenerator<'a>, MockagenError> {
        type GenResult<'a>
            = Result<Vec<SelectedDefGen<'a>>, Vec<String>>;

        fn select_gens<'a, 'b>(gen_set: &'a GeneratorSet, is_dependency: bool, gen_result: GenResult<'a>, id: &'b str) -> GenResult<'a> {
            match gen_result {
                Ok(mut selected_gens) => 
                    match gen_set.def_gens.get(id) {
                        Some(gen) => {
                            let selected_gen = SelectedDefGen { is_dependency, def_gen: gen };
                            selected_gens.push(selected_gen);

                            gen.dependencies.iter()
                                .fold(
                                    Ok(selected_gens),
                                    |folded_gens, id| select_gens(gen_set, true, folded_gens, id)
                                )
                        },

                        None =>
                            Err(vec![ id.to_string() ]),
                    },

                Err(mut errored_gens) =>
                    if gen_set.def_gens.contains_key(id) {
                        Err(errored_gens)
                    } else {
                        errored_gens.push(id.to_string());
                        Err(errored_gens)
                    }
            }
        }

        ids.iter()
            .fold(Ok(vec![]), |folded_gens, id| select_gens(self, false, folded_gens, id))
            .map(|selected_gens| ColumnGenerator { gens: selected_gens })
            .map_err(EvaluationError::MissingIdentifiers)
            .map_err(MockagenError::from_eval_err)
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
