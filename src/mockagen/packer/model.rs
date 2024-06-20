use std::fmt::Debug;
use chrono::NaiveDate;

use crate::mockagen::{evaluator::model::OutValue, parser::Rule};


// TODO expand the supported expressions in future
#[derive(Debug, Clone)]
pub enum MatchExpr {
    Literal(String),
    Any,
}

impl From<&Value> for MatchExpr {
    fn from(value: &Value) -> Self {
        match value {
            Value::PrimitiveValue(PrimitiveValue::Literal(literal)) => Self::Literal(literal.clone()),
            _ => todo!("Implement support for other Values / MatchExprs")
        }
    }
}

impl MatchExpr {
    // FIXME this feels like coupling between the packer and evaluator modules.
    // Evaluator can know MatchExpr, but MatchExpr should not know OutValue.
    pub fn is_match(&self, value: &OutValue) -> bool {
        match (self, value) {
            (MatchExpr::Any, _) =>
                true,

            (MatchExpr::Literal(match_value), OutValue::String(found_value)) =>
                *match_value == *found_value, // TODO is this the idiomatic way to do this?

            _ =>
                false
        }
    }
}

#[derive(Debug)]
pub enum PrimitiveValue {
    DateRange(NaiveDate, NaiveDate),
    Literal(String),
    IntegerRange(i64, i64),
    StringRange(i64, i64),
    RealRange(f64, f64),
}

#[derive(Debug)]
pub enum HigherOrderValue {
    Join(Vec<Value>),
    Identifier(String),
}

#[derive(Debug)]
pub enum Value {
    PrimitiveValue(PrimitiveValue),
    HigherOrderValue(HigherOrderValue),
}

pub type Weight = f64;

#[derive(Debug)]
pub struct WeightedValue {
    pub weight: Option<Weight>,
    pub value: Value,
}

#[derive(Debug)]
pub struct MatchNode<T> {
    pub matchers: Vec<MatchExpr>,
    pub children: DefNode<T>,
}

#[derive(Debug)]
pub struct WildcardNode<T> {
    pub children: DefNode<T>,
}

#[derive(Debug)]
pub struct NestedAssignNode {
    pub weight: Option<Weight>,
    pub values: Vec<WeightedValue>,
    pub children: Option<Vec<NestedAssignNode>>
}

#[derive(Debug)]
pub struct TerminalAssignNode {
    pub weight: Option<Weight>,
    pub values: Vec<WeightedValue>,
}

impl TerminalAssignNode {
    pub fn make_match_exprs(&self) -> Vec<MatchExpr> {
        self.values
            .iter()
            .map(|wv| MatchExpr::from(&wv.value))
            .collect()
    }
}


#[derive(Debug)]
pub enum MatchChildren<T> {
    Exhaustive(Vec<MatchNode<T>>),
    Wildcard { children: Vec<MatchNode<T>>, wildcard_child: Box<WildcardNode<T>> },
}


/// Represents a fork in a given definition tree.
/// Each child of the fork must be of the same type - either Match, MatchWithWildcard or Assign
#[derive(Debug)]
pub enum DefNode<T> { // TODO Should I restrict what T can be, here?
    Match(MatchChildren<T>),
    Assign(Vec<T>),
}

pub type NestedDefNode = DefNode<NestedAssignNode>;

pub type TerminalDefNode = DefNode<TerminalAssignNode>;


#[derive(Debug)]
pub enum Definition {
    SingleDefinition {
        identifier: String,
        values: Vec<WeightedValue>
    },
    NestedDefinition {
        using_ids: Option<Vec<String>>,
        identifiers: Vec<String>,
        nested_def_set: NestedDefNode,
    },
}

#[derive(Debug)]
pub enum Statement {
    Include(Vec<String>),
    Definition(Definition),
}


#[derive(Debug)]
pub enum PackingErrorVariant {
    SyntaxUnhandledTreeShape(String),
    SyntaxChildrenArrayCastError(Vec<Option<(Rule, String, Option<String>)>>), // TODO This could probably use a type alias
    SyntaxNodeCountMismatch(Vec<Option<(Rule, String, Option<String>)>>), // TODO This could probably use a type alias
    ParseIntError(core::num::ParseIntError),
    ParseFloatError(core::num::ParseFloatError),
    DateParseError(chrono::ParseError),
}

impl From<core::num::ParseIntError> for PackingErrorVariant {
    fn from(value: core::num::ParseIntError) -> Self {
        PackingErrorVariant::ParseIntError(value)
    }
}

impl From<core::num::ParseFloatError> for PackingErrorVariant {
    fn from(value: core::num::ParseFloatError) -> Self {
        PackingErrorVariant::ParseFloatError(value)
    }
}

impl From<chrono::ParseError> for PackingErrorVariant {
    fn from(value: chrono::ParseError) -> Self {
        PackingErrorVariant::DateParseError(value)
    }
}

pub type PackingError =
    crate::utils::packing::PackingError<PackingErrorVariant, Rule>;

pub type SyntaxTree<'a> =
    crate::utils::packing::SyntaxTree<'a, Rule>;

pub type SyntaxChildren<'a> =
    crate::utils::packing::SyntaxChildren<'a, Rule>;
