use std::rc::Rc;

use chrono::{Duration, NaiveDate};
use itertools::Itertools;
use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng, Rng,
};

use crate::{mockagen::{
    evaluator::model::{Context, CumulWeightedGen, EvaluationError, Result},
    packer::packer::{
        AssignClause, AssignClauses, DateLiteral, HigherOrderValue, Identifier, IdentifierValue, IntegerLiteral, IntegerValue, JoinValue, LiteralValue, MatchClause, MatchClauses, MatchExpr, MatcherSet, Matchers, NestedClauses, PrimitiveValue, RealLiteral, RealValue, StringContent, StringLiteral, StringValue, TimestampDateValue, Value, ValueSet, Values, WeightedValue, WeightedValues, WildcardClause
    }
}, utils::iterator::FindOk};

use super::model::{MaybeWeightedGen, OutValue, WeightedGen};

pub trait Generator2 {
    fn generate_value(&self, ctxt: &mut Context) -> Result<OutValue>;
}

#[derive(Debug)]
pub struct DateRangeGen { after: NaiveDate, range_in_days: i64 }

impl DateRangeGen {
    fn new(from: NaiveDate, to: NaiveDate) -> Self {
        Self {
            range_in_days: to.signed_duration_since(from).num_days(),
            after: from
        }
    }
}

impl Generator2 for DateRangeGen {
    fn generate_value(&self, _: &mut Context) -> Result<OutValue> {
        let offset = Duration::days(thread_rng().gen_range(0..=self.range_in_days));

        Ok(OutValue::NaiveDate(self.after + offset))
    }
}

#[derive(Debug)]
pub struct IntegerRangeGen { from: i64, to: i64 }

impl IntegerRangeGen {
    fn new(from: i64, to: i64) -> Self {
        // Rust doesn't support backwards ranges, so reverse negative inputs
        Self {
            from: from.min(to),
            to: from.max(to)
        }
    }
}

impl Generator2 for IntegerRangeGen {
    fn generate_value(&self, _: &mut Context) -> Result<OutValue> {
        Ok(OutValue::I64(thread_rng().gen_range(self.from..=self.to)))
    }
}

#[derive(Debug)]
pub struct RealRangeGen { from: f64, to: f64 }

impl RealRangeGen {
    fn new(from: f64, to: f64) -> Self {
        // Rust doesn't support backwards ranges, so reverse negative inputs
        Self {
            from: from.min(to),
            to: from.max(to)
        }
    }
}

impl Generator2 for RealRangeGen {
    fn generate_value(&self, _: &mut Context) -> Result<OutValue> {
        Ok(OutValue::F64(thread_rng().gen_range(self.from..=self.to)))
    }
}

#[derive(Debug)]
pub struct StringRangeGen { from: i64, to: i64 }

impl StringRangeGen {
    fn new(from: i64, to: i64) -> Self {
        Self { from, to }
    }
}

impl Generator2 for StringRangeGen {
    fn generate_value(&self, _: &mut Context) -> Result<OutValue> {
        let length = thread_rng().gen_range(self.from..=self.to) as usize;

        Ok(OutValue::String(Alphanumeric.sample_string(&mut thread_rng(), length)))
    }
}

#[derive(Debug)]
pub struct LiteralGen(String);

impl LiteralGen {
    fn new(literal: String) -> Self {
        Self(literal)
    }
}

impl Generator2 for LiteralGen {
    fn generate_value(&self, _: &mut Context) -> Result<OutValue> {
        Ok(OutValue::String(self.0.clone()))
    }
}

#[derive(Debug)]
pub struct IdentifierGen(String);

impl IdentifierGen {
    fn new(literal: String) -> Self {
        Self(literal)
    }
}

impl Generator2 for IdentifierGen {
    fn generate_value(&self, ctxt: &mut Context) -> Result<OutValue> {
        let value = ctxt.get_value(&self.0)?;

        Ok((*value).clone())
    }
}

#[derive(Debug)]
pub struct JoinGen(Vec<GeneratorEnum>);

impl JoinGen {
    fn new(values: Vec<Value>) -> Self {
        let generators = values
            .into_iter()
            .map(GeneratorEnum::from)
            .collect();

        Self(generators)
    }
}

impl Generator2 for JoinGen {
    fn generate_value(&self, ctxt: &mut Context) -> Result<OutValue> {
        let output = self.0.iter()
            .map(|gen| gen.generate_value(ctxt))
            .map_ok(|v| v.to_string()) // TODO benchmark this against creating a vector and calling join on it
            .collect::<Result<_>>()?;

        Ok(OutValue::String(output))
    }
}

#[derive(Debug)]
pub struct AlternationGen {
    wgens: Vec<CumulWeightedGen>,
    last: CumulWeightedGen,
}

impl AlternationGen {
    fn new(maybe_weighteds: Vec<MaybeWeightedGen>) -> Self {
        let implicit_weighting = Self::get_implicit_weighting(maybe_weighteds.as_slice());
        let mut explicit_weighted = maybe_weighteds.into_iter()
            .map(|w| w.as_weighted_gen(implicit_weighting))
            .rev();

        let WeightedGen { value: last_value, .. } = explicit_weighted.next()
            .expect("Should have at least one value"); // TODO real error handling

        let others = explicit_weighted
            .scan(0.0, |cumul_weight, WeightedGen { weight, value }| {
                *cumul_weight += weight;

                Some(CumulWeightedGen { cumul_weight: *cumul_weight, value })
            })
            .collect();

        Self {
            wgens: others,
            last: CumulWeightedGen { cumul_weight: 100.0, value: last_value }
        }
    }

    fn get_implicit_weighting(maybe_weighteds: &[MaybeWeightedGen]) -> f64 {
        let mut total_explicit_percentage = 0.0;
        let mut implicit_percentage_count = 0.0;

        for maybe_weighted in maybe_weighteds {
            match maybe_weighted.weight {
                Some(weight) => total_explicit_percentage += weight,
                None => implicit_percentage_count += 1.0,
            }
        }

        (100.0 - total_explicit_percentage) / implicit_percentage_count
    }
}

impl Generator2 for AlternationGen {
    fn generate_value(&self, ctxt: &mut Context) -> Result<OutValue> {
        let target_weighting = thread_rng().gen_range(0.0..=100.0);

        self.wgens.iter()
            .find(|CumulWeightedGen { cumul_weight, .. }| target_weighting < *cumul_weight)
            .unwrap_or(&self.last)
            .value
            .generate_value(ctxt)
    }
}


#[derive(Debug)]
pub enum ValueTree {
    Match(Vec<MatchArm>, Option<Box<ValueTree>>),
    Assign(Vec<AssignArm>, GeneratorEnum),
}

#[derive(Debug)]
struct MatchArm {
    match_conditions: MatchConditions,
    children: ValueTree,
}

impl MatchArm {
    fn new(clause: MatchClause, ids: &[&str]) -> Result<Self> {
        let MatchClause(_, matchers, nested_clauses) = clause;

        Ok(Self {
            match_conditions: MatchConditions::from_matchers(ids[0], matchers),
            children: ValueTree::from_nested_clauses(&ids[1..], nested_clauses)?,
        })
    }

    pub fn is_match(&self, ctxt: &mut Context) -> Result<bool> {
        self.match_conditions.is_match(ctxt)
    }
}

#[derive(Debug)]
struct MatchConditions {
    id: String,
    matchers: Vec<MatchExpr>
}

impl MatchConditions {
    fn from_matchers(id: &str, matchers: Matchers) -> Self {
        let matchers = match matchers {
            Matchers::MatchExpr(match_expr) => vec![ match_expr, ],
            Matchers::MatcherSet(MatcherSet(match_exprs)) => match_exprs,
        };

        Self { id: id.to_owned(), matchers }
    }

    fn from_match_exprs(id: &str, match_exprs: Vec<MatchExpr>) -> Self {
        Self { id: id.to_owned(), matchers: match_exprs }
    }

    pub fn is_match(&self, ctxt: &mut Context) -> Result<bool> {
        let value = ctxt.get_value(self.id.as_str())?;

        Ok(self.matchers.iter().any(|m| m.is_match(value.as_ref())))
    }
}

#[derive(Debug)]
struct AssignArm {
    match_conditions: MatchConditions,
    children: Option<ValueTree>
}

impl AssignArm {
    pub fn new(id: &str, weighted_values: &WeightedValues, children: Option<ValueTree>) -> Result<Self> {
        let match_exprs = weighted_values.1.iter()
            .map(MatchExpr::try_from)
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            match_conditions: MatchConditions::from_match_exprs(id, match_exprs),
            children,
        })
    }

    pub fn is_match(&self, ctxt: &mut Context) -> Result<bool> {
        self.match_conditions.is_match(ctxt)
    }
}

impl ValueTree {
    pub fn from_nested_clauses(ids: &[&str], nested_clauses: NestedClauses) -> Result<Self> {
        match nested_clauses {
            NestedClauses::AssignClauses(clauses) => Self::from_assign_clauses(ids, clauses),
            NestedClauses::MatchClauses(clauses) => Self::from_match_clauses(ids, clauses, None),
            NestedClauses::MatchClausesWithWildcard(clauses) => Self::from_match_clauses(ids, clauses.0, Some(clauses.1)),
        }
    }

    fn from_match_clauses(ids: &[&str], match_clauses: MatchClauses, wildcard: Option<WildcardClause>) -> Result<Self> {
        let arms = match_clauses.0.into_iter()
            .map(|clause| MatchArm::new(clause, ids))
            .collect::<Result<_>>()?;

        let wildcard_arm = wildcard.map(|clause|
                Self::from_nested_clauses(&ids[1..], *clause.1).map(Box::new))
            .transpose()?;

        Ok(Self::Match(arms, wildcard_arm))
    }

    fn from_assign_clauses(ids: &[&str], assign_clauses: AssignClauses) -> Result<Self> {
        let (arms, gens): (Vec<_>, Vec<MaybeWeightedGen>) = assign_clauses.0.into_iter()
            .map(|AssignClause(_, wvalues, maybe_children)| {
                let children = maybe_children
                    .map(|c| Self::from_assign_clauses(&ids[1..], c))
                    .transpose()?;

                let arm = AssignArm::new(ids[0], &wvalues, children)?;
                Ok((arm, wvalues.into()))
            })
            .collect::<Result<Vec<(AssignArm, MaybeWeightedGen)>>>()?
            .into_iter()
            .unzip();

        Ok(Self::Assign(arms, GeneratorEnum::from(gens)))
    }

    fn get_value(&self, ctxt: &mut Context) -> Result<OutValue> {
        match self {
            ValueTree::Assign(_, gen) => gen.generate_value(ctxt),
            ValueTree::Match(_, _) => Err(EvaluationError::ExpectedValueFoundMatcher),
        }
    }

    fn find_child_from_context(&self, ctxt: &mut Context) -> Result<&ValueTree> {
        match self {
            ValueTree::Match(match_arms, sibling_set) =>
                match_arms.iter()
                    .find_ok(|arm| arm.is_match(ctxt))?
                    .map(|arm| &arm.children)
                    .or(sibling_set.as_ref().map(|b| b.as_ref()))
                    .ok_or(EvaluationError::NoMatchForValue),

            ValueTree::Assign(assign_arms, _) =>
                assign_arms.iter()
                    .find_ok(|arm| arm.is_match(ctxt))?
                    .ok_or(EvaluationError::NoMatchForValue)?
                    .children
                    .as_ref()
                    .ok_or(EvaluationError::NoChildrenForTree),
        }
    }

    fn generate_value_at_depth(&self, ctxt: &mut Context, read_depth: usize) -> Result<OutValue> {
        (0..read_depth)
            .try_fold(self, |tree, _| tree.find_child_from_context(ctxt))?
            .get_value(ctxt)
    }
}

#[derive(Debug)]
pub struct NestedGenerator { read_depth: usize, tree: Rc<ValueTree> }

impl NestedGenerator {
    pub fn new(read_depth: usize, tree: Rc<ValueTree>) -> Self {
        Self { read_depth, tree }
    }
}

impl Generator2 for NestedGenerator {
    fn generate_value(&self, ctxt: &mut Context) -> Result<super::model::OutValue> {
        self.tree.generate_value_at_depth(ctxt, self.read_depth)
    }
}

#[derive(Debug)]
pub enum GeneratorEnum {
    DateRange(DateRangeGen),
    IntegerRange(IntegerRangeGen),
    RealRange(RealRangeGen),
    StringRange(StringRangeGen),
    Literal(LiteralGen),
    Identifier(IdentifierGen),
    Alternation(Box<AlternationGen>),
    Join(JoinGen),
    Nested(NestedGenerator),
}

impl Generator2 for GeneratorEnum {
    fn generate_value(&self, ctxt: &mut Context) -> Result<OutValue> {
        match self {
            Self::DateRange(gen) => gen.generate_value(ctxt),
            Self::IntegerRange(gen) => gen.generate_value(ctxt),
            Self::RealRange(gen) => gen.generate_value(ctxt),
            Self::StringRange(gen) => gen.generate_value(ctxt),
            Self::Literal(gen) => gen.generate_value(ctxt),
            Self::Identifier(gen) => gen.generate_value(ctxt),
            Self::Alternation(gen) => gen.generate_value(ctxt),
            Self::Join(gen) => gen.generate_value(ctxt),
            Self::Nested(gen) => gen.generate_value(ctxt),
        }
    }
}

impl From<HigherOrderValue> for GeneratorEnum {
    fn from(value: HigherOrderValue) -> Self {
        match value {
            HigherOrderValue::JoinValue(JoinValue(values)) =>
                Self::Join(JoinGen::new(values)),

            HigherOrderValue::IdentifierValue(IdentifierValue(Identifier(identifier))) =>
                Self::Identifier(IdentifierGen::new(identifier)),
        }
    }
}

impl From<PrimitiveValue> for GeneratorEnum {
    fn from(value: PrimitiveValue) -> Self {
        match value {
            PrimitiveValue::TimestampDate(TimestampDateValue(DateLiteral(from), DateLiteral(to))) =>
                Self::DateRange(DateRangeGen::new(from, to)),

            PrimitiveValue::Literal(LiteralValue(StringLiteral(StringContent(literal)))) =>
                Self::Literal(LiteralGen::new(literal)),

            PrimitiveValue::Integer(IntegerValue(IntegerLiteral(from), maybe_to)) =>
                Self::IntegerRange(IntegerRangeGen::new(from, maybe_to.map(|IntegerLiteral(i)| i).unwrap_or(i64::MAX))),

            PrimitiveValue::String(StringValue(IntegerLiteral(from), IntegerLiteral(to))) =>
                Self::IntegerRange(IntegerRangeGen::new(from, to)),

            PrimitiveValue::Real(RealValue(RealLiteral(from), RealLiteral(to))) =>
                Self::RealRange(RealRangeGen::new(from, to)),
        }
    }
}

impl From<Value> for GeneratorEnum {
    fn from(value: Value) -> Self {
        match value {
            Value::HigherOrder(higher_order_value) => higher_order_value.into(),
            Value::Primitive(primitive_value) => primitive_value.into(),
        }
    }
}

impl From<ValueSet> for GeneratorEnum {
    fn from(values: ValueSet) -> Self {
        let wvals = values.0.into_iter()
            .map(|v| MaybeWeightedGen {
                weight: v.0.map(|w| w.get()),
                value: GeneratorEnum::from(v.1)
            })
            .collect();

        Self::Alternation(AlternationGen::new(wvals).into())
    }
}

impl From<Vec<MaybeWeightedGen>> for GeneratorEnum {
    fn from(value: Vec<MaybeWeightedGen>) -> Self {
        GeneratorEnum::Alternation(AlternationGen::new(value).into())
    }
}

impl From<WeightedValues> for MaybeWeightedGen {
    fn from(value: WeightedValues) -> Self {
        let WeightedValues(weight, values) = value;

        let weight = weight.map(|w| w.get());

        let maybe_weighteds = match values {
            Values::Value(value) =>
                vec![ MaybeWeightedGen { weight: Some(100.0), value: GeneratorEnum::from(value) }, ],

            Values::ValueSet(ValueSet(wvalues)) =>
                wvalues.into_iter()
                    .map(|v| MaybeWeightedGen { weight: v.0.map(|w| w.get()), value: GeneratorEnum::from(v.1) })
                    .collect(),
        };
        let value = GeneratorEnum::Alternation(AlternationGen::new(maybe_weighteds).into());

        Self { weight, value }
    }
}
