use std::collections::HashMap;

use chrono::NaiveDate;

use crate::mockagen::model::Error;

#[derive(Debug)]
pub enum EvaluationError {
    MissingIdentifier(String),
}

#[derive(Clone)]
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

pub type ValueContext = HashMap<String, OutValue>;

pub type Generator = Box<dyn Fn(&ValueContext) -> Result<OutValue, Error>>;

// TODO figure out how passing parameters might work
pub struct DefGen {
    pub id: String,
    pub gen: Generator,
}
