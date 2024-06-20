use crate::mockadoc::{packer::model::PackingError, parser::Rule};

pub trait PackingResult {
    fn with_rule(self, rule: Rule) -> Self;
}

impl<T> PackingResult for Result<T, PackingError> {
    fn with_rule(self, rule: Rule) -> Self
    {
        self.map_err(|err| err.with_rule(rule))
    }
}
