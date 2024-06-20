use pest::iterators::Pairs;

use crate::{mockadoc::{packer::model::{ColumnData, MetadataProperty}, parser::Rule, MockadocError}, utils::{error::LanguageError, iterator::{Transpose, Group}}};

use super::{error::{make_no_array_match_found_error, make_tree_shape_error}, model::{Block, CellData, Column, ColumnHeading, ColumnValues, Document, ImportStatement, MetadataProperties, PackingError, PackingErrorVariant, PackingResult, SyntaxChildren, SyntaxToken, SyntaxTree}, utils::{vec_first_and_rest, vec_into_array_varied_length, FirstAndRest}};

fn parse_title(trees: Vec<SyntaxTree>) -> Result<String, PackingError> {
    match vec_into_array_varied_length(trees)? {
        [ Some((Rule::TEXT, providence, None))
        ] =>
            Ok(providence.as_string()),

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
                        Ok(ColumnHeading::Text(providence.as_string())),
                    
                    (Rule::METADATA_TAG, _, _) =>
                        Ok(ColumnHeading::MetadataTag),

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

fn parse_data_cell(tree: SyntaxTree) -> Result<CellData, PackingError> {
    match (tree.token.rule, tree.token.providence, tree.children) {
        (Rule::METADATA_PROPERTIES, _, Some(children)) =>
            parse_metadata_properties(children.get_values())
                .map(MetadataProperties)
                .map(CellData::MetadataProperties)
                .with_rule(Rule::METADATA_PROPERTIES),

        (Rule::mockagen_identifier, _, Some(SyntaxChildren::One(child))) => 
            parse_mockagen_identifier(*child)
                .map(CellData::MockagenId)
                .with_rule(Rule::mockagen_identifier),

        (Rule::TEXT, providence, _) =>
            Ok(CellData::Text(providence.as_string())),

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
    let data_columns = data_rows.into_iter().transpose();

    column_headings.into_iter()
        .zip(data_columns)
        .map(|(heading, column)| {
            match heading {
                ColumnHeading::MetadataTag => {
                    let tags_by_row = column.into_iter()
                        .enumerate()
                        .map(|(row, cell)|
                            match cell {
                                CellData::MetadataProperties(props) =>
                                    Ok(props),

                            _ =>
                                Err(PackingErrorVariant::InconsistentColumnTypes { heading: heading.clone(), cell, row })
                        })
                        .collect::<Result<Vec<_>, _>>()?;

                    Ok(Column { title: ColumnHeading::MetadataTag, data: ColumnValues::Metadata(tags_by_row) })
                }
                ColumnHeading::Text(text) => {
                    #[derive(PartialEq, Eq, Hash, Copy, Clone)]
                    enum CellType {
                        MockagenId = 0,
                        Text = 1,
                    }

                    let groups = column.into_iter()
                        .map(|cell|
                            match cell {
                                CellData::MockagenId(id) => (CellType::MockagenId, id),
                                CellData::Text(text) => (CellType::Text, text),
                                _ =>
                                    panic!("TODO user error to say you've got metadata in a data column")
                            })
                        .group();

                    let mut iter = groups.into_iter();

                    match iter.next() {
                        Some((cell_type, group)) => 
                            match iter.next() {
                                None => {
                                    let values = match cell_type {
                                        CellType::MockagenId =>
                                            ColumnValues::Data(ColumnData::MockagenId(group)),

                                        CellType::Text =>
                                            ColumnValues::Data(ColumnData::Text(group)),
                                    };

                                    Ok(Column { title: ColumnHeading::Text(text), data: values })
                                }
                                Some(_) => panic!("TODO compiler error to complain that there's mixed types. Use the values in the iter")
                            }

                        None =>
                            Err(PackingErrorVariant::TableHasNoRows { column_heading: text })
                    }
                }
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
            panic!("Unexpected value")
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

fn parse_import_statement(trees: Vec<SyntaxTree>) -> ImportStatement {
    ImportStatement(trees.into_iter().map(|path| path.as_string()).collect())
}

pub fn pack_mockadoc(pairs: Pairs<'_, Rule>) -> Result<Vec<Block>, MockadocError> {
    pairs.map(SyntaxTree::from)
        .map_while(|tree| {
            match (tree.token.rule, tree.children) {
                (Rule::import_statement, Some(children)) =>
                    Some(Ok(Block::ImportStatement(parse_import_statement(children.get_values())))),

                (Rule::documents, Some(children)) =>
                    Some(parse_documents(children.get_values()).with_rule(Rule::documents).map(Block::Documents)),

                (Rule::EOI, None) =>
                    None,

                (rule, children) =>
                    Some(make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children)))),
            }
        })
        .collect::<Result<_, _>>()
        .map_err(MockadocError::from_packing_err)
}
