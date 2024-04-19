use chrono::NaiveDate;
use pest::{iterators::Pair, Span};

use super::{evaluator::model::EvaluationError, parser::Rule};


#[derive(Debug)]
pub enum Value {
    DateRange(NaiveDate, NaiveDate),
    Literal(String),
    IntegerRange(i64, i64),
    StringRange(i64, i64),
    RealRange(f64, f64),
    Join(Vec<Value>),

    // Are these actually values? They can't be generated the way the others can.
    // Perhaps they belong in a different enum
    Any,
    Identifier(String),
}

pub type Weight = f64;

#[derive(Debug)]
pub struct WeightedValue {
    pub weight: Option<Weight>,
    pub value: Value,
}

#[derive(Debug)]
pub enum DefNode {
    Match { matchers: Vec<Value>, children: Option<Vec<DefNode>>},
    Assign {
        weight: Option<Weight>,
        values: Vec<WeightedValue>,
        children: Option<Vec<DefNode>>
    }
}

#[derive(Debug)]
pub enum Definition {
    SingleDefinition {
        identifier: String,
        values: Vec<WeightedValue>
    },
    NestedDefinition {
        using_ids: Option<Vec<String>>,
        identifiers: Vec<String>,
        branches: Vec<DefNode>,
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
pub enum PackingError<'a> { // At some point I'll have to break this out into sub-errors
    SyntaxUnhandledTreeShape(SyntaxTree<'a>),
    SyntaxChildrenArrayCastError(Vec<Option<(Rule, Providence<'a>, Option<SyntaxChildren<'a>>)>>), // TODO This could probably use a type alias
    SyntaxNodeCountMismatch(Vec<Option<(Rule, Providence<'a>, Option<SyntaxChildren<'a>>)>>), // TODO This could probably use a type alias
    ParseIntError(core::num::ParseIntError),
    ParseRealError(core::num::ParseFloatError),
    DateParseError(chrono::ParseError),
}

impl From<core::num::ParseIntError> for PackingError<'_> {
    fn from(value: core::num::ParseIntError) -> Self {
        PackingError::ParseIntError(value)
    }
}

impl From<core::num::ParseFloatError> for PackingError<'_> {
    fn from(value: core::num::ParseFloatError) -> Self {
        PackingError::ParseRealError(value)
    }
}

impl From<chrono::ParseError> for PackingError<'_> {
    fn from(value: chrono::ParseError) -> Self {
        PackingError::DateParseError(value)
    }
}

#[derive(Debug)]
pub struct AnnotatedPackingError<'a> {
    pub error: PackingError<'a>,
    pub providence: Providence<'a>,
}

#[derive(Debug)]
pub enum Error<'a> { // TODO see about removing the lifetime specifier as it's more trouble than it's worth
    PackingError(AnnotatedPackingError<'a>),
    ParsingError(pest::error::Error<Rule>),
    EvaluationError(EvaluationError),
}

impl <'a>From<AnnotatedPackingError<'a>> for Error<'a> {
    fn from(value: AnnotatedPackingError<'a>) -> Self {
        Error::PackingError(value)
    }
}

impl From<pest::error::Error<Rule>> for Error<'_> {
    fn from(value: pest::error::Error<Rule>) -> Self {
        Error::ParsingError(value)
    }
}

impl From<EvaluationError> for Error<'_> {
    fn from(value: EvaluationError) -> Self {
        Error::EvaluationError(value)
    }
}




// Trying to figure out how to simplify the process of representing the AST

#[derive(Debug, Clone)]
pub struct SyntaxToken<'a> {
    pub rule: Rule,
    pub providence: Providence<'a>,
}

#[derive(Debug, Clone)]
pub struct SyntaxTree<'a> {
    pub token: SyntaxToken<'a>,
    pub children: Option<SyntaxChildren<'a>>,
}

#[derive(Debug, Clone)]
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

impl <'a>From<Pair<'a, Rule>> for SyntaxTree<'a> {
    fn from(pair: Pair<'a, Rule>) -> Self {
        let rule = pair.as_rule();
        let providence = Providence { src: pair.as_str(), span: pair.as_span() };
        let token = SyntaxToken { rule, providence };

        let mut inner = pair.into_inner();

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
