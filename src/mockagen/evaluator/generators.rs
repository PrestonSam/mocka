use std::iter::once;

use chrono::{Duration, NaiveDate};
use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng, Rng,
};

use crate::mockagen::{
    evaluator::model::EvaluationError,
    model::{DefNode, Definition, HigherOrderValue, MatchChildren, MatchExpr, MatchNode, MockagenError, NestedAssignNode, NestedDefNode, PrimitiveValue, TerminalAssignNode, TerminalDefNode, Value, WeightedValue}, utils::iterator::Transpose,
};

use super::model::{DefGen, Generator, MaybeWeighted, OutValue, WeightedT};


fn make_date_range_gen(low: NaiveDate, high: NaiveDate) -> Generator {
    let range_size = high.signed_duration_since(low).num_days();

    Box::new(move |_| {
        Ok(OutValue::NaiveDate(low + Duration::days(thread_rng().gen_range(0..=range_size))))
    })
}

fn make_integer_range_gen(low: i64, high: i64) -> Generator {
    // For negative inputs i.e. from -100 to -1000
    let (low, high) = match low < high {
        true => (low, high),
        false => (high, low)
    };

    Box::new(move |_| Ok(OutValue::I64(thread_rng().gen_range(low..=high))))
}

fn make_real_range_gen(low: f64, high: f64) -> Generator {
    // For negative inputs i.e. from -100 to -1000
    let (low, high) = match low < high {
        true => (low, high),
        false => (high, low)
    };

    Box::new(move |_| Ok(OutValue::F64(thread_rng().gen_range(low..=high))))
}

fn make_string_range_gen(low: i64, high: i64) -> Generator {
    Box::new(move |_| {
        let length = thread_rng().gen_range(low..=high) as usize;

        Ok(OutValue::String(Alphanumeric.sample_string(&mut thread_rng(), length)))
    })
}

fn make_literal_gen(literal: String) -> Generator {
    Box::new(move |_| Ok(OutValue::String(literal.clone())))
}

fn make_identifier_gen(identifier: String) -> Generator {
    Box::new(move |context|
        context.get(identifier.as_str())
            .map(|value| value.clone())
            .ok_or_else(|| MockagenError::from_eval_err(EvaluationError::MissingIdentifier(identifier.clone())))
    )
}

fn make_join_gen(values: Vec<Value>) -> Generator {
    let generators: Vec<Generator> = values.into_iter()
        .map(|value| get_generator_from_value(value))
        .collect();

    Box::new(move |context| {
        let output = generators.iter()
            .map(|gen| Ok::<_, MockagenError>(gen(context)?.to_string()))
            .collect::<Result<_, _>>()?;

        Ok(OutValue::String(output))
    })
}

fn get_generator_from_primitive_value(value: PrimitiveValue) -> Generator {
    match value {
        PrimitiveValue::DateRange(low, high) => make_date_range_gen(low, high),
        PrimitiveValue::IntegerRange(low, high) => make_integer_range_gen(low, high),
        PrimitiveValue::RealRange(low, high) => make_real_range_gen(low, high),
        PrimitiveValue::StringRange(low, high) => make_string_range_gen(low, high),
        PrimitiveValue::Literal(literal) => make_literal_gen(literal),
    }
}

fn get_generator_from_higher_order_value(value: HigherOrderValue) -> Generator {
    match value {
        HigherOrderValue::Identifier(identifier) =>
            make_identifier_gen(identifier),

        HigherOrderValue::Join(values) =>
            make_join_gen(values),
    }
}

fn get_generator_from_value(value: Value) -> Generator {
    match value {
        Value::PrimitiveValue(value) =>
            get_generator_from_primitive_value(value),

        Value::HigherOrderValue(value) =>
            get_generator_from_higher_order_value(value),
    }
}

fn determine_implicit_weightings<T>(maybe_weighteds: Vec<MaybeWeighted<T>>) -> Vec<WeightedT<T>> {
    let mut total_explicit_percentage = 0.0;
    let mut implicit_percentage_count = 0.0;

    for maybe_weighted in &maybe_weighteds {
        let MaybeWeighted { weight, value: _ } = maybe_weighted;

        match weight {
            Some(weight) => total_explicit_percentage += weight,
            None => implicit_percentage_count += 1.0,
        }
    }

    let implicit_weighting = (100.0 - total_explicit_percentage) / implicit_percentage_count;

    maybe_weighteds
        .into_iter()
        .map(|MaybeWeighted { weight, value }|
            WeightedT {
                weight: weight.unwrap_or(implicit_weighting),
                value,
            }
        )
        .collect()
}

fn into_explicit_weightings<T>(vals: Vec<T>) -> Vec<WeightedT<T>>
where MaybeWeighted<T>: From<T>
{
    let maybe_weighteds = vals.into_iter().map(MaybeWeighted::from).collect();
    determine_implicit_weightings(maybe_weighteds)
}

fn make_weighted_alternation_gen(weighted_ts: Vec<WeightedT<Generator>>) -> Generator {
    Box::new(move |context| {
        let mut chosen_percentage = thread_rng().gen_range(0.0..=100.0);

        let gen = weighted_ts
            .iter()
            .find_map(|WeightedT { weight, value: gen }| {
                chosen_percentage -= weight;

                Option::Some(gen)
                    .filter(|_| chosen_percentage <= 0.0)
            })
            .expect("Random number generator managed to generate number outside of percentage range");

        gen(context)
    })
}

fn get_alternation_gen_from_weighted_values(weighted_values: Vec<WeightedValue>) -> Generator {
    let weighted_values = into_explicit_weightings(weighted_values)
        .into_iter()
        .map(|WeightedT { weight, value }| WeightedT { weight, value: get_generator_from_value(value.value) })
        .collect();

    make_weighted_alternation_gen(weighted_values)
}

fn get_dependencies_from_values(values: &[&Value]) -> Vec<String> {
    values.iter()
        .filter_map(|value| match value {
            Value::HigherOrderValue(hov) => Some(hov),
            _ => None,
        }).flat_map(|hov|
            match hov {
                HigherOrderValue::Identifier(id) =>
                    vec![ id.clone() ],

                HigherOrderValue::Join(values) => 
                    get_dependencies_from_values(values.iter().collect::<Vec<_>>().as_slice()),
            }
        ).collect()
}

fn unstack_def_node(def_node: NestedDefNode) -> Vec<TerminalDefNode> {
    match def_node {
        DefNode::Match(children) => {
            let children = match children {
                MatchChildren::Exhaustive(children) => children,
                MatchChildren::Wildcard { .. } => todo!("Implement unstacking for wildcard matchers"),
            };

            let (matchers_by_child, unstacked_grandchildren): (Vec<_>, Vec<_>) = children.into_iter()
                .map(|node| (node.matchers, unstack_def_node(node.children)))
                .unzip();
            let grandchildren_by_depth = unstacked_grandchildren.into_iter().transpose();

            grandchildren_by_depth.into_iter()
                .map(|grandchildren_defnodes_at_depth| {
                    let match_nodes: Vec<_> = grandchildren_defnodes_at_depth.into_iter()
                        .zip(matchers_by_child.clone().into_iter())
                        .map(|(children, matchers)| MatchNode::<TerminalAssignNode> { matchers, children })
                        .collect();
                    DefNode::Match(MatchChildren::Exhaustive(match_nodes))
                })
                .collect()
        }
        DefNode::Assign(children) => {

            fn unpack_assign_nodes(children: Vec<NestedAssignNode>) -> Vec<TerminalDefNode> {

                let (terminal_nodes, maybe_grandchildren_by_child): (Vec<TerminalAssignNode>, Vec<Option<Vec<NestedAssignNode>>>) = children.into_iter()
                    .map(|node| (TerminalAssignNode { weight: node.weight, values: node.values }, node.children))
                    .unzip();

                let maybe_grandchildren_by_child: Option<Vec<_>> = maybe_grandchildren_by_child.into_iter().collect();

                match maybe_grandchildren_by_child {
                    Some(grandchildren_by_child) => {
                        let grandchildren_defnodes_by_depth = grandchildren_by_child.into_iter()
                            .map(|grandchildren| unpack_assign_nodes(grandchildren))
                            .transpose();

                        dbg!(&grandchildren_defnodes_by_depth);

                        fn get_match_exprs_by_child(terminal_nodes: &Vec<TerminalAssignNode>) -> Vec<Vec<MatchExpr>> {
                            terminal_nodes.iter()
                                .map(|child| child.make_match_exprs())
                                .collect()
                        }
                        
                        let wrapped_grandchildren_defnodes_by_depth: Vec<_> = grandchildren_defnodes_by_depth.into_iter()
                            .map(|grandchildren_defnodes_at_depth| {
                                let match_nodes: Vec<_> = grandchildren_defnodes_at_depth.into_iter()
                                    .zip(get_match_exprs_by_child(&terminal_nodes).into_iter())
                                    .map(|(children, matchers)| MatchNode::<TerminalAssignNode> { matchers, children })
                                    .collect();

                                TerminalDefNode::Match(MatchChildren::Exhaustive(match_nodes))
                            })
                            .collect();


                        once(TerminalDefNode::Assign(terminal_nodes))
                            .chain(wrapped_grandchildren_defnodes_by_depth)
                            .collect()
                    }
                    None =>
                        vec![
                            TerminalDefNode::Assign(terminal_nodes)
                        ]
                }
            }

            unpack_assign_nodes(children)
        },
    }
}

fn make_terminal_def_node_gen<'a>(match_ids: &Vec<&String>, def_node: TerminalDefNode, depth: usize) -> Generator {
    match def_node {
        DefNode::Assign(children) => {
            let weighted_generators = into_explicit_weightings(children)
                .into_iter()
                .map(|WeightedT { weight, value }| {
                    WeightedT { weight, value: get_alternation_gen_from_weighted_values(value.values) }
                })
                .collect::<Vec<_>>();

            make_weighted_alternation_gen(weighted_generators)
        }
        DefNode::Match(children) => {
            dbg!(&match_ids, &children);

            let this_id = match_ids.get(depth)
                .map(|id| (*id).clone())
                .expect(&format!("DefNode::Match didn't have a paired id {:?}", children));

            let children = match children {
                MatchChildren::Exhaustive(children) => children,
                MatchChildren::Wildcard { .. } => todo!("Implement evaluation for wildcard matchers"),
            };

            let child_generators: Vec<_> = children.into_iter()
                .map(|child| (child.matchers, make_terminal_def_node_gen(match_ids, child.children, depth + 1)))
                .collect();

            Box::new(move |context| {
                let match_value = context.get(this_id.as_str())
                    .ok_or_else(|| MockagenError::from_eval_err(EvaluationError::MissingIdentifier(this_id.clone())))?;

                child_generators.iter()
                    .find(|(matchers, _)| matchers.iter().any(|matcher| matcher.is_match(match_value)))
                    .map(|(_, gen)| gen(context))
                    .unwrap_or_else(|| Err(MockagenError::from_eval_err(EvaluationError::NoMatchBranchForValue(match_value.clone()))))
            })
        }
    }
}


pub fn make_definition_gens(definition: Definition) -> Vec<DefGen> {
    match definition {
        Definition::NestedDefinition { using_ids, identifiers, nested_def_set } => {
            let bind_ids = identifiers.clone();
            let using_ids = using_ids.unwrap_or_else(|| vec![]);

            let ref_ids_by_assigned_id: Vec<Vec<_>> = (0..identifiers.len())
                .map(|length|
                    using_ids.iter()
                        .chain(identifiers.iter().take(length))
                        .collect())
                .collect();

            let unstacked_def_nodes = unstack_def_node(nested_def_set);

            dbg!(ref_ids_by_assigned_id.len(), unstacked_def_nodes.len(), bind_ids.len());

            ref_ids_by_assigned_id.into_iter()
                .zip(unstacked_def_nodes)
                .zip(bind_ids)
                .map(|((ref_ids, def_node), id)| {
                    dbg!(&ref_ids, &def_node, &id);
                    DefGen {
                        id,
                        gen: make_terminal_def_node_gen(&ref_ids, def_node, 0),
                        dependencies: ref_ids.into_iter().map(|s| s.to_string()).collect(),
                    }
                })
                .collect()
        }
        Definition::SingleDefinition { identifier, values: weighted_values } => {
            let values: Vec<_> = weighted_values.iter()
                .map(|wval| &wval.value)
                .collect();

            let dependencies = get_dependencies_from_values(&values);

            vec![
                DefGen {
                    id: identifier,
                    dependencies,
                    gen: get_alternation_gen_from_weighted_values(weighted_values),
                }
            ]
        }
    }
}
