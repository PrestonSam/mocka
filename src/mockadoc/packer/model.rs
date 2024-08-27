use crate::{mockadoc::parser::Rule, utils::{iterator::TransposeError, packing::SkipRules}};

impl SkipRules for Rule {
    type Rule = Rule;

    fn get_skip_rules(&self) -> Vec<Rule> {
        vec![ Rule::column_divider ]
    }
}

#[derive(Debug)]
pub enum MetadataProperty {
    PrimaryTimestamp,
    Personal,
}

#[derive(Debug)]
pub struct MetadataProperties(pub Vec<MetadataProperty>);

#[derive(Debug)]
pub struct MockagenIdAndMeta(pub String, pub MetadataProperties);

#[derive(Debug)]
pub enum CellData {
    MockagenIdAndMeta(MockagenIdAndMeta),
    Text(String),
}

#[derive(Debug, Clone)]
pub struct ColumnHeading(pub String);

#[derive(Debug)]
pub enum Column {
    MockagenIdAndMeta { title: String, data: Vec<MockagenIdAndMeta> },
    Text { title: String, data: Vec<String> },
}

#[derive(Debug)]
pub struct Document {
    pub title: String,
    pub columns: Vec<Column>, // I think this should have "the" generator column.
}

#[derive(Debug)]
pub struct ImportStatement(pub Vec<String>);

#[derive(Debug)]
pub struct MockadocFile {
    pub import_statement: ImportStatement,
    pub documents: Vec<Document>,
}


#[derive(Debug)]
pub enum PackingErrorVariant {
    SyntaxUnhandledTreeShape(String),
    SyntaxChildrenArrayCastError(Vec<Option<(Rule, String, Option<String>)>>), // TODO This could probably use a type alias
    SyntaxNodeCountMismatch(Vec<Option<(Rule, String, Option<String>)>>), // TODO This could probably use a type alias
    InconsistentColumnTypes { column_number: usize, row: usize },
    TableHasNoRows { column_heading: String },
    InconsistentTableRowWidths(TransposeError)
}

pub type PackingError =
    crate::utils::packing::PackingError<PackingErrorVariant, Rule>;

impl From<TransposeError> for PackingError {
    fn from(value: TransposeError) -> Self {
        PackingError::new(PackingErrorVariant::InconsistentTableRowWidths(value))
    }
}

pub type SyntaxToken<'a> =
    crate::utils::packing::SyntaxToken<'a, Rule>;

pub type SyntaxTree<'a> =
    crate::utils::packing::SyntaxTree<'a, Rule>;

pub type SyntaxChildren<'a> =
    crate::utils::packing::SyntaxChildren<'a, Rule>;


pub trait PackingResult {
    fn with_rule(self, rule: Rule) -> Self;
}

impl<T> PackingResult for Result<T, PackingError> {
    fn with_rule(self, rule: Rule) -> Self
    {
        self.map_err(|err| err.with_rule(rule))
    }
}
