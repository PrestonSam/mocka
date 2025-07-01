use crate::{mockagen::{model::PackingError2, parser::Rule}};

// TODO I'd like to make this properly generic but I don't really know how...
pub trait PackingResult {
    fn with_rule(self, rule: Rule) -> Self;
}

impl<T> PackingResult for Result<T, PackingError2> {
    fn with_rule(self, rule: Rule) -> Self {
        self.map_err(|err| err.with_rule(rule))
    }
}
