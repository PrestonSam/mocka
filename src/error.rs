#[derive(Debug)]
pub enum Error {
    MockagenError(crate::mockagen::MockagenError)
}
