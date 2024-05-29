use std::fmt::Debug;

use chrono::NaiveDate;
use pest::{iterators::Pair, Span};

use super::{evaluator::model::{EvaluationError, OutValue}, parser::Rule};


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

#[derive(Clone)]
pub struct Providence<'a> {
    pub span: Span<'a>,
    pub src: &'a str,
}

fn trunc(str: &str, len: usize) -> String {
    if str.len() <= len {
        format!("{:?}", str)
    } else {
        format!("{:?}..", format!("{:.*}", len, str))
    }
}

impl <'a>core::fmt::Debug for Providence<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (line, column) = self.span.start_pos().line_col();

        write!(f, "At {}:{}, source code: {}", line, column, trunc(self.src, 40))
    }
}

#[derive(Debug)]
pub enum PackingErrorVariant { // At some point I'll have to break this out into sub-errors
    SyntaxUnhandledTreeShape(String),
    SyntaxChildrenArrayCastError(Vec<Option<(Rule, String, Option<String>)>>), // TODO This could probably use a type alias
    SyntaxNodeCountMismatch(Vec<Option<(Rule, String, Option<String>)>>), // TODO This could probably use a type alias
    ParseIntError(core::num::ParseIntError),
    ParseFloatError(core::num::ParseFloatError),
    DateParseError(chrono::ParseError),
}

#[derive(Debug)]
enum PackingErrorContext {
    Providence(String),
    Rule(Rule),
}

#[derive(Debug) ]
pub struct PackingError {
    error: PackingErrorVariant,
    context: Vec<PackingErrorContext>,
}

impl PackingError {
    pub fn new(error: PackingErrorVariant) -> Self {
        PackingError {
            error,
            context: vec![],
        }
    }

    pub fn with_providence(mut self, providence: Providence<'_>) -> Self {
        self.context.push(PackingErrorContext::Providence(format!("{:?}", providence)));
        self
    }

    pub fn with_rule(mut self, rule: Rule) -> Self {
        self.context.push(PackingErrorContext::Rule(rule));
        self
    }
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

#[derive(Debug)]
pub struct AnnotatedPackingError {
    pub error: PackingErrorVariant,
    pub providence: String,
}

#[derive(Debug)]
pub enum MockagenErrorVariant {
    PackingError(PackingError),
    ParsingError(Box<pest::error::Error<Rule>>),
    EvaluationError(EvaluationError),
}

#[derive(Debug)]
pub struct MockagenError { // TODO do I need this?
    error: MockagenErrorVariant,
}

impl MockagenError {
    pub fn from_parsing_err(error: pest::error::Error<Rule>) -> Self {
        MockagenError {
            error: MockagenErrorVariant::ParsingError(Box::from(error))
        }
    }

    pub fn from_packing_err(error: PackingError) -> Self {
        MockagenError {
            error: MockagenErrorVariant::PackingError(error)
        }
    }

    pub fn from_eval_err(error: EvaluationError) -> Self {
        MockagenError {
            error: MockagenErrorVariant::EvaluationError(error)
        }
    }
}

impl From<PackingError> for MockagenErrorVariant {
    fn from(value: PackingError) -> Self {
        MockagenErrorVariant::PackingError(value)
    }
}

impl From<pest::error::Error<Rule>> for MockagenErrorVariant {
    fn from(value: pest::error::Error<Rule>) -> Self {
        MockagenErrorVariant::ParsingError(Box::from(value))
    }
}

impl From<EvaluationError> for MockagenErrorVariant {
    fn from(value: EvaluationError) -> Self {
        MockagenErrorVariant::EvaluationError(value)
    }
}

#[derive(Debug, Clone)]
pub struct SyntaxToken<'a> {
    pub rule: Rule,
    pub providence: Providence<'a>,
}

#[derive(Clone)]
pub struct SyntaxTree<'a> {
    pub token: SyntaxToken<'a>,
    pub children: Option<SyntaxChildren<'a>>,
}

#[derive(Clone)]
pub enum SyntaxChildren<'a> {
    One(Box<SyntaxTree<'a>>),
    Many(Vec<SyntaxTree<'a>>),
}

impl <'a>SyntaxChildren<'a> {
    pub fn get_values(self) -> Vec<SyntaxTree<'a>> {
        match self {
            SyntaxChildren::One(child) => vec![*child],
            SyntaxChildren::Many(children) => children,
        }
    }

    pub fn get_values_iter(self) -> impl Iterator<Item = SyntaxTree<'a>> {
        self.get_values().into_iter()
    }
}

impl <'a>Debug for SyntaxChildren<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyntaxChildren::One(val) =>
                f.debug_list()
                    .entries(vec![ val ])
                    .finish(),

            SyntaxChildren::Many(vals) => {
                let rules = vals
                    .iter()
                    .map(|child| child.token.rule);

                f.debug_list()
                    .entries(rules)
                    .finish()
            }
        }
    }
}

impl <'a>From<Pair<'a, Rule>> for SyntaxTree<'a> {
    fn from(pair: Pair<'a, Rule>) -> Self {
        let rule = pair.as_rule();
        let providence = Providence { src: pair.as_str(), span: pair.as_span() };
        let token = SyntaxToken { rule, providence };

        let inner_without_tabs: Vec<_> = pair.into_inner()
            .filter(|pair| pair.as_rule() != Rule::TAB)
            .collect();

        let mut inner = inner_without_tabs.into_iter();

        let children = match inner.len() {
            0 => None,
            1 => {
                let child = inner.next()
                    .expect("Perhaps there's a better way to do this");

                Some(SyntaxChildren::One(SyntaxTree::from(child).into()))
            },
            _ => {
                let children = inner.map(SyntaxTree::from).collect();

                Some(SyntaxChildren::Many(children))
            }
        };

        SyntaxTree { token, children }
    }
}

impl <'a>From<(Rule, Providence<'a>, Option<SyntaxChildren<'a>>)> for SyntaxTree<'a> {
    fn from((rule, providence, children): (Rule, Providence<'a>, Option<SyntaxChildren<'a>>)) -> Self {
        SyntaxTree { token: SyntaxToken { rule, providence }, children }
    }
}

impl <'a>Debug for SyntaxTree<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyntaxTree { token, children: None } =>
                f.debug_struct("TreeLeaf")
                    .field("token", token)
                    .finish(),

            SyntaxTree { token, children: Some(children) } =>
                f.debug_struct("TreeNode")
                    .field("token", token)
                    .field("children", children)
                    .finish()
        }
    }
}
