use pest::iterators::Pairs;

use crate::{mockadoc::{packer::model::MetadataProperty, parser::Rule, MockadocError}, utils::{error::LanguageError, iterator::Transpose}};

use super::{error::{make_no_array_match_found_error, make_tree_shape_error}, model::{CellData, Column, ColumnHeading, Document, ImportStatement, MetadataProperties, MockadocFile, PackingError, PackingErrorVariant, PackingResult, SyntaxChildren, SyntaxToken, SyntaxTree}, utils::{vec_first_and_rest, vec_into_array_varied_length, FirstAndRest}};

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

                    (Rule::DATA_KEY, providence, _) =>
                        Ok(ColumnHeading::DataKey(providence.as_string())),
                    
                    (Rule::METADATA_TAG, _, _) =>
                        Ok(ColumnHeading::MetadataTag),
                    
                    (Rule::GENERATOR_TAG, _, _) =>
                        Ok(ColumnHeading::GeneratorTag),

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

macro_rules! unpack_enum {
    ($expression:expr, $pattern:pat => $value:expr) => {
        match $expression {
            $pattern => Some($value),
            _ => None
        }
    };
}

macro_rules! unpack_enum_fn {
    ($pattern:pat => $value:expr) => {
        |cell| unpack_enum!(cell, $pattern => $value)
    }
}

fn transpose_table(column_headings: Vec<ColumnHeading>, data_rows: Vec<Vec<CellData>>) -> Result<Vec<Column>, PackingError> {
    let data_columns = data_rows.into_iter()
        .transpose()
        .map_err(PackingError::from)?;

    column_headings.into_iter()
        .zip(data_columns.enumerate())
        .map(|(heading, (col_no, column))| {
            fn into_column<T>(column_number: usize, column: Vec<CellData>, get_props: fn(CellData) -> Option<T>, make_column: impl FnOnce(Vec<T>) -> Column) -> Result<Column, PackingErrorVariant> {
                let tags_by_row = column.into_iter()
                    .enumerate()
                    .map(|(row, cell)|
                        get_props(cell)
                            .ok_or_else(|| PackingErrorVariant::InconsistentColumnTypes { column_number, row }))
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(make_column(tags_by_row))
            }

            match heading {
                ColumnHeading::MetadataTag =>
                    into_column(col_no, column, unpack_enum_fn!(CellData::MetadataProperties(props) => props), Column::Metadata),
                
                ColumnHeading::GeneratorTag =>
                    into_column(col_no, column, unpack_enum_fn!(CellData::MockagenId(props) => props), Column::Generators),
                
                ColumnHeading::Text(title) =>
                    into_column(col_no, column, unpack_enum_fn!(CellData::Text(props) => props), |data| Column::Text { title, data }),
                
                ColumnHeading::DataKey(title) =>
                    into_column(col_no, column, unpack_enum_fn!(CellData::Text(props) => props), |data| Column::DataKey { title, data })
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

    parse_entrypoint(trees).map_err(MockadocError::from_packing_err)

}
