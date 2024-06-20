#[derive(Debug)]
pub enum Error {
    MockagenError(crate::mockagen::MockagenError),
    MockadocError(crate::mockadoc::MockadocError),
}
