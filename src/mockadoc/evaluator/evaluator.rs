use crate::{
    mockadoc::MockadocError,
    mockadoc::packer::*,
    mockagen::{run_mockagen, ColumnGenerator, GeneratorSet},
    utils::error::LanguageError,
};

use super::model::{EvaluationError, OutDocument};

fn transpose_table(
        column_headings: Vec<ColumnHeading>,
        data_rows: Vec<Vec<CellData>>
    ) -> Result<Vec<Column>, PackingError>
{
    fn into_column<T>(
        column_number: usize,
        column: Vec<CellData>,
        get_props: fn(CellData) -> Option<T>,
        make_column: impl FnOnce(Vec<T>) -> Column
    ) -> Result<Column, PackingErrorVariant> {

        let tags_by_row = column.into_iter()
            .enumerate()
            .map(|(row, cell)|
                get_props(cell)
                    .ok_or_else(|| PackingErrorVariant::InconsistentColumnTypes { column_number, row }))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(make_column(tags_by_row))
    }

    let data_columns = data_rows.into_iter()
        .transpose()
        .map_err(PackingError::from)?;

    column_headings.into_iter()
        .zip(data_columns.enumerate())
        .map(|(heading, (column_number, column))| {

            enum ColumnPacker {
                Uninitialised,
                Errored,
                Valid(ColumnData)
            }

            impl ColumnPacker {
                fn new() -> Self {
                    ColumnPacker::Uninitialised
                }

                fn append(self, cell_data: CellData) -> Self {
                    match self {
                        ColumnPacker::Valid(cell_column) =>
                            cell_column.append(cell_data)
                                .map(ColumnPacker::Valid)
                                .unwrap_or(ColumnPacker::Errored),

                        ColumnPacker::Uninitialised =>
                            ColumnPacker::Valid(ColumnData::new(cell_data)),

                        ColumnPacker::Errored =>
                            ColumnPacker::Errored,
                    }
                }
            }

            let cell_column = column.into_iter()
                .enumerate()
                .fold(Ok(ColumnPacker::new()), |cell_column, (row, cell)| {
                    match cell_column?.append(cell) {
                        ColumnPacker::Errored =>
                            Err(PackingErrorVariant::InconsistentColumnTypes { column_number, row }),

                        cell_column =>
                            Ok(cell_column),
                    }
                })?;
            
            let ColumnHeading(title) = heading; 

            match cell_column {
                ColumnPacker::Valid(data) => Ok(Column { title, data }),
                ColumnPacker::Errored => todo!("Produce error for this edgecase (should be impossible as these errors should already be propagated)"),
                ColumnPacker::Uninitialised => todo!("Produce error complaining that there are no cells in this column")
            }
        })
        .collect::<Result<_, _>>()
        .map_err(PackingError::new)
}

fn evaluate_column(column: Column, generators: &GeneratorSet) -> Result<ColumnGenerator, EvaluationError> {
    todo!()
    // match column {
    //     Column::Generators(gen_ids) =>
    //         generators
    //             .make_column_generator(gen_ids)
    //             .map_err(EvaluationError::from),

    //     Column::Metadata(metadatas) =>
    //         todo!("not quite sure how to do this yet"),

    //     Column::Text { title, data } => // This shouldn't be part of the data generation
    //         todo!(),
        
    //     Column::DataKey { title, data } =>
    //         todo!(),
    // }
}

fn evaluate_document(document: Document, generators: &GeneratorSet) -> Result<OutDocument, EvaluationError> {
    let Document(title, Schema(Table(heading, rows)), outputs) = document;

    let columns = transpose_table(column_headings, data_rows)?;
    
    let rows = columns.into_iter()
        .map(|column| evaluate_column(column, generators))
        .collect::<Result<Vec<_>, _>>()?;


    Ok(OutDocument(rows))
}

fn evaluate_imports(import_statement: ImportStatement) -> Result<GeneratorSet, MockadocError> {
    match import_statement {
        ImportStatement(imports) => {
            let mut generators = None::<GeneratorSet>;

            for path in imports.into_iter() {
                let Path(path) = path;
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

pub fn evaluate_mockadoc(mockadoc_file: Body) -> Result<(), MockadocError> {
    let Body(_, import_statement, Documents(documents), _) = mockadoc_file;
    let generators = evaluate_imports(import_statement)?;

    let documents = documents
        .into_iter()
        .map(|document| evaluate_document(document, &generators).map_err(MockadocError::from_eval_err))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}


// fn get_debug_object() -> MockadocFile {
//     MockadocFile {
//         import_statement: ImportStatement(vec![ String::from("\"./debug.mkg\"") ]),
//         documents: vec![
//             Document {
//                 title: String::from("Transaction"),
//                 columns: vec![
//                     Column::Text {
//                         title: String::from("Template name"),
//                         data: vec![ String::from("Timestamp"), String::from("Name") ],
//                     },
//                     Column::Text {
//                         title: String::from("Internal name"),
//                         data: vec![ String::from("UnixTimestamp"), String::from("ActorName") ],
//                     },
//                     Column::Text {
//                         title: String::from("DSQL Type"),
//                         data: vec![ String::from("string"), String::from("string") ],
//                     },
//                     Column::Generators(vec![ String::from("unix-timestamp"), String::from("full-name") ]),
//                     Column::Metadata(
//                         vec![
//                             MetadataProperties(vec![MetadataProperty::PrimaryTimestamp]),
//                             MetadataProperties(vec![MetadataProperty::Personal]),
//                         ],
//                     ),
//                 ],
//             },
//         ],
//     }
// }
