use super::parser::Rule;


pub enum Error {
    ParsingError(pest::error::Error<Rule>),
}

impl From<pest::error::Error<Rule>> for Error {
    fn from(value: pest::error::Error<Rule>) -> Self {
        Error::ParsingError(value)
    }
}
