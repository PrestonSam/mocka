use crate::mockadoc::{packer::model::{Block, Column, Document}, MockadocError};

fn evaluate_column(column: Column) -> Result<(), MockadocError> {
    let _something = match column {
        Column::Generators(generators) => todo!(),
        Column::Metadata(metadatas) => todo!(),
        Column::Text { title, data } => todo!(),
    };

    Ok(())
}

fn evaluate_document(document: Document) -> Result<(), MockadocError> {
    let _something = document.columns.into_iter()
        .map(evaluate_column)
        .collect::<Vec<_>>();

    Ok(())
}

pub fn evaluate_mockadoc(blocks: Vec<Block>) -> Result<(), MockadocError> {
    let _something = blocks.into_iter()
        .map(|block|
            match block {
                Block::ImportStatement(imports) =>
                    todo!("imports"),

                Block::Documents(documents) =>
                    documents.into_iter()
                        .map(evaluate_document)
                        .collect::<Vec<_>>(),
            }
        )
        .collect::<Vec<_>>();
    
    Ok(())
}
