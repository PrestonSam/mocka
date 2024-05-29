use pest::iterators::Pairs;

use super::{
    model::{Definition, HigherOrderValue, MatchChildren, MatchExpr, MatchNode, MockagenError, NestedAssignNode, NestedDefNode, PackingError, PackingErrorVariant, PrimitiveValue, Providence, Statement, SyntaxChildren, SyntaxTree, Value, Weight, WeightedValue, WildcardNode}, parser::Rule, utils::{
        error::{make_no_array_match_found_error, make_tree_shape_error},
        unpackers::{unpack_range, vec_into_array_varied_length}
    }
};


// TODO maybe move this someplace else
trait PackingResult {
    fn with_rule(self, rule: Rule) -> Self;
}

impl<T> PackingResult for Result<T, PackingError> {
    fn with_rule(self, rule: Rule) -> Self
    {
        self.map_err(|err| err.with_rule(rule))
    }
}


fn unpack_string_literal(tree: SyntaxTree) -> Result<String, PackingError> {
    match (tree.token.rule, tree.children) {
        (Rule::STRING_LITERAL, Some(SyntaxChildren::One(string_content))) =>
            Ok(string_content.token.providence.src.to_string()),

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_primitive_value(tree: SyntaxTree) -> Result<PrimitiveValue, PackingError> {
    match (tree.token.rule, tree.children) {
        (Rule::integer_value, Some(children)) =>
            unpack_range(Rule::INTEGER_LITERAL, PrimitiveValue::IntegerRange, children.get_values())
                .with_rule(Rule::integer_value),

        (Rule::real_value, Some(children)) =>
            unpack_range(Rule::REAL_LITERAL, PrimitiveValue::RealRange, children.get_values())
                .with_rule(Rule::real_value),

        (Rule::string_value, Some(children)) =>
            unpack_range(Rule::INTEGER_LITERAL, PrimitiveValue::StringRange, children.get_values())
                .with_rule(Rule::string_value),

        (Rule::timestamp_date_value, Some(children)) =>
            unpack_range(Rule::DATE_LITERAL, PrimitiveValue::DateRange, children.get_values())
                .with_rule(Rule::timestamp_date_value),

        (Rule::literal_value, Some(SyntaxChildren::One(string_literal))) => {
            let literal = unpack_string_literal(*string_literal)
                .with_rule(Rule::literal_value)?;

            Ok(PrimitiveValue::Literal(literal))
        }

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_higher_order_value(tree: SyntaxTree) -> Result<HigherOrderValue, PackingError> {
    match (tree.token.rule, tree.children) {
        (Rule::identifier_value, Some(SyntaxChildren::One(identifier))) =>
            Ok(HigherOrderValue::Identifier(identifier.token.providence.src.to_string())),

        (Rule::join_value, Some(children)) =>
            Ok(HigherOrderValue::Join(children.get_values_iter()
                .map(|tree| match (tree.token.rule, tree.children) {
                    (Rule::value, Some(SyntaxChildren::One(value))) =>
                        parse_value_subtype(*value),
                    
                    (rule, children) =>
                        make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
                })
                .collect::<Result<Vec<_>, PackingError>>()
                .with_rule(Rule::join_value)?)
            ),

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_value_subtype(tree: SyntaxTree) -> Result<Value, PackingError> {
    match (tree.token.rule, tree.children) {
        (Rule::primitive_value, Some(SyntaxChildren::One(child))) => {
            let value = parse_primitive_value(*child)
                .with_rule(Rule::primitive_value)?;

            Ok(Value::PrimitiveValue(value))
        }
        
        (Rule::higher_order_value, Some(SyntaxChildren::One(child))) => {
            let value = parse_higher_order_value(*child)
                .with_rule(Rule::higher_order_value)?;

            Ok(Value::HigherOrderValue(value))
        }

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_match_expr(tree: SyntaxTree) -> Result<MatchExpr, PackingError> {
    match (tree.token.rule, tree.children) {
        (Rule::literal_value, Some(SyntaxChildren::One(string_literal))) => {
            let literal = unpack_string_literal(*string_literal)
                .with_rule(Rule::literal_value)?;

            Ok(MatchExpr::Literal(literal))
        }

        (Rule::any_value, None) =>
            Ok(MatchExpr::Any),

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_matchers(tree: SyntaxTree) -> Result<Vec<MatchExpr>, PackingError> {
    match (tree.token.rule, tree.children) {
        (Rule::match_expr, Some(SyntaxChildren::One(child))) => {
            let match_expr = parse_match_expr(*child)
                .with_rule(Rule::match_expr)?;

            Ok(vec![ match_expr ])
        }

        (Rule::matcher_set, Some(children)) =>
            children.get_values_iter()
                .map(parse_match_expr)
                .collect::<Result<_, _>>()
                .with_rule(Rule::matcher_set),

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_weighting(tree: SyntaxTree) -> Result<f64, PackingError> {
    let providence = tree.token.providence;
    let percentage_str = providence.src;

    percentage_str.parse::<f64>()
        .map_err(|err| PackingError::new(PackingErrorVariant::ParseFloatError(err)).with_providence(providence))
}

fn parse_maybe_weighted(tree: Vec<SyntaxTree>) -> Result<(Option<Weight>, SyntaxTree), PackingError> {
    match vec_into_array_varied_length(tree)? {
        [ Some((Rule::WEIGHT, _, Some(SyntaxChildren::One(weighting))))
        , Some(value)
        ] => {
            let weighting = parse_weighting(*weighting)
                .with_rule(Rule::WEIGHT)?;

            Ok((Some(weighting), SyntaxTree::from(value)))
        }
        
        [ Some(value)
        , None
        ] => 
            Ok((None, SyntaxTree::from(value))),

        nodes =>
            make_no_array_match_found_error(nodes),
    }
}

fn parse_weighted_value(tree: SyntaxTree) -> Result<WeightedValue, PackingError> {
    match (tree.token.rule, tree.children) {
        (Rule::weighted_value, Some(children)) => {
            let (maybe_weight, value_tree) = parse_maybe_weighted(children.get_values())
                .with_rule(Rule::weighted_value)?;

            match (value_tree.token.rule, value_tree.children) {
                (Rule::value, Some(SyntaxChildren::One(value))) => {
                    let value = parse_value_subtype(*value)
                        .with_rule(Rule::value)?;

                    Ok(WeightedValue {
                        weight: maybe_weight,
                        value
                    })
                }

                (rule, children) =>
                    make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children)))
                        .with_rule(Rule::weighted_value),
            }
        }

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_weighted_value_or_set(tree: SyntaxTree) -> Result<Vec<WeightedValue>, PackingError> {
    match (tree.token.rule, tree.children) {
        (Rule::value, Some(SyntaxChildren::One(value_tree))) => {
            let value = parse_value_subtype(*value_tree)
                .map_err(|err| err.with_rule(Rule::value))?;

            Ok(vec![ WeightedValue { weight: None, value } ])
        }

        (Rule::value_set, Some(children)) =>
            children.get_values_iter()
                .map(parse_weighted_value)
                .collect::<Result<_, _>>()
                .with_rule(Rule::value_set),

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_values(tree: SyntaxTree) -> Result<Vec<WeightedValue>, PackingError> {
    match (tree.token.rule, tree.children) {
        (Rule::values, Some(SyntaxChildren::One(wval_or_set))) =>
            parse_weighted_value_or_set(*wval_or_set).with_rule(Rule::values),

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_single_definition(nodes: Vec<SyntaxTree>) -> Result<Definition, PackingError> {
    match vec_into_array_varied_length(nodes)? {
        [ Some((Rule::IDENTIFIER, Providence { src: id, .. }, None))
        , Some(weighted_values)
        ] => {
            let values = parse_weighted_value_or_set(SyntaxTree::from(weighted_values))
                .with_rule(Rule::weighted_values)?;

            Ok(Definition::SingleDefinition {
                identifier: id.to_string(),
                values
            })
        }

        nodes =>
            make_no_array_match_found_error(nodes),
    }
}

fn parse_names(trees: Vec<SyntaxTree>) -> Result<Vec<String>, PackingError> {
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

fn parse_match_clause(tree: SyntaxTree) -> Result<MatchNode<NestedAssignNode>, PackingError> {
    match (tree.token.rule, tree.children) {
        (Rule::match_clause, Some(children)) =>
            match vec_into_array_varied_length(children.get_values())? {
                [ Some((Rule::matchers, _, Some(SyntaxChildren::One(matchers))))
                , Some((Rule::nested_clauses, _, Some(SyntaxChildren::One(tree)) ))
                ] => {
                    let matchers = parse_matchers(*matchers)
                        .with_rule(Rule::matchers)?;

                    let children = parse_def_set(*tree)
                        .with_rule(Rule::nested_clauses)?;

                    Ok(MatchNode { matchers, children, })
                }

                nodes =>
                    make_no_array_match_found_error(nodes),
            }

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_assign_clause(tree: SyntaxTree) -> Result<NestedAssignNode, PackingError> {
    match (tree.token.rule, tree.children) {
        (Rule::assign_clause, Some(children)) => {
            let (weighted_value_trees, maybe_children) = match vec_into_array_varied_length(children.get_values()).with_rule(Rule::assign_clause)? {
                [ Some((Rule::weighted_values, _, Some(weighted_value_trees)))
                , Some((Rule::assign_clauses, _, Some(sub_assignment_trees)))
                ] => {
                    let children = sub_assignment_trees.get_values_iter()
                        .map(parse_assign_clause)
                        .collect::<Result<_, _>>().with_rule(Rule::assign_clauses).with_rule(Rule::assign_clause)?;

                    (weighted_value_trees, Some(children))
                }

                [ Some((Rule::weighted_values, _, Some(weighted_value_trees)))
                , None
                ] => 
                    (weighted_value_trees, None),

                nodes =>
                    make_no_array_match_found_error(nodes)?,
            };

            let (maybe_weight, values_tree) = parse_maybe_weighted(weighted_value_trees.get_values())
                .with_rule(Rule::weighted_values).with_rule(Rule::assign_clause)?;

            let values = parse_values(values_tree)
                .with_rule(Rule::weighted_values).with_rule(Rule::assign_clause)?;

            Ok(NestedAssignNode {
                weight: maybe_weight,
                values,
                children: maybe_children,
            })
        }
            

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_def_set(tree: SyntaxTree) -> Result<NestedDefNode, PackingError> {
    match (tree.token.rule, tree.children) {
        (Rule::match_clauses, Some(children)) => {
            Ok(NestedDefNode::Match(MatchChildren::Exhaustive(
                children.get_values_iter()
                    .map(parse_match_clause)
                    .collect::<Result<_, _>>()
                    .with_rule(Rule::match_clauses)?
            )))
        }

        (Rule::match_clauses_with_wildcard, Some(children)) => {
            match vec_into_array_varied_length(children.get_values()).with_rule(Rule::match_clauses_with_wildcard).with_rule(Rule::match_clauses)? {
                [ Some((Rule::match_clauses, _, Some(match_clause_children)))
                , Some((Rule::wildcard_clause, _, Some(SyntaxChildren::One(wildcard_nested_clauses))))
                ] => {
                    let nodes = match_clause_children.get_values_iter()
                        .map(parse_match_clause)
                        .collect::<Result<_, _>>()
                        .with_rule(Rule::match_clauses).with_rule(Rule::match_clauses_with_wildcard)?;

                    let wildcard_node = WildcardNode {
                        children: parse_nested_clauses(*wildcard_nested_clauses).with_rule(Rule::wildcard_clause).with_rule(Rule::match_clauses_with_wildcard)?
                    };

                    Ok(NestedDefNode::Match(MatchChildren::Wildcard {
                        children: nodes,
                        wildcard_child: Box::new(wildcard_node)
                    }))
                }

                nodes =>
                    make_no_array_match_found_error(nodes).with_rule(Rule::match_clauses_with_wildcard),
            }
        }

        (Rule::assign_clauses, Some(children)) => {
            Ok(NestedDefNode::Assign(
                children.get_values_iter()
                    .map(parse_assign_clause)
                    .collect::<Result<_, _>>()
                    .with_rule(Rule::assign_clauses)?
            ))
        }

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}

fn parse_nested_clauses(tree: SyntaxTree) -> Result<NestedDefNode, PackingError> {
    match (tree.token.rule, tree.children) {
        (Rule::nested_clauses, Some(SyntaxChildren::One(tree))) =>
            parse_def_set(*tree).with_rule(Rule::nested_clauses),

        (rule, children) =>
            make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children))),
    }
}


fn parse_nested_definition(tree: Vec<SyntaxTree>) -> Result<Definition, PackingError> {
    let (maybe_using_ids, assign_ids, nested_clauses) = match vec_into_array_varied_length(tree)? {
        [ Some((Rule::using_ids, _, Some(using_ids)))
        , Some((Rule::assign_ids, _, Some(assign_ids)))
        , Some((Rule::nested_clauses, _, Some(SyntaxChildren::One(nested_clauses))))
        ] =>
            (Some(parse_names(using_ids.get_values()).with_rule(Rule::using_ids)?), assign_ids, nested_clauses),

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
        nested_def_set: parse_def_set(*nested_clauses)?,
    })
}

pub fn pack_mockagen(pairs: Pairs<'_, Rule>) -> Result<Vec<Statement>, MockagenError> {
    pairs.map(SyntaxTree::from)
        .map_while(|tree| {
            match (tree.token.rule, tree.children) {
                (Rule::include_statement, Some(children)) =>
                    Some(children.get_values_iter()
                        .map(unpack_string_literal)
                        .collect::<Result<_, _>>()
                        .with_rule(Rule::include_statement)
                        .map(Statement::Include)
                    ),
                
                (Rule::single_definition, Some(children)) =>
                    Some(parse_single_definition(children.get_values()).with_rule(Rule::single_definition).map(Statement::Definition)),

                (Rule::nested_definition, Some(children)) =>
                    Some(parse_nested_definition(children.get_values()).with_rule(Rule::nested_definition).map(Statement::Definition)),
                
                (Rule::EOI, None) =>
                    None,

                (rule, children) =>
                    Some(make_tree_shape_error(SyntaxTree::from((rule, tree.token.providence, children)))),
            }
        })
        .collect::<Result<_, _>>()
        .map_err(|err| MockagenError::from_packing_err(err))
}
