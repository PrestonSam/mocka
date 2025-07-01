#![allow(dead_code)]

use thiserror::Error;

use crate::{mockadoc::{packer::MockagenIdAndMetadata, parser::Rule}, utils::{iterator::LegacyTransposeError, packing::SkipRules}};

impl SkipRules for Rule {
    type Rule = Rule;

    fn get_skip_rules(&self) -> Vec<Rule> {
        vec![ Rule::column_divider ]
    }
}

// #[derive(Debug)]
// pub enum MetadataProperty {
//     PrimaryTimestamp,
//     Personal,
// }

// #[derive(Debug)]
// pub struct MetadataProperties(pub Vec<MetadataProperty>);

// #[derive(Debug)]
// pub struct MockagenIdAndMeta(pub String, pub MetadataProperties);

// #[derive(Debug)]
// pub enum CellData {
//     MockagenIdAndMeta(MockagenIdAndMeta),
//     Text(String),
// }

#[derive(Debug, Clone)]
pub struct ColumnHeading(pub String);

#[derive(Debug)]
pub enum ColumnData {
    Text(Vec<String>),
    MockagenIdAndMetadata(Vec<MockagenIdAndMetadata>),
}

// impl ColumnData {
//     pub fn new(cell_data: CellData) -> Self {
//         match cell_data {
//             CellData::MockagenIdAndMeta(id_and_metadata) =>
//                 Self::MockagenIdAndMetadata(vec![ id_and_metadata ]),

//             CellData::Text(text) =>
//                 Self::Text(vec![ text ]),
//         }
//     }

//     pub fn append(self, cell_data: CellData) -> Option<Self> {
//         match (self, cell_data) {
//             (ColumnData::MockagenIdAndMetadata(mut ids_and_metadatas)
//             , CellData::MockagenIdAndMeta(id_and_metadata)
//             ) => {
//                 ids_and_metadatas.push(id_and_metadata);
//                 Some(ColumnData::MockagenIdAndMetadata(ids_and_metadatas))
//             }

//             (ColumnData::Text(mut texts)
//             , CellData::Text(text)
//             ) => {
//                 texts.push(text);
//                 Some(ColumnData::Text(texts))
//             }
            
//             _ => None
//         }
//     }
// }

#[derive(Debug)]
pub struct Column {
    pub heading: String,
    pub data: ColumnData,
}

#[derive(Debug)]
pub struct DocumentOutput;

 #[derive(Debug)]
pub struct TabularOutput;

#[derive(Debug)]
pub struct TabularFormats;

#[derive(Debug)]
pub enum OutputType {
    Tabular(TabularOutput),
    Document(DocumentOutput),
}

#[derive(Debug)]
pub struct Document {
    pub title: String,
    pub columns: Vec<Column>,
    pub outputs: Vec<OutputType>,
}

#[derive(Debug)]
pub struct ImportStatement(pub Vec<String>);

#[derive(Debug)]
pub struct MockadocFile {
    pub import_statement: ImportStatement,
    pub documents: Vec<Document>,
}


#[derive(Debug, Error)]
pub enum PackingErrorVariant {
    #[error("Unhandled tree shape")]
    SyntaxUnhandledTreeShape(String),

    #[error("SyntaxChildrenArrayCastError")]
    SyntaxChildrenArrayCastError(Vec<Option<(Rule, String, Option<String>)>>), // TODO This could probably use a type alias

    #[error("SyntaxNodeCountMismatch")]
    SyntaxNodeCountMismatch(Vec<Option<(Rule, String, Option<String>)>>), // TODO This could probably use a type alias
    
    #[error("InconsistentColumnTypes")]
    InconsistentColumnTypes { column_number: usize, row: usize },

    #[error("TableHasNoRows")]
    TableHasNoRows { column_heading: String },

    #[error("InconsistentTableRowWidths")]
    InconsistentTableRowWidths(LegacyTransposeError)
}

pub type PackingError =
    crate::utils::packing::PackingError<PackingErrorVariant, Rule>;

impl From<LegacyTransposeError> for PackingError {
    fn from(value: LegacyTransposeError) -> Self {
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
