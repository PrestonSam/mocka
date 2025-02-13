use lang_packer::Packer;

use crate::mockadoc::parser::Rule;

#[derive(Debug, Packer)]
#[packer(rule = Rule::PERSONAL)]
pub struct Personal;

#[derive(Debug, Packer)]
#[packer(rule = Rule::PRIMARY_TIMESTAMP)]
pub struct PrimaryTimestamp;

#[derive(Debug, Packer)]
#[packer(rule = Rule::primary_timestamp_and_personal)]
pub struct PrimaryTimestampAndPersonal(pub PrimaryTimestamp, pub Personal);

#[derive(Debug, Packer)]
#[packer(rule = Rule::METADATA_PROPERTIES)]
pub enum MetadataProperties {
    PrimaryTimestampAndPersonal(PrimaryTimestampAndPersonal),
    PrimaryTimestamp(PrimaryTimestamp),
    Personal(Personal),
}

#[derive(Debug, Packer)]
#[packer(rule = Rule::MOCKAGEN_IDENTIFIER)]
pub struct MockagenId(pub String);

#[derive(Debug, Packer)]
#[packer(rule = Rule::mockagen_identifier)]
pub struct MockagenIdentifier(pub MockagenId);

#[derive(Debug, Packer)]
#[packer(rule = Rule::mockagen_id_and_metadata)]
pub struct MockagenIdAndMetadata(pub MockagenIdentifier, pub MetadataProperties);

#[derive(Debug, Packer)]
#[packer(rule = Rule::row_value)]
pub enum RowValue {
    MockagenIdAndMetadata(MockagenIdAndMetadata),
    Text(Text),
}

#[derive(Debug, Packer)]
#[packer(rule = Rule::TEXT)]
pub struct Text(pub String);

#[derive(Debug, Packer)]
#[packer(rule = Rule::indented_x4_text)]
pub struct IndentedX4Text(pub Text);

#[derive(Debug, Packer)]
#[packer(rule = Rule::output_document_members)]
pub struct DocumentMembers(pub Vec<IndentedX4Text>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::json)]
pub struct Json;

#[derive(Debug, Packer)]
#[packer(rule = Rule::output_document_format)]
pub enum DocumentFormat {
    Json(Json),
}

#[derive(Debug, Packer)]
#[packer(rule = Rule::output_document_format_indented)]
pub struct DocumentFormatIndented(pub DocumentFormat);

#[derive(Debug, Packer)]
#[packer(rule = Rule::output_document_formats)]
pub struct DocumentFormats(pub Vec<DocumentFormatIndented>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::output_document)]
pub struct OutputDocument(pub DocumentFormats, pub DocumentMembers);

#[derive(Debug, Packer)]
#[packer(rule = Rule::output_tabular_row_values)]
pub struct TabularRowValues(pub Vec<IndentedX4Text>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::output_tabular_column_names)]
pub struct TabularColumnNames(pub Vec<IndentedX4Text>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::csv)]
pub struct Csv;

#[derive(Debug, Packer)]
#[packer(rule = Rule::tsv)]
pub struct Tsv;

#[derive(Debug, Packer)]
#[packer(rule = Rule::output_tabular_format_type)]
pub enum OutputTabularFormatType {
    Csv(Csv),
    Tsv(Tsv),
}

#[derive(Debug, Packer)]
#[packer(rule = Rule::output_tabular_format_type_indented)]
pub struct TabularFormatTypeIndented(pub OutputTabularFormatType);

#[derive(Debug, Packer)]
#[packer(rule = Rule::output_tabular_formats)]
pub struct TabularFormats(pub Vec<TabularFormatTypeIndented>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::output_tabular)]
pub struct OutputTabular(pub TabularFormats, pub TabularColumnNames, pub TabularRowValues);

#[derive(Debug, Packer)]
#[packer(rule = Rule::output_type)]
pub enum OutputType {
    Tabular(OutputTabular),
    Document(OutputDocument),
}

#[derive(Debug, Packer)]
#[packer(rule = Rule::outputs)]
pub struct Outputs(pub Vec<OutputType>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::row)]
pub struct Row(pub RowValue);

#[derive(Debug, Packer)]
#[packer(rule = Rule::column_names)]
pub struct ColumnNames(pub Vec<Text>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::heading)]
pub struct Heading(pub ColumnNames);

#[derive(Debug, Packer)]
#[packer(rule = Rule::table)]
pub struct Table(pub Heading, pub Vec<Row>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::schema)]
pub struct Schema(pub Table);

#[derive(Debug, Packer)]
#[packer(rule = Rule::title)]
pub struct Title(pub Text);

#[derive(Debug, Packer)]
#[packer(rule = Rule::document)]
pub struct Document(pub Title, pub Schema, pub Outputs);

#[derive(Debug, Packer)]
#[packer(rule = Rule::documents)]
pub struct Documents(pub Vec<Document>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::path)]
pub struct Path(pub String);

#[derive(Debug, Packer)]
#[packer(rule = Rule::import_statement)]
pub struct ImportStatement(pub Vec<Path>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::properties)]
pub struct Properties;

#[derive(Debug, Packer)]
#[packer(rule = Rule::EOI)]
pub struct EOI;

#[derive(Debug, Packer)]
#[packer(rule = Rule::body)]
pub struct Body(pub Option<Properties>, pub ImportStatement, pub Documents, pub EOI);
