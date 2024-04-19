use pest::iterators::Pairs;

use crate::mockagen::utils::error::make_error_from_providence;

use super::{
    model::{DefNode, Definition, Error, PackingError, Providence, Statement, SyntaxChildren, SyntaxTree, Value, Weight, WeightedValue},
    parser::Rule,
    utils::{
        error::{make_no_array_match_found_error, make_tree_shape_error},
        unpackers::{unpack_range, vec_into_array_varied_length}
    }
};

fn unpack_string_literal(tree: SyntaxTree) -> Result<String, Error> {
    match (tree.token.rule, tree.children) {
        (Rule::STRING_LITERAL, Some(SyntaxChildren::One(string_content))) =>
            Ok(string_content.token.providence.src.to_string()),

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_value_type(tree: SyntaxTree) -> Result<Value, Error> {
    match (tree.token.rule, tree.children) {
        (Rule::integer_value, Some(children)) =>
            unpack_range(Rule::INTEGER_LITERAL, Value::IntegerRange, children.get_values()),

        (Rule::real_value, Some(children)) =>
            unpack_range(Rule::REAL_LITERAL, Value::RealRange, children.get_values()),

        (Rule::string_value, Some(children)) =>
            unpack_range(Rule::INTEGER_LITERAL, Value::StringRange, children.get_values()),

        (Rule::timestamp_date_value, Some(children)) =>
            unpack_range(Rule::DATE_LITERAL, Value::DateRange, children.get_values()),

        (Rule::any_value, None) =>
            Ok(Value::Any),

        (Rule::literal_value, Some(SyntaxChildren::One(string_literal))) =>
            Ok(Value::Literal(unpack_string_literal(*string_literal)?)),

        (Rule::identifier_value, Some(SyntaxChildren::One(identifier))) =>
            Ok(Value::Identifier(identifier.token.providence.src.to_string())),

        (Rule::join_value, Some(children)) =>
            Ok(Value::Join(children.get_values_iter()
                .map(parse_value_type)
                .collect::<Result<Vec<_>, Error>>()?)
            ),

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_matchers(tree: SyntaxTree) -> Result<Vec<Value>, Error> {
    match (tree.token.rule, tree.children) {
        (Rule::value, Some(SyntaxChildren::One(child))) =>
            Ok(vec![ parse_value_type(*child)? ]),

        (Rule::matcher_set, Some(children)) =>
            children.get_values_iter()
                .map(parse_value_type)
                .collect(),

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_weighting(tree: SyntaxTree) -> Result<f64, Error> {
    let providence = tree.token.providence;
    let percentage_str = providence.src;

    percentage_str.parse::<f64>()
        .map_err(|err| make_error_from_providence(providence, PackingError::from(err)))
}

fn parse_maybe_weighted(tree: Vec<SyntaxTree>) -> Result<(Option<Weight>, SyntaxTree), Error> {
    match vec_into_array_varied_length(tree)? {
        [ Some((Rule::WEIGHT, _, Some(SyntaxChildren::One(weighting))))
        , Some(value)
        ] =>
            Ok((Some(parse_weighting(*weighting)?), SyntaxTree::from(value))),
        
        [ Some(value)
        , None
        ] => 
            Ok((None, SyntaxTree::from(value))),

        nodes =>
            make_no_array_match_found_error(nodes),
    }
}

fn parse_weighted_value(tree: SyntaxTree) -> Result<WeightedValue, Error> {
    match (tree.token.rule, tree.children) {
        (Rule::weighted_value, Some(children)) => {
            let (maybe_weight, value_tree) = parse_maybe_weighted(children.get_values())?;

            match (value_tree.token.rule, value_tree.children) {
                (Rule::value, Some(SyntaxChildren::One(value))) =>
                    Ok(WeightedValue {
                        weight: maybe_weight,
                        value: parse_value_type(*value)?
                    }),

                (rule, children) =>
                    make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
            }
        }

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_weighted_value_or_set<'a>(tree: SyntaxTree<'a>) -> Result<Vec<WeightedValue>, Error<'a>> {
    match (tree.token.rule, tree.children) {
        (Rule::value, Some(SyntaxChildren::One(value_tree))) =>
            Ok(vec![ WeightedValue { weight: None, value: parse_value_type(*value_tree)? } ]),

        (Rule::value_set, Some(children)) =>
            children.get_values_iter()
                .map(parse_weighted_value)
                .collect(),

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_values(tree: SyntaxTree) -> Result<Vec<WeightedValue>, Error> {
    match (tree.token.rule, tree.children) {
        (Rule::values, Some(SyntaxChildren::One(wval_or_set))) =>
            parse_weighted_value_or_set(*wval_or_set),

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_single_definition(nodes: Vec<SyntaxTree>) -> Result<Definition, Error> {
    match vec_into_array_varied_length(nodes)? {
        [ Some((Rule::IDENTIFIER, Providence { src: id, .. }, None))
        , Some(weighted_values)
        ] =>
            Ok(Definition::SingleDefinition {
                identifier: id.to_string(),
                values: parse_weighted_value_or_set(SyntaxTree::from(weighted_values))?
            }),

        nodes =>
            make_no_array_match_found_error(nodes),
    }
}

fn parse_names(trees: Vec<SyntaxTree>) -> Result<Vec<String>, Error> {
    trees.into_iter()
        .map(|tree| {
            match (tree.token.rule, tree.token.providence) {
                (Rule::IDENTIFIER, Providence { src, .. }) =>
                    Ok(src.to_string()),

                (rule, providence) =>
                    make_tree_shape_error(SyntaxTree::from((rule, providence, tree.children))),
            }
        }).collect()
}

fn parse_nest_def_branch(tree: SyntaxTree) -> Result<DefNode, Error> {
    match (tree.token.rule, tree.children) {
        (Rule::r#match, Some(children)) =>
            match vec_into_array_varied_length(children.get_values())? {
                [ Some((Rule::matchers, _, Some(SyntaxChildren::One(matchers))))
                , Some((Rule::match_sub_assignments | Rule::match_sub_matches, _, Some(children) ))
                ] => {
                    let children = children.get_values_iter()
                        .map(parse_nest_def_branch)
                        .collect::<Result<_, _>>()?;

                    Ok(DefNode::Match {
                        matchers: parse_matchers(*matchers)?,
                        children: Some(children),
                    })
                }

                nodes =>
                    make_no_array_match_found_error(nodes),
            }

        (Rule::assign, Some(children)) =>
            match vec_into_array_varied_length(children.get_values())? {
                [ Some((Rule::weighted_values, _, Some(weighted_value_trees)))
                , Some((Rule::sub_assignments, _, maybe_sub_assignment_trees))
                ] => {
                    let (maybe_weight, values_tree) = parse_maybe_weighted(weighted_value_trees.get_values())?;

                    let children = match maybe_sub_assignment_trees {
                        Some(trees) => Some(trees.get_values_iter().map(parse_nest_def_branch).collect::<Result<_, _>>()?),
                        None => None
                    };

                    Ok(DefNode::Assign {
                        weight: maybe_weight,
                        values: parse_values(values_tree)?,
                        children,
                    })
                }

                nodes =>
                    make_no_array_match_found_error(nodes),
            }

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_nested_definition(tree: Vec<SyntaxTree>) -> Result<Definition, Error> {
    let (maybe_using_ids, assign_ids, nested_clauses) = match vec_into_array_varied_length(tree)? {
        [ Some((Rule::using_ids, _, Some(using_ids)))
        , Some((Rule::assign_ids, _, Some(assign_ids)))
        , Some((Rule::nested_clauses, _, Some(nested_clauses)))
        ] =>
            (Some(parse_names(using_ids.get_values())?), assign_ids, nested_clauses),

        [ Some((Rule::assign_ids, _, Some(assign_ids)))
        , Some((Rule::nested_clauses, _, Some(nested_clauses)))
        , None
        ] =>
            (None, assign_ids, nested_clauses),

        nodes =>
            make_no_array_match_found_error(nodes)?,
    };


    let assign_ids = parse_names(assign_ids.get_values())?;
    let branches = nested_clauses.get_values_iter()
        .map(parse_nest_def_branch)
        .collect::<Result<Vec<_>, Error>>()?;

    Ok(Definition::NestedDefinition {
        using_ids: maybe_using_ids,
        identifiers: assign_ids,
        branches
    })
}

pub fn pack_mockagen(pairs: Pairs<'_, Rule>) -> Result<Vec<Statement>, Error> {
    pairs.map(SyntaxTree::from).map_while(|tree| {
        match (tree.token.rule, tree.children) {
            (Rule::include_statement, Some(children)) =>
                Some(children.get_values_iter()
                    .map(unpack_string_literal)
                    .collect::<Result<_, _>>()
                    .map(Statement::Include)
                ),
            
            (Rule::single_definition, Some(children)) =>
                Some(parse_single_definition(children.get_values()).map(Statement::Definition)),

            (Rule::nested_definition, Some(children)) =>
                Some(parse_nested_definition(children.get_values()).map(Statement::Definition)),
            
            (Rule::EOI, None) =>
                None,

            (rule, children) =>
                Some(make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children)))),
        }
    }).collect()
}
