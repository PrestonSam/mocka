use crate::{
    mockagen::{ColumnGenerator, MockagenError, OutValue},
    utils::iterator::{Transpose, TransposeError}
};


#[derive(Debug)]
pub enum EvaluationError {
    MockagenError(MockagenError),
    FileReadError(std::io::Error),
    DocumentShapeError(TransposeError<OutValue>)
}

impl From<MockagenError> for EvaluationError {
    fn from(value: MockagenError) -> Self {
        EvaluationError::MockagenError(value)
    }
}


pub struct OutRow(pub Vec<OutValue>);

pub struct OutDocument<'a>(pub Vec<ColumnGenerator<'a>>);

impl<'a> OutDocument<'a> {
    pub fn generate(&self) -> Result<Vec<OutRow>, EvaluationError> {
        let output = self.0.iter()
            .map(|col_gen| col_gen.generate_column())
            .collect::<Result<Vec<_>, _>>().map_err(EvaluationError::from)?
            .into_iter()
            .transpose().map_err(EvaluationError::DocumentShapeError)?
            .map(OutRow)
            .collect();

        Ok(output)
    }
}
