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
pub enum CellData {
    MetadataProperties(MetadataProperties),
    MockagenId(String),
    Text(String),
}

#[derive(Debug, Clone)]
pub enum ColumnHeading {
    MetadataTag,
    GeneratorTag,
    Text(String),
}

#[derive(Debug)]
pub enum Column {
    Metadata(Vec<MetadataProperties>),
    Generators(Vec<String>),
    Text { title: String, data: Vec<String> },
}

#[derive(Debug)]
pub struct Document {
    pub title: String,
    pub columns: Vec<Column>,
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
    InconsistentColumnTypes { heading: ColumnHeading, cell: CellData, row: usize },
    TableHasNoRows { column_heading: String },
    InconsistentTableRowWidths(TransposeError<CellData>)
}

pub type PackingError =
    crate::utils::packing::PackingError<PackingErrorVariant, Rule>;

impl From<TransposeError<CellData>> for PackingError {
    fn from(value: TransposeError<CellData>) -> Self {
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
