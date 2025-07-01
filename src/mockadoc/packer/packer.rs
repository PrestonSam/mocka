#![allow(dead_code)]

use std::marker::PhantomData;

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
pub struct MockagenIdAndMetadata(pub MockagenIdentifier, pub Option<MetadataProperties>);

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
#[packer(rule = Rule::indented_x2_text)]
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
pub struct Row(pub Vec<RowValue>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::column_names)]
pub struct ColumnNames(pub Vec<Text>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::column_divider)]
pub struct ColumnDivider;

#[derive(Debug, Packer)]
#[packer(rule = Rule::table_divider)]
pub struct TableDivider(Vec<ColumnDivider>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::heading)]
pub struct Heading(pub ColumnNames, pub TableDivider);

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
#[packer(rule = Rule::path_chars)]
pub struct PathChars(pub String);

#[derive(Debug, Packer)]
#[packer(rule = Rule::path)]
pub struct Path(pub PathChars);

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






// At the moment the purpose of this is just to grab the type.
// I might be able to merge it with CollectVariantIter, which I believe has enough overlap

// This is attached to RowValue
pub trait HasVariantIters {
    type VariantIters<I>: CollectItersByVariant<I>
    where
        I: Iterator<Item = Self> + Sized,
        I::Item: HasVariantIters;
}

// See? I told you so
impl HasVariantIters for RowValue {
    type VariantIters<I> = RowValueCollectedItersByVariant<I>
    where
        I: Iterator<Item = Self> + Sized;
}

// This is an interface for the outputted enum returned after you call collect_variant on an iterator
pub trait CollectItersByVariant<I>
where
    I: Iterator<Item: HasVariantIters> + Sized
{
    fn new(src: I, first: I::Item) -> Self;
}

// This is an implementation of CollectItersByVariant
pub enum RowValueCollectedItersByVariant<I>
where
    I: Iterator<Item = RowValue> + Sized,
{
    MockagenIdAndMetadata(CollectedVariantIter<I, MockagenIdAndMetadata>),
    Text(CollectedVariantIter<I, Text>),
}

// There. I told you so
impl<I> CollectItersByVariant<I> for RowValueCollectedItersByVariant<I>
where
    I: Iterator<Item = RowValue> + Sized,
{
    fn new(src: I, first: I::Item) -> Self
    {
        match &first {
            RowValue::MockagenIdAndMetadata(_) => Self::MockagenIdAndMetadata(CollectedVariantIter::new(src, first)),
            RowValue::Text(_) => Self::Text(CollectedVariantIter::new(src, first)),
        }
    }
}

// This is an error that's produced when a value is discovered to be of the wrong variant
#[derive(Debug)]
pub enum CollectVariantError {
    InconsistentEnumVariants,
}

// This consumes the enum (RowValue) and produces the value under the variant.
// This is basically "UnwrapEnumVariant"
pub trait TryGetVariant
where
    Self: Sized
{
    type Enum;

    fn try_from_variant(from: Self::Enum) -> Option<Self>;
}

// Here I implement the trait for MockagenIdAndMetadata
impl TryGetVariant for MockagenIdAndMetadata {
    type Enum = RowValue;

    fn try_from_variant(from: Self::Enum) -> Option<Self> {
        match from {
            RowValue::MockagenIdAndMetadata(mockagen_id_and_metadata) => Some(mockagen_id_and_metadata),
            RowValue::Text(_) => None,
        }
    }
}

// And here I implement the trait for Text
impl TryGetVariant for Text {
    type Enum = RowValue;

    fn try_from_variant(from: Self::Enum) -> Option<Self> {
        match from {
            RowValue::Text(text) => Some(text),
            RowValue::MockagenIdAndMetadata(_) => None,
        }
    }
}

// This is the state for the iterator that produces either the extracted enum or the error we visited earlier
pub struct CollectedVariantIter<I, V>
where
    I: Iterator + Sized,
    I::Item: HasVariantIters,
    V: TryGetVariant<Enum = I::Item>
{
    src: I,
    first: Option<I::Item>,
    _phantom: PhantomData<V>,
}

// Here's a `new` function for the sake of convenience
impl<I, V> CollectedVariantIter<I, V>
where
    I: Iterator + Sized,
    I::Item: HasVariantIters,
    V: TryGetVariant<Enum = I::Item>
{
    fn new(src: I, first: I::Item) -> Self {
        CollectedVariantIter {
            src,
            first: Some(first),
            _phantom: PhantomData
        }
    }
}

// Here's the implementation of iterator for that state struct
impl<I, V> Iterator for CollectedVariantIter<I, V> 
where
    I: Iterator + Sized,
    I::Item: HasVariantIters,
    V: TryGetVariant<Enum = I::Item>,
{
    type Item = Result<V, CollectVariantError>;

    fn next(&mut self) -> Option<Self::Item> {
        let first = &mut self.first; // Broke this out onto two lines to evade a bug in rust-analyzer
        let next = first.take()
            .or_else(|| self.src.next())?;

        let variant = V::try_from_variant(next)
            .ok_or(CollectVariantError::InconsistentEnumVariants);

        Some(variant)
    }
}

// This is the trait that adds `collect_variant` to any iterator whose Item implements `HasVariantIters`
pub trait CollectVariant
where
    Self: Iterator + Sized,
    Self::Item: HasVariantIters,
{
    fn collect_variant(mut self) -> Option<<Self::Item as HasVariantIters>::VariantIters<Self>> {
        let fst = self.next()?;

        let out = <Self::Item as HasVariantIters>::VariantIters::new(self, fst);

        Some(out)
    }
}

// Here's the boilerplate to automatically bind the trait to all iterators matching the criteria
impl<I> CollectVariant for I
where
    I: Iterator + Sized,
    I::Item: HasVariantIters,
{}

#[test]
fn test_collect_variant() {
    let vals = vec![
        RowValue::Text(Text("Soem text".into())),
        RowValue::Text(Text("Other text".into())),
        RowValue::Text(Text("Final text".into())),
        // RowValue::MockagenIdAndMetadata(MockagenIdAndMetadata(MockagenIdentifier(MockagenId("SomeId".into())), None)),
    ];

    let variant_iter = vals.into_iter().collect_variant().unwrap();

    match variant_iter {
        RowValueCollectedItersByVariant::MockagenIdAndMetadata(mockagen_id_and_metadata_iter) => {
            dbg!(mockagen_id_and_metadata_iter.collect::<Result<Vec<_>, _>>().unwrap());
        },
        RowValueCollectedItersByVariant::Text(text_iter) => {
            dbg!(text_iter.collect::<Result<Vec<_>, _>>().unwrap());
        },
    };
}
