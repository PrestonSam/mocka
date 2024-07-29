use crate::{
    mockadoc::{
        packer::model::{Column, Document, ImportStatement, MockadocFile},
        MockadocError,
    },
    mockagen::{run_mockagen, ColumnGenerator, GeneratorSet},
    utils::error::LanguageError,
};

use super::model::{EvaluationError, OutDocument};

fn evaluate_column(column: Column, generators: &GeneratorSet) -> Result<ColumnGenerator, EvaluationError> {
    match column {
        Column::Generators(gen_ids) =>
            generators
                .make_column_generator(gen_ids)
                .map_err(EvaluationError::from),

        Column::Metadata(metadatas) =>
            todo!("not quite sure how to do this yet"),

        Column::Text { title, data } => // This shouldn't be part of the data generation
            todo!(),
        
        Column::DataKey { title, data } =>
            todo!(),
    }
}

fn evaluate_document(document: Document, generators: &GeneratorSet) -> Result<OutDocument, EvaluationError> {
    let rows = document.columns
        .into_iter()
        .map(|column| evaluate_column(column, generators))
        .collect::<Result<Vec<_>, _>>()?;


    // I need some way to choose either to generate data or to produce the JSON output.
    // I'm becoming unsure about the exact purpose of the JSON text. Shouldn't I take it out of the scope all together?
    // Out or the original context, the JSON generation seems difficult to really justify
    // I think use clap, then have three modes
    // 'Static', 'Generated', and 'Both'

    Ok(OutDocument(rows))
}

fn evaluate_imports(import_statement: ImportStatement) -> Result<GeneratorSet, MockadocError> {
    match import_statement {
        ImportStatement(imports) => {
            let mut generators = None::<GeneratorSet>;

            for path in imports.into_iter() {
                let file = std::fs::read_to_string(path).map_err(MockadocError::from)?;
                let gen_set = run_mockagen(&file).map_err(MockadocError::from)?;

                match &mut generators {
                    Some(gens) => gens.merge(gen_set),
                    None => generators = Some(gen_set),
                }
            }

            Ok(generators.expect("TODO create error explaining that there are no imports"))
        }
    }
}

pub fn evaluate_mockadoc(mockadoc_file: MockadocFile) -> Result<(), MockadocError> {
    let generators = evaluate_imports(mockadoc_file.import_statement)?;

    let documents = mockadoc_file.documents
        .into_iter()
        .map(|document| evaluate_document(document, &generators).map_err(MockadocError::from_eval_err))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}
