use thiserror::Error;

use crate::{
    mockagen::{MockagenError, OutValue},
    utils::iterator::TransposeError
};


#[derive(Error, Debug)]
pub enum EvaluationError {
    #[error("{0}")]
    MockagenError(#[from] MockagenError),

    #[error("mockagen import file read error")]
    FileReadError(#[from] std::io::Error),

    #[error("malformed table")]
    TableShapeError(#[from] TransposeError),
}

// pub struct OutRow(pub Vec<OutValue>);

// pub struct OutDocument(pub Vec<()>);

// impl OutDocument<'_> {
//     pub fn generate(&self) -> Result<Vec<OutRow>, EvaluationError> {
//         let output = self.0.iter()
//             .map(|col_gen| col_gen.generate_column())
//             .collect::<Result<Vec<_>, _>>().map_err(EvaluationError::from)?
//             .into_iter()
//             .transpose().map_err(EvaluationError::TableShapeError)?
//             .map(OutRow)
//             .collect();

//         Ok(output)
//     }
// }
