use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    MockagenError(#[from] crate::mockagen::MockagenError),

    #[error("{0}")]
    MockadocError(#[from] crate::mockadoc::MockadocError),
}
