use chrono::{Duration, NaiveDate};
use itertools::Itertools;
use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng, Rng,
};

use crate::mockagen::{
    evaluator::model::{Context, Result},
    packer::packer::{
        DateLiteral, HigherOrderValue, Identifier, IdentifierValue, IntegerLiteral, IntegerValue, JoinValue, LiteralValue, PrimitiveValue, RealLiteral, RealValue, StringContent, StringLiteral, StringValue, TimestampDateValue, Value, ValueSet, WeightedValue
    }
};

use super::model::{MaybeWeighted, OutValue, WeightedT};

pub trait Generator2 {
    fn generate_value(&self, ctxt: &mut Context) -> Result<OutValue>;
}

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

pub struct AlternationGen {
    first: WeightedT<GeneratorEnum>,
    rest: Vec<WeightedT<GeneratorEnum>>
}

fn get_implicit_weighting<T>(maybe_weighteds: &[MaybeWeighted<T>]) -> f64 {
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

impl AlternationGen {
    fn new(weighted_values: Vec<WeightedValue>) -> Self {
        let maybe_weighteds = weighted_values.into_iter().map(MaybeWeighted::from).collect_vec();
        let implicit_weighting = get_implicit_weighting(maybe_weighteds.as_slice());
        let mut explicit_weighted = maybe_weighteds.into_iter()
            .map(|w| WeightedT::new(w, implicit_weighting));

        let WeightedT { weight: fst_weight, value: fst_value } = explicit_weighted.next().expect("Should have at least one value"); // TODO real error handling
        let first = WeightedT { weight: fst_weight, value: GeneratorEnum::from(fst_value) };
        let rest = explicit_weighted
            .scan(fst_weight, |cumul_weight, WeightedT { weight, value }| {
                *cumul_weight += weight;

                Some(WeightedT {
                    weight: *cumul_weight,
                    value: GeneratorEnum::from(value)
                })
            })
            .collect();

        Self { first, rest }
    }
}

impl Generator2 for AlternationGen {
    fn generate_value(&self, ctxt: &mut Context) -> Result<OutValue> {
        let target_weighting = thread_rng().gen_range(0.0..=100.0);

        self.rest.iter()
            .rev()
            .find(|WeightedT { weight, .. }| *weight < target_weighting)
            .unwrap_or(&self.first)
            .value
            .generate_value(ctxt)
    }
}

pub enum GeneratorEnum {
    DateRange(DateRangeGen),
    IntegerRange(IntegerRangeGen),
    RealRange(RealRangeGen),
    StringRange(StringRangeGen),
    Literal(LiteralGen),
    Identifier(IdentifierGen),
    Alternation(Box<AlternationGen>),
    Join(JoinGen),
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
        Self::Alternation(Box::new(AlternationGen::new(values.0)))
    }
}
