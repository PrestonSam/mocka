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
        (Rule::STRING_LITERAL, SyntaxChildren::Wrapper(string_content)) =>
            Ok(string_content.token.providence.src.to_string()),

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_value_type(tree: SyntaxTree) -> Result<Value, Error> {
    match (tree.token.rule, tree.children) {
        (Rule::integer_value, SyntaxChildren::Node(children)) =>
            unpack_range(Rule::INTEGER_LITERAL, Value::IntegerRange, children),

        (Rule::real_value, SyntaxChildren::Node(children)) =>
            unpack_range(Rule::REAL_LITERAL, Value::RealRange, children),

        (Rule::string_value, SyntaxChildren::Node(children)) =>
            unpack_range(Rule::INTEGER_LITERAL, Value::StringRange, children),

        (Rule::timestamp_date_value, SyntaxChildren::Node(children)) =>
            unpack_range(Rule::DATE_LITERAL, Value::DateRange, children),

        (Rule::any_value, SyntaxChildren::Leaf) =>
            Ok(Value::Any),

        (Rule::literal_value, SyntaxChildren::Wrapper(string_literal)) =>
            Ok(Value::Literal(unpack_string_literal(*string_literal)?)),

        (Rule::identifier_value, SyntaxChildren::Wrapper(identifier)) =>
            Ok(Value::Identifier(identifier.token.providence.src.to_string())),

        (Rule::join_value, SyntaxChildren::Node(children)) =>
            Ok(Value::Join(children.into_iter()
                .map(parse_value_type)
                .collect::<Result<Vec<_>, Error>>()?)
            ),

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_matchers(tree: SyntaxTree) -> Result<Vec<Value>, Error> {
    match (tree.token.rule, tree.children) {
        (Rule::value, SyntaxChildren::Wrapper(child)) =>
            Ok(vec![ parse_value_type(*child)? ]),

        (Rule::matcher_set, SyntaxChildren::Node(children)) =>
            children.into_iter()
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
        [ Some((Rule::WEIGHT, _, SyntaxChildren::Wrapper(weighting)))
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
        (Rule::weighted_value, SyntaxChildren::Node(children)) => {
            let (maybe_weight, value_tree) = parse_maybe_weighted(children)?;

            match (value_tree.token.rule, value_tree.children) {
                (Rule::value, SyntaxChildren::Wrapper(value)) =>
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
        (Rule::value, SyntaxChildren::Wrapper(value_tree)) =>
            Ok(vec![ WeightedValue { weight: None, value: parse_value_type(*value_tree)? } ]),

        (Rule::value_set, SyntaxChildren::Node(children)) =>
            children.into_iter()
                .map(parse_weighted_value)
                .collect(),

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_values(tree: SyntaxTree) -> Result<Vec<WeightedValue>, Error> {
    match (tree.token.rule, tree.children) {
        (Rule::values, SyntaxChildren::Wrapper(wval_or_set)) =>
            parse_weighted_value_or_set(*wval_or_set),

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_single_definition(nodes: Vec<SyntaxTree>) -> Result<Definition, Error> {
    match vec_into_array_varied_length(nodes)? {
        [ Some((Rule::IDENTIFIER, Providence { src: id, .. }, SyntaxChildren::Leaf))
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

fn parse_children(trees: Vec<SyntaxTree>) -> Result<Option<Vec<DefNode>>, Error> {
    if 0 < trees.len() {
        trees.into_iter()
            .map(parse_nest_def_branch)
            .collect::<Result<_, _>>()
            .map(Some)
    } else {
        Ok(None)
    }
}

fn parse_nest_def_branch(tree: SyntaxTree) -> Result<DefNode, Error> {
    match (tree.token.rule, tree.children) {
        (Rule::r#match, SyntaxChildren::Node(children)) =>
            match vec_into_array_varied_length(children)? {
                [ Some((Rule::matchers, _, SyntaxChildren::Wrapper(matchers)))
                , Some((Rule::match_sub_assignments | Rule::match_sub_matches, _, SyntaxChildren::Node(children) ))
                ] =>
                    Ok(DefNode::Match {
                        matchers: parse_matchers(*matchers)?,
                        children: parse_children(children)?
                    }),

                nodes =>
                    make_no_array_match_found_error(nodes),
            }

        (Rule::assign, SyntaxChildren::Node(children)) =>
            match vec_into_array_varied_length(children)? {
                [ Some((Rule::weighted_values, _, SyntaxChildren::Node(weighted_value_pairs)))
                , Some((Rule::sub_assignments, _, SyntaxChildren::Node(sub_assignments_pairs)))
                ] => {
                    let (maybe_weight, values_tree) = parse_maybe_weighted(weighted_value_pairs)?;

                    Ok(DefNode::Assign {
                        weight: maybe_weight,
                        values: parse_values(values_tree)?,
                        children: parse_children(sub_assignments_pairs)?
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
        [ Some((Rule::using_ids, _, SyntaxChildren::Node(using_ids)))
        , Some((Rule::assign_ids, _, SyntaxChildren::Node(assign_ids)))
        , Some((Rule::nested_clauses, _, SyntaxChildren::Node(nested_clauses)))
        ] =>
            (Some(parse_names(using_ids)?), assign_ids, nested_clauses),

        [ Some((Rule::assign_ids, _, SyntaxChildren::Node(assign_ids)))
        , Some((Rule::nested_clauses, _, SyntaxChildren::Node(nested_clauses)))
        , None
        ] =>
            (None, assign_ids, nested_clauses),

        nodes =>
            make_no_array_match_found_error(nodes)?,
    };


    let assign_ids = parse_names(assign_ids)?;
    let branches = nested_clauses.into_iter()
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
            (Rule::include_statement, SyntaxChildren::Node(children)) =>
                Some(children.into_iter()
                    .map(unpack_string_literal)
                    .collect::<Result<_, _>>()
                    .map(Statement::Include)
                ),
            
            (Rule::single_definition, SyntaxChildren::Node(children)) =>
                Some(parse_single_definition(children).map(Statement::Definition)),

            (Rule::nested_clauses, SyntaxChildren::Node(children)) =>
                Some(parse_nested_definition(children).map(Statement::Definition)),
            
            (Rule::EOI, SyntaxChildren::Leaf) =>
                None,

            (rule, children) =>
                Some(make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children)))),
        }
    }).collect()
}
