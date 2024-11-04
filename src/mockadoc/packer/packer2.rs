use lang_packer::Packer;
use crate::mockadoc::packer::model::OutputType;


#[derive(Packer)]
#[Pack((Rule::TEXT, Kids::AsString))]
struct Title(String);

#[derive(Packer)]
#[Pack((Rule::TEXT, Kids::AsString))]
struct ColumnHeading(String);

#[derive(Packer)]
#[Pack((Rule::column_names, Kids::Many))]
struct Heading(Vec<ColumnHeading>);

#[derive(Packer)]
#[Pack((Rule::MOCKAGEN_IDENTIFIER, Kids::ToString))]
struct MockagenId(String);

#[derive(Packer)]
enum MetadataProperty {
    #[Pack((Rule::PRIMARY_TIMESTAMP, Kids::None))]
    PrimaryTimestamp,

    #[Pack((Rule::PERSONAL, Kids::None))]
    Personal,
}

#[derive(Packer)]
#[Pack(
    (Rule::mockagen_identifier, Kids::One),
    (Rule::METADATA_PROPERTIES, Kids::Many),
)]
struct MockagenIdAndMeta(MockagenId, Vec<MetadataProperty>);

#[derive(Packer)]
enum DataCell {
    #[Pack((Rule::mockagen_id_and_metadata, Kids::Many))]
    MockagenIdAndMetadata(MockagenIdAndMeta),

    #[Pack((Rule::TEXT, Kids::ToString))]
    Text(String),
}

#[derive(Packer)]
#[Pack((Rule::row, Kids::Many))]
struct Row(Vec<DataCell>);

#[derive(Packer)]
#[Pack(
    (Rule::heading, Kids::ToString),
    Rule::row ..
)]
struct Table(Heading, Vec<Row>);

#[derive(Packer)]
#[Pack(
    (Rule::SCHEMA_TAG, Kids::None),
    (Rule::table, Kids::Many),
)]
struct Schema(Table);

#[derive(Packer)]
#[Pack(
    (Rule::OUTPUTS_TAG, Kids::None),
    Rule::output_type ..
)]
struct Outputs(Vec<OutputType>);

#[derive(Packer)]
#[Pack(
    (Rule::title, Kids::Many),
    (Rule::schema, Kids::Many),
    (Rule::outputs, Kids::Many),
)]
struct Document(Title, Schema, Outputs);

#[derive(Packer)]
struct Documents(Vec<Document>);

#[derive(Packer)]
struct ImportStatement(Vec<String>);

#[derive(Packer)]
#[Pack(
    (Rule::import_statement, Kids::Many),
    (Rule::documents, Kids::Many),
    (Rule::EOI, Kids::None),
)]
struct MockadocFile(ImportStatement, Documents);
