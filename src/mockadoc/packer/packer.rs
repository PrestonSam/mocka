use pest::iterators::Pairs;

use crate::{mockadoc::{packer::model::{MetadataProperty, MockagenIdAndMeta}, parser::Rule, MockadocError}, utils::{error::LanguageError, iterator::Transpose}};

use super::{error::{make_no_array_match_found_error, make_tree_shape_error}, model::{CellData, Column, ColumnHeading, Document, ImportStatement, MetadataProperties, MockadocFile, PackingError, PackingErrorVariant, PackingResult, SyntaxChildren, SyntaxToken, SyntaxTree}, utils::{vec_first_and_rest, vec_into_array_varied_length, FirstAndRest}};

fn parse_title(trees: Vec<SyntaxTree>) -> Result<String, PackingError> {
    match vec_into_array_varied_length(trees)? {
        [ Some((Rule::TEXT, providence, None))
        ] =>
            Ok(providence.as_trimmed_string()),

        nodes =>
            make_no_array_match_found_error(nodes),
    }
}

fn parse_column_names(tree: SyntaxTree) -> Result<Vec<ColumnHeading>, PackingError> {
    match (tree.token.rule, tree.children) {
        (Rule::column_names, Some(children)) =>
            children.get_values_iter().map(|child| {
                match (child.token.rule, child.token.providence, child.children) {
                    (Rule::TEXT, providence, _) =>
                        Ok(ColumnHeading(providence.as_trimmed_string())),

                    (rule, providence, children) =>
                        make_tree_shape_error(SyntaxTree::from((rule, providence, children))),
                }
            })
            .collect::<Result<_, _>>()
            .with_rule(Rule::column_names),

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_metadata_properties(trees: Vec<SyntaxTree>) -> Result<Vec<MetadataProperty>, PackingError> {
    trees.into_iter()
        .map(|tree| {
            match (tree.token.rule, tree.children) {
                (Rule::PRIMARY_TIMESTAMP, _) =>
                    Ok(MetadataProperty::PrimaryTimestamp),
                
                (Rule::PERSONAL, _) =>
                    Ok(MetadataProperty::Personal),

                (rule, children) =>
                    make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
            }
        })
        .collect()
}

fn parse_mockagen_identifier(tree: SyntaxTree) -> Result<String, PackingError> {
    match (tree.token.rule, tree.token.providence, tree.children) {
        (Rule::MOCKAGEN_IDENTIFIER, providence, _) =>
            Ok(providence.as_string()),

        (rule, providence, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, providence, children))),
    }
}

fn parse_mockagen_id_and_metadata(trees: Vec<SyntaxTree>) -> Result<MockagenIdAndMeta, PackingError> {
    match vec_into_array_varied_length(trees)? {
        [ Some((Rule::mockagen_identifier, _, Some(SyntaxChildren::One(child))))
        , Some((Rule::METADATA_PROPERTIES, _, Some(children)))
        ] => {
            let id = parse_mockagen_identifier(*child)?;
            let metadata = parse_metadata_properties(children.get_values())
                .map(MetadataProperties)?;

            Ok(MockagenIdAndMeta(id, metadata))
        },

        nodes =>
            make_no_array_match_found_error(nodes),
    }
}

fn parse_data_cell(tree: SyntaxTree) -> Result<CellData, PackingError> {
    match (tree.token.rule, tree.token.providence, tree.children) {
        (Rule::mockagen_id_and_metadata, _, Some(children)) =>
            parse_mockagen_id_and_metadata(children.get_values())
                .map(CellData::MockagenIdAndMeta)
                .with_rule(Rule::mockagen_id_and_metadata),

        (Rule::TEXT, providence, _) =>
            Ok(CellData::Text(providence.as_trimmed_string())),

        (rule, providence, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, providence, children))),
    }
}

fn parse_data_row(tree: SyntaxTree) -> Result<Vec<CellData>, PackingError> {
    match (tree.token.rule, tree.children) {
        (Rule::row, Some(children)) =>
            children.get_values_iter()
                .map(parse_data_cell)
                .collect::<Result<_, _>>()
                .with_rule(Rule::row),

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn transpose_table(column_headings: Vec<ColumnHeading>, data_rows: Vec<Vec<CellData>>) -> Result<Vec<Column>, PackingError> {
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

            enum CellColumnData {
                Text(Vec<String>),
                MockagenIdAndMeta(Vec<MockagenIdAndMeta>),
            }

            impl CellColumnData {
                fn new(cell_data: CellData) -> Self {
                    match cell_data {
                        CellData::MockagenIdAndMeta(id_and_metadata) =>
                            Self::MockagenIdAndMeta(vec![ id_and_metadata ]),

                        CellData::Text(text) =>
                            Self::Text(vec![ text ]),
                    }
                }

                fn append(self, cell_data: CellData) -> Option<Self> {
                    match (self, cell_data) {
                        (CellColumnData::MockagenIdAndMeta(mut ids_and_metadatas), CellData::MockagenIdAndMeta(id_and_metadata)) => {
                            ids_and_metadatas.push(id_and_metadata);
                            Some(CellColumnData::MockagenIdAndMeta(ids_and_metadatas))
                        }

                        (CellColumnData::Text(mut texts), CellData::Text(text)) => {
                            texts.push(text);
                            Some(CellColumnData::Text(texts))
                        }
                        
                        _ => None
                    }
                }
            }
            
            enum CellColumn {
                Uninitialised,
                Errored,
                Valid(CellColumnData)
            }

            impl CellColumn {
                fn new() -> Self {
                    CellColumn::Uninitialised
                }

                fn append(self, cell_data: CellData) -> Self {
                    match self {
                        CellColumn::Valid(cell_column) =>
                            cell_column.append(cell_data)
                                .map(CellColumn::Valid)
                                .unwrap_or(CellColumn::Errored),

                        CellColumn::Uninitialised =>
                            CellColumn::Valid(CellColumnData::new(cell_data)),

                        CellColumn::Errored =>
                            CellColumn::Errored,
                    }
                }
            }

            let cell_column = column.into_iter()
                .enumerate()
                .fold(Ok(CellColumn::new()), |cell_column, (row, cell)| {
                    match cell_column?.append(cell) {
                        CellColumn::Errored =>
                            Err(PackingErrorVariant::InconsistentColumnTypes { column_number, row }),

                        cell_column =>
                            Ok(cell_column),
                    }
                })?;
            
            let ColumnHeading(title) = heading; 

            match cell_column {
                CellColumn::Valid(data) => {
                    match data {
                        CellColumnData::MockagenIdAndMeta(ids_and_metas) =>
                            Ok(Column::MockagenIdAndMeta { title, data: ids_and_metas }),

                        CellColumnData::Text(data) =>
                            Ok(Column::Text { title, data })
                    }
                },
                CellColumn::Errored => todo!("Produce error for this edgecase (should be impossible as these errors should already be propagated)"),
                CellColumn::Uninitialised => todo!("Produce error complaining that there are no cells in this column")
            }
        })
        .collect::<Result<_, _>>()
        .map_err(PackingError::new)
}

fn parse_table(trees: Vec<SyntaxTree>) -> Result<Vec<Column>, PackingError> {
    match vec_first_and_rest(trees) {
        FirstAndRest::Both(
            SyntaxTree {
                token: SyntaxToken { rule: Rule::heading, providence },
                children: Some(SyntaxChildren::One(column_names))
            },
            data_rows
        ) => {
            let column_headings = parse_column_names(*column_names)
                .with_rule(Rule::heading)?;
            let data_rows: Vec<_> = data_rows.into_iter()
                .map(parse_data_row)
                .collect::<Result<_, _>>()?;

            transpose_table(column_headings, data_rows)
                .map_err(|err| err.with_providence(providence))
        }

        FirstAndRest::OnlyFirst(_) =>
            todo!("Produce error: 'only header but no data'"),

        FirstAndRest::Neither =>
            todo!("Produce error: 'no header nor data'"),

        v => {
            dbg!(&v);
            todo!("Produce error: 'Unexpected value'")
        }
    }
}

fn parse_document(trees: Vec<SyntaxTree>) -> Result<Document, PackingError> {
    match vec_into_array_varied_length(trees)? {
        [ Some((Rule::title, _, Some(title_children)))
        , Some((Rule::table, _, Some(SyntaxChildren::Many(table_children))))
        ] => {
            let title = parse_title(title_children.get_values()).with_rule(Rule::title)?;
            let columns = parse_table(table_children).with_rule(Rule::table)?;

            Ok(Document { title, columns })
        }

        nodes =>
            make_no_array_match_found_error(nodes),
    }
}

fn parse_documents(trees: Vec<SyntaxTree>) -> Result<Vec<Document>, PackingError> {
    trees.into_iter()
        .map(|tree| {
            match (tree.token.rule, tree.children) {
                (Rule::document, Some(children)) =>
                    parse_document(children.get_values())
                        .with_rule(Rule::document),

                (rule, children) =>
                    make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
            }
        })
        .collect::<Result<_, _>>()
}

// TODO maybe put some validation in here? I'm not sure
fn parse_import_statement(trees: Vec<SyntaxTree>) -> ImportStatement {
    let imports = trees.into_iter()
        .map(|path| path.as_string())
        .collect();

    ImportStatement(imports)
}

fn parse_entrypoint(trees: Vec<SyntaxTree>) -> Result<MockadocFile, PackingError> {
    match vec_into_array_varied_length(trees)? {
        [ Some((Rule::import_statement, _, Some(import_children)))
        , Some((Rule::documents, _, Some(document_children)))
        , Some((Rule::EOI, _, None))
        ] => {
            let import_statement = parse_import_statement(import_children.get_values());
            let documents = parse_documents(document_children.get_values())
                .with_rule(Rule::documents)?;

            Ok(MockadocFile { import_statement, documents })
        }

        nodes =>
            make_no_array_match_found_error(nodes),
    }
}

pub fn pack_mockadoc(pairs: Pairs<'_, Rule>) -> Result<MockadocFile, MockadocError> {
    let trees = pairs.map(SyntaxTree::from).collect();

    parse_entrypoint(trees)
        .map_err(MockadocError::from_packing_err)
}
