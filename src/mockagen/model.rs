use chrono::NaiveDate;
use pest::{iterators::{Pair, Pairs}, Span};

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

pub struct AnnotatedPair<'a> {
    pub providence: Providence<'a>,
    pub pair: Pair<'a, Rule>,
}

impl <'a>From<Pair<'a, Rule>> for AnnotatedPair<'a> {
    fn from(pair: Pair<'a, Rule>) -> Self {
        AnnotatedPair {
            providence: Providence {
                span: pair.as_span(),
                src: pair.as_str()
            },
            pair,
        }
    }
}

pub struct AnnotatedPairs<'a> {
    pub providence: Providence<'a>,
    pub pairs: Pairs<'a, Rule>,
}

impl <'a>From<Pair<'a, Rule>> for AnnotatedPairs<'a> {
    fn from(value: Pair<'a, Rule>) -> Self {
        AnnotatedPairs {
            providence: Providence {
                span: value.as_span(),
                src: value.as_str()
            },
            pairs: value.into_inner(),
        }
    }
}

impl <'a>core::fmt::Debug for AnnotatedPairs<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rules = self.pairs
            .clone()
            .map(|pair| pair.as_rule())
            .collect::<Vec<_>>();

        write!(f, "Pairs {{ {:?}, child rules: {:?} }}", self.providence, rules)
    }
}

#[derive(Debug)]
pub struct RuleData<'a> {
    pub rule: Rule,
    pub inner: AnnotatedPairs<'a>,
}

impl <'a>From<Pair<'a, Rule>> for RuleData<'a> {
    fn from(value: Pair<'a, Rule>) -> Self {
        RuleData {
            rule: value.as_rule(),
            inner: AnnotatedPairs::from(value),
        }
    }
}

impl <'a>From<AnnotatedPair<'a>> for RuleData<'a> {
    fn from(value: AnnotatedPair<'a>) -> Self {
        RuleData {
            rule: value.pair.as_rule(),
            inner: AnnotatedPairs::from(value.pair),
        }
    }
}

#[derive(Debug)]
pub enum PackingError<'a> { // At some point I'll have to break this out into sub-errors
    ASTPackerNoMatchFound(Vec<Option<RuleData<'a>>>),
    ASTPackerEmptyInner,
    PairsCountMismatch(Vec<Pair<'a, Rule>>),
    ArrayCastError(Vec<Option<Pair<'a, Rule>>>),
    NoRuleFound(Rule),
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
pub struct AnnotatedParsingError<'a> {
    pub error: PackingError<'a>,
    pub providence: Providence<'a>,
}

#[derive(Debug)]
pub enum Error<'a> { // TODO see about removing the lifetime specifier as it's more trouble than it's worth
    SequencingError(AnnotatedParsingError<'a>),
    ParsingError(pest::error::Error<Rule>),
    EvaluationError(EvaluationError),
}

impl <'a>From<AnnotatedParsingError<'a>> for Error<'a> {
    fn from(value: AnnotatedParsingError<'a>) -> Self {
        Error::SequencingError(value)
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

struct Token<'a> {
    token: Rule,
    providence: Providence<'a>,
}

enum TokenTree<'a> {
    Node(Token<'a>, Vec<TokenTree<'a>>),
    Leaf(Token<'a>),
}
