use chrono::NaiveDate;

#[derive(Debug)]
pub enum EvaluationError {
    MissingIdentifier(String),
}

pub enum OutValue {
    String(String),
    I64(i64),
    F64(f64),
    NaiveDate(NaiveDate),
}
