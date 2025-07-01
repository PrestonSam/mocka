use thiserror::Error;

use crate::{
    mockagen::{MockagenError, OutValue},
    utils::iterator::{LegacyTransposeError, Transpose, TransposeError}
};


#[derive(Error, Debug)]
pub enum EvaluationError {
    #[error("{0}")]
    MockagenError(#[from] MockagenError),

    #[error("mockagen import file read error")]
    FileReadError(#[from] std::io::Error),

    #[error("malformed table")]
    TableShapeError(#[from] TransposeError),

    #[error("legacy")]
    LegacyDocumentShapeError(LegacyTransposeError)
}

pub struct OutRow(pub Vec<OutValue>);

pub struct OutDocument<'a>(pub Vec<ColumnGenerator<'a>>);

impl OutDocument<'_> {
    pub fn generate(&self) -> Result<Vec<OutRow>, EvaluationError> {
        let output = self.0.iter()
            .map(|col_gen| col_gen.generate_column())
            .collect::<Result<Vec<_>, _>>().map_err(EvaluationError::from)?
            .into_iter()
            .transpose().map_err(EvaluationError::TableShapeError)?
            .map(OutRow)
            .collect();

        Ok(output)
    }
}
