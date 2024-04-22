use pest::iterators::Pairs;

use crate::mockagen::utils::error::make_error_from_providence;

use super::{
    model::{AssignNode, DefSet, Definition, Error, HigherOrderValue, MatchExpr, MatchNode, PackingError, PrimitiveValue, Providence, Statement, SyntaxChildren, SyntaxTree, Value, Weight, WeightedValue, WildcardNode},
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

fn parse_primitive_value(tree: SyntaxTree) -> Result<PrimitiveValue, Error> {
    match (tree.token.rule, tree.children) {
        (Rule::integer_value, Some(children)) =>
            unpack_range(Rule::INTEGER_LITERAL, PrimitiveValue::IntegerRange, children.get_values()),

        (Rule::real_value, Some(children)) =>
            unpack_range(Rule::REAL_LITERAL, PrimitiveValue::RealRange, children.get_values()),

        (Rule::string_value, Some(children)) =>
            unpack_range(Rule::INTEGER_LITERAL, PrimitiveValue::StringRange, children.get_values()),

        (Rule::timestamp_date_value, Some(children)) =>
            unpack_range(Rule::DATE_LITERAL, PrimitiveValue::DateRange, children.get_values()),

        (Rule::literal_value, Some(SyntaxChildren::One(string_literal))) =>
            Ok(PrimitiveValue::Literal(unpack_string_literal(*string_literal)?)),

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_higher_order_value(tree: SyntaxTree) -> Result<HigherOrderValue, Error> {
    match (tree.token.rule, tree.children) {
        (Rule::identifier_value, Some(SyntaxChildren::One(identifier))) =>
            Ok(HigherOrderValue::Identifier(identifier.token.providence.src.to_string())),

        (Rule::join_value, Some(children)) =>
            Ok(HigherOrderValue::Join(children.get_values_iter()
                .map(parse_value)
                .collect::<Result<Vec<_>, Error>>()?)
            ),

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_value(tree: SyntaxTree) -> Result<Value, Error> {
    match (tree.token.rule, tree.children) {
        (Rule::primitive_value, Some(SyntaxChildren::One(child))) =>
            Ok(Value::PrimitiveValue(parse_primitive_value(*child)?)),
        
        (Rule::higher_order_value, Some(SyntaxChildren::One(child))) =>
            Ok(Value::HigherOrderValue(parse_higher_order_value(*child)?)),

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_match_expr(tree: SyntaxTree) -> Result<MatchExpr, Error> {
    match (tree.token.rule, tree.children) {
        (Rule::literal_value, Some(SyntaxChildren::One(string_literal))) =>
            Ok(MatchExpr::Literal(unpack_string_literal(*string_literal)?)),

        (Rule::any_value, None) =>
            Ok(MatchExpr::Any),

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_matchers(tree: SyntaxTree) -> Result<Vec<MatchExpr>, Error> {
    match (tree.token.rule, tree.children) {
        (Rule::match_expr, Some(SyntaxChildren::One(child))) =>
            Ok(vec![ parse_match_expr(*child)? ]),

        (Rule::matcher_set, Some(children)) =>
            children.get_values_iter()
                .map(parse_match_expr)
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
                        value: parse_value(*value)?
                    }),

                (rule, children) =>
                    make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
            }
        }

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_weighted_value_or_set<'a>(tree: SyntaxTree<'a>) -> Result<Vec<WeightedValue>, Error> {
    match (tree.token.rule, tree.children) {
        (Rule::value, Some(SyntaxChildren::One(value_tree))) =>
            Ok(vec![ WeightedValue { weight: None, value: parse_value(*value_tree)? } ]),

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

fn parse_match_clause(tree: SyntaxTree) -> Result<MatchNode, Error> {
    match (tree.token.rule, tree.children) {
        (Rule::match_clause, Some(children)) =>
            match vec_into_array_varied_length(children.get_values())? {
                [ Some((Rule::matchers, _, Some(SyntaxChildren::One(matchers))))
                , Some((Rule::nested_clauses, _, Some(SyntaxChildren::One(tree)) ))
                ] => {
                    Ok(MatchNode {
                        matchers: parse_matchers(*matchers)?,
                        children: parse_def_set(*tree)?,
                    })
                }

                nodes =>
                    make_no_array_match_found_error(nodes),
            }

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_assign_clause(tree: SyntaxTree) -> Result<AssignNode, Error> {
    match (tree.token.rule, tree.children) {
        (Rule::assign_clause, Some(children)) => {
            let (weighted_value_trees, maybe_children) = match vec_into_array_varied_length(children.get_values())? {
                [ Some((Rule::weighted_values, _, Some(weighted_value_trees)))
                , Some((Rule::assign_clauses, _, Some(sub_assignment_trees)))
                ] => {
                    let children = sub_assignment_trees.get_values_iter()
                        .map(parse_assign_clause)
                        .collect::<Result<_, _>>()?;

                    (weighted_value_trees, Some(children))
                }

                [ Some((Rule::weighted_values, _, Some(weighted_value_trees)))
                , None
                ] => 
                    (weighted_value_trees, None),

                nodes =>
                    make_no_array_match_found_error(nodes)?,
            };

            let (maybe_weight, values_tree) = parse_maybe_weighted(weighted_value_trees.get_values())?;

            Ok(AssignNode {
                weight: maybe_weight,
                values: parse_values(values_tree)?,
                children: maybe_children,
            })
        }
            

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_def_set(tree: SyntaxTree) -> Result<DefSet, Error> {
    match (tree.token.rule, tree.children) {
        (Rule::match_clauses, Some(children)) => {
            Ok(DefSet::Match {
                nodes: children.get_values_iter()
                    .map(parse_match_clause)
                    .collect::<Result<_, _>>()?
            })
        }

        (Rule::match_clauses_with_wildcard, Some(children)) => {
            match vec_into_array_varied_length(children.get_values())? {
                [ Some((Rule::match_clauses, _, Some(match_clause_children)))
                , Some((Rule::wildcard_clause, _, Some(SyntaxChildren::One(wildcard_nested_clauses))))
                ] => {
                    let nodes = match_clause_children.get_values_iter()
                        .map(parse_match_clause)
                        .collect::<Result<_, _>>()?;

                    let wildcard_node = WildcardNode {
                        children: parse_nested_clauses(*wildcard_nested_clauses)?
                    };

                    Ok(DefSet::MatchWithWildCard {
                        nodes,
                        wildcard_node: Box::new(wildcard_node),
                    })
                }

                nodes =>
                    make_no_array_match_found_error(nodes)?,
            }
        }

        (Rule::assign_clauses, Some(children)) => {
            Ok(DefSet::Assign {
                nodes: children.get_values_iter()
                    .map(parse_assign_clause)
                    .collect::<Result<_, _>>()?
            })
        }

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_nested_clauses(tree: SyntaxTree) -> Result<DefSet, Error> {
    match (tree.token.rule, tree.children) {
        (Rule::nested_clauses, Some(SyntaxChildren::One(tree))) =>
            parse_def_set(*tree),

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_nested_definition(tree: Vec<SyntaxTree>) -> Result<Definition, Error> {
    let (maybe_using_ids, assign_ids, nested_clauses) = match vec_into_array_varied_length(tree)? {
        [ Some((Rule::using_ids, _, Some(using_ids)))
        , Some((Rule::assign_ids, _, Some(assign_ids)))
        , Some((Rule::nested_clauses, _, Some(SyntaxChildren::One(nested_clauses))))
        ] =>
            (Some(parse_names(using_ids.get_values())?), assign_ids, nested_clauses),

        [ Some((Rule::assign_ids, _, Some(assign_ids)))
        , Some((Rule::nested_clauses, _, Some(SyntaxChildren::One(nested_clauses))))
        , None
        ] =>
            (None, assign_ids, nested_clauses),

        nodes =>
            make_no_array_match_found_error(nodes)?,
    };

    Ok(Definition::NestedDefinition {
        using_ids: maybe_using_ids,
        identifiers: parse_names(assign_ids.get_values())?,
        def_set: parse_def_set(*nested_clauses)?,
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
