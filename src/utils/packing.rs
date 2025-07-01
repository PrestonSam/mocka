use std::{fmt::Debug, iter::once};
use pest::{iterators::Pair, RuleType, Span};
use thiserror::Error;


#[derive(Clone)]
pub struct Providence<'a> {
    pub span: Span<'a>,
    pub src: &'a str,
}

impl Providence<'_> {
    pub fn as_string(&self) -> String {
        self.src.to_string()
    }
    
     pub fn as_trimmed_string(&self) -> String {
        self.src.trim().to_string()
     }
}

fn trunc(str: &str, len: usize) -> String {
    if str.len() <= len {
        format!("{:?}", str)
    } else {
        format!("{:?}..", format!("{:.*}", len, str))
    }
}

impl core::fmt::Debug for Providence<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (line, column) = self.span.start_pos().line_col();

        write!(f, "At {}:{}, source code: {}", line, column, trunc(self.src, 40))
    }
}


#[derive(Debug)]
enum PackingErrorContext<Rule> {
    Providence(String),
    Rule(Rule),
}

#[derive(Debug, Error) ]
pub struct PackingError<Variant, Rule> {
    error: Variant,
    context: Vec<PackingErrorContext<Rule>>,
}

impl<Variant, Rule> PackingError<Variant, Rule> {
    pub fn new(error: Variant) -> Self {
        PackingError {
            error,
            context: vec![],
        }
    }

    pub fn with_providence(mut self, providence: Providence) -> Self {
        self.context.push(PackingErrorContext::Providence(format!("{:?}", providence)));
        self
    }

    pub fn with_rule(mut self, rule: Rule) -> Self {
        self.context.push(PackingErrorContext::Rule(rule));
        self
    }
}

pub trait SkipRules {
    type Rule: RuleType + SkipRules + Debug + Copy + Ord;

    fn get_skip_rules(&self) -> Vec<Self::Rule>;
}


#[derive(Debug, Clone)]
pub struct SyntaxToken<'a, Rule> {
    pub rule: Rule,
    pub providence: Providence<'a>,
}

impl<Rule> SyntaxToken<'_, Rule> {
    pub fn as_string(&self) -> String {
        self.providence.src.to_string()
    }
}

#[derive(Clone)]
pub struct SyntaxTree<'a, Rule> {
    pub token: SyntaxToken<'a, Rule>,
    pub children: Option<SyntaxChildren<'a, Rule>>,
}

impl<Rule> SyntaxTree<'_, Rule> {
    pub fn as_string(&self) -> String {
        self.token.as_string()
    }
}

#[derive(Clone)]
pub enum SyntaxChildren<'a, Rule> {
    One(Box<SyntaxTree<'a, Rule>>),
    Many(Vec<SyntaxTree<'a, Rule>>),
}

impl<'a, Rule> SyntaxChildren<'a, Rule> {
    pub fn get_values(self) -> Vec<SyntaxTree<'a, Rule>> {
        match self {
            SyntaxChildren::One(child) => vec![*child],
            SyntaxChildren::Many(children) => children,
        }
    }

    pub fn get_values_iter(self) -> impl Iterator<Item = SyntaxTree<'a, Rule>> {
        self.get_values().into_iter()
    }
}

impl<Rule> Debug for SyntaxChildren<'_, Rule>
where Rule: Debug + Copy
{
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

impl<'a, Rule> From<Pair<'a, Rule>> for SyntaxTree<'a, Rule>
where Rule: RuleType + SkipRules<Rule = Rule>
{
    fn from(pair: Pair<'a, Rule>) -> Self {
        let rule = pair.as_rule();
        let providence = Providence { src: pair.as_str(), span: pair.as_span() };
        let token = SyntaxToken { rule, providence };
        let skip_rules = rule.get_skip_rules();

        let mut inner_without_skip_rules = pair.into_inner()
            .filter(|pair| !(skip_rules.contains(&pair.as_rule())))
            .map(SyntaxTree::from);

        let children = match inner_without_skip_rules.next() {
            None => None,
            Some(first_child) => {
                match inner_without_skip_rules.next() {
                    None =>
                        Some(SyntaxChildren::One(first_child.into())),

                    Some(second_child) => {
                        let children = once(first_child)
                            .chain(once(second_child))
                            .chain(inner_without_skip_rules)
                            .collect();

                        Some(SyntaxChildren::Many(children))
                    }
                }
            }
        };

        SyntaxTree { token, children }
    }
}

impl<'a, Rule> From<(Rule, Providence<'a>, Option<SyntaxChildren<'a, Rule>>)> for SyntaxTree<'a, Rule> {
    fn from((rule, providence, children): (Rule, Providence<'a>, Option<SyntaxChildren<'a, Rule>>)) -> Self {
        SyntaxTree { token: SyntaxToken { rule, providence }, children }
    }
}

impl<Rule> Debug for SyntaxTree<'_, Rule>
where Rule: Debug + Copy
{
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
                    .finish(),
        }
    }
}
