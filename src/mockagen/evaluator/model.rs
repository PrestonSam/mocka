use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

use chrono::NaiveDate;
use serde::{Serialize, Serializer};
use thiserror::Error;

use crate::mockagen::evaluator::generators::{Generator2, GeneratorEnum};
use crate::mockagen::packer::packer::{Value, WeightedValue};

#[derive(Error, Debug)]
pub enum EvaluationError {
    #[error("duplicate identifier")]
    DuplicateIdentifier(String),

    #[error("unbound identifier")]
    UnboundIdentifier(String),
}

pub type Result<T> = std::result::Result<T, EvaluationError>;


#[derive(Default)]
pub struct Bindings(HashMap<String, Rc<GeneratorEnum>>);

impl Bindings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, id: String, gen: GeneratorEnum) -> Result<Rc<GeneratorEnum>> {
        let value = Rc::new(gen);

        if !self.0.contains_key(&id) {
            self.0.insert(id, value.clone()).unwrap();
            Ok(value)
        } else {
            Err(EvaluationError::DuplicateIdentifier(id))
        }
    }

    pub fn get(&self, id: &str) -> Result<Rc<GeneratorEnum>> {
        self.0.get(id)
            .map(Rc::to_owned)
            .ok_or_else(|| EvaluationError::UnboundIdentifier(id.to_owned()))
    }
}

#[derive(Default)]
pub struct Scope(HashMap<String, Rc<OutValue>>);

impl Scope {
    fn get_value(&self, id: &str) -> Option<Rc<OutValue>> {
        self.0.get(id).map(Rc::to_owned)
    }

    fn set_value(&mut self, id: &str, value: OutValue) -> Result<Rc<OutValue>> {
        let value = Rc::new(value);

        match self.0.insert(id.to_owned(), value.clone()) {
            Some(_) => Err(EvaluationError::DuplicateIdentifier(id.to_owned())),
            None => Ok(value),
        }
    }
}

// TODO I suspect that Bindings should be AsRef instead of owned.
// Actually better idea, let's assemble context from Bindings, then dismantle it into Bindings later
pub struct Context(Bindings, Scope);

impl Context {
    fn new(bindings: Bindings) -> Self {
        Self(bindings, Default::default())
    }

    pub fn bindings(&self) -> &Bindings {
        &self.0
    }

    pub fn get_value(&mut self, id: &str) -> Result<Rc<OutValue>> {
        match self.1.get_value(id) {
            Some(scoped_value) => Ok(scoped_value),
            None => {
                let binding = self.0.get(id)?;
                let value = binding.generate_value(self)?;

                self.1.set_value(id, value)
            },
        }
    }
}

#[derive(Clone, Debug)]
pub enum OutValue {
    String(String),
    I64(i64),
    F64(f64),
    NaiveDate(NaiveDate),
}

impl std::fmt::Display for OutValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutValue::String(v) => f.write_str(v),
            OutValue::I64(v) => f.write_fmt(format_args!("{v}")),
            OutValue::F64(v) => f.write_fmt(format_args!("{v}")),
            OutValue::NaiveDate(v) => f.write_fmt(format_args!("{}", &v.format("%Y-%m-%d"))),
        }
    }
}


impl Serialize for OutValue {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
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

pub struct MaybeWeighted<T> {
    pub weight: Option<f64>,
    pub value: T
}

impl From<WeightedValue> for MaybeWeighted<Value> {
    fn from(value: WeightedValue) -> Self {
        MaybeWeighted {
            weight: value.0.map(|w| w.get()),
            value: value.1,
        }
    }
}

pub struct WeightedT<T> {
    pub weight: f64,
    pub value: T
}

impl<T> WeightedT<T> {
    pub fn new(maybe_weighted: MaybeWeighted<T>, implicit_weight: f64) -> Self {
        WeightedT {
            weight: maybe_weighted.weight.unwrap_or(implicit_weight),
            value: maybe_weighted.value,
        }
    }
}
