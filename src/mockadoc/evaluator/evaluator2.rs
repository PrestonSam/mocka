use itertools::Itertools;

use crate::{mockadoc::{evaluator::model::EvaluationError, packer::{model::{Column, ColumnData}, Body, CollectVariant, ColumnNames, Document, Documents, Heading, ImportStatement, Outputs, Path, PathChars, RowValue, RowValueCollectedItersByVariant, Schema, Table}, MockadocError}, mockagen::{run_mockagen}, utils::iterator::Transpose};

fn parse_column(heading: String, cells: Vec<RowValue>) -> Column {
    let variant_iters = cells.into_iter()
        .collect_variant()
        .expect("TODO nice error for empty table");

    let column_data = match variant_iters {
        RowValueCollectedItersByVariant::MockagenIdAndMetadata(iter) =>
            ColumnData::MockagenIdAndMetadata(
                iter.collect::<Result<Vec<_>, _>>()
                    .expect("Another error, should probably be included in return type")),
        RowValueCollectedItersByVariant::Text(iter) => 
            ColumnData::Text(
                iter.map_ok(|t| t.0)
                    .collect::<Result<Vec<_>, _>>()
                    .expect("Another error, should probably be included in return type")),
    };

    Column { heading, data: column_data }
}

fn evaluate_imports(import_statement: ImportStatement) -> Result<GeneratorSet, EvaluationError> {
    match import_statement {
        ImportStatement(imports) => {
            let mut generators = None::<GeneratorSet>;

            for Path(PathChars(path)) in imports.into_iter() {
                let file = std::fs::read_to_string(path)?;
                let gen_set = run_mockagen(&file)?;

                match &mut generators {
                    Some(gens) => gens.merge(gen_set),
                    None => generators = Some(gen_set),
                }
            }

            Ok(generators.expect("TODO create error explaining that there are no imports"))
        }
    }
}

fn evaluate_column(column: Column, generators: &GeneratorSet) -> Result<ColumnGenerator, EvaluationError> {
    match column.data {
        ColumnData::Text(_) => todo!(),
        ColumnData::MockagenIdAndMetadata(mockagen_id_and_metadatas) => {
            let ids = mockagen_id_and_metadatas.iter()
                .map(|m| m.0.0.0.as_str());

            // TODO revisit `make_column_generator` internal logic
            generators.make_column_generator(ids)
                .map_err(EvaluationError::from)
        }
    }
}

fn evaluate_document(document: Document, generators: &GeneratorSet) -> Result<(), EvaluationError> {
    let Document(title, Schema(Table(Heading(ColumnNames(headings), _), rows)), Outputs(outputs)) = document;

    // The original design doesn't really work
    // as I need information from the outputs to determine what the columns mean.
    // How does this thing even parse?
    // Oh yeah it just tries whatever.
    // I'd still argue this is a sort of parsing.
    // It's funny. I think I need _two_ models.
    // One's a literal analogue of the parser, while the other is a more condensed version.

    // These two statements should ideally be moved into packer.rs as TokenRepackers
    let columns = rows.into_iter()
        .map(|r| r.0)
        .transpose::<Vec<_>>()?;

    let columns: Vec<_> = headings.into_iter()
        .zip(columns)
        .map(|(h, c)| parse_column(h.0, c))
        .collect();

    let col_gens = columns.into_iter()
        .map(|c| evaluate_column(c, &generators))
        .collect::<Result<Vec<_>, _>>()?;

    let output = col_gens.iter()
        .map(|c_g| c_g.generate_column())
        .collect::<Result<Vec<_>, _>>()?;

    dbg!(output);
    
    todo!()
}

pub fn evaluate_mockadoc(body: Body) -> Result<(), MockadocError> {
    let Body(_, import_statement, Documents(documents), _) = body;

    let generators = evaluate_imports(import_statement)?;

    let documents = documents.into_iter()
        .map(|document| evaluate_document(document, &generators).map_err(MockadocError::from))
        .collect::<Result<Vec<_>, _>>()?;


    todo!()
}