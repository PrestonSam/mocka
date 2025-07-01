#![allow(dead_code)]

use chrono::NaiveDate;
use lang_packer::{DbgPacker, Packer};

use crate::mockagen::parser::Rule;

#[derive(Debug, Packer)]
#[packer(rule = Rule::body)] // TODO nested special cases :(
pub struct Body(pub Option<IncludeStatements>, pub Vec<Definition>, pub EOI);

#[derive(Debug, Packer)]
#[packer(rule = Rule::EOI)]
pub struct EOI;

#[derive(Debug, Packer)]
#[packer(rule = Rule::include_statements)]
pub struct IncludeStatements(pub Vec<IncludeStatement>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::include_statement)]
pub struct IncludeStatement;

#[derive(Debug, Packer)]
#[packer(rule = Rule::definition)]
pub enum Definition {
    Single(SingleDefinition),
    Nested(NestedDefinition),
}

#[derive(Debug, Packer)]
#[packer(rule = Rule::single_definition)]
pub enum SingleDefinition {
    SingleVal(SingleValDef),
    MultiVal(MultiValDef),
}

#[derive(Debug, Packer)]
#[packer(rule = Rule::single_val_def)]
pub struct SingleValDef(pub Identifier, pub Value);

#[derive(Debug, Packer)]
#[packer(rule = Rule::multi_val_def)]
pub struct MultiValDef(pub Identifier, pub Vec<Tab>, pub ValueSet);

#[derive(Debug, Packer)]
#[packer(rule = Rule::nested_definition)]
pub struct NestedDefinition(pub Option<UsingIds>, pub AssignIds, pub NestedClauses);

#[derive(Debug, Packer)]
#[packer(rule = Rule::using_ids)]
pub struct UsingIds(pub Names);

#[derive(Debug, Packer)]
#[packer(rule = Rule::assign_ids)]
pub struct AssignIds(pub Names);

#[derive(Debug, Packer)]
#[packer(rule = Rule::names)]
pub struct Names(pub Vec<Identifier>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::nested_clauses)]
pub enum NestedClauses {
    MatchClausesWithWildcard(MatchClausesWithWildcard),
    MatchClauses(MatchClauses),
    AssignClauses(AssignClauses),
}

#[derive(Debug, Packer)]
#[packer(rule = Rule::match_clauses_with_wildcard)]
pub struct MatchClausesWithWildcard(pub MatchClauses, pub WildcardClause);

#[derive(Debug, Packer)]
#[packer(rule = Rule::match_clauses)]
pub struct MatchClauses(pub Vec<MatchClause>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::match_clause)]
pub struct MatchClause(pub Vec<Tab>, pub Matchers, pub NestedClauses);

#[derive(Debug, Packer)]
#[packer(rule = Rule::wildcard_clause)]
pub struct WildcardClause(pub Vec<Tab>, pub Box<NestedClauses>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::matchers)]
pub enum Matchers {
    MatchExpr(MatchExpr),
    MatcherSet(MatcherSet),
}

#[derive(Debug, Packer)]
#[packer(rule = Rule::matcher_set)]
pub struct MatcherSet(pub Vec<MatchExpr>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::match_expr)]
pub enum MatchExpr {
    LiteralValue(LiteralValue),
}

#[derive(Debug, Packer)]
#[packer(rule = Rule::assign_clauses)]
pub struct AssignClauses(pub Vec<AssignClause>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::assign_clause)]
pub struct AssignClause(pub Vec<Tab>, pub WeightedValues, pub Option<AssignClauses>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::weighted_values)]
pub struct WeightedValues(pub Option<Weight>, pub Values);

#[derive(Debug, Packer)]
#[packer(rule = Rule::values)]
pub enum Values {
    Value(Value),
    ValueSet(ValueSet),
}

#[derive(Debug, Packer)]
#[packer(rule = Rule::value_set)]
pub struct ValueSet(pub Vec<WeightedValue>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::weighted_value)]
pub struct WeightedValue(pub Option<Weight>, pub Value);

#[derive(Debug, Packer)]
#[packer(rule = Rule::value)]
pub enum Value {
    HigherOrder(HigherOrderValue),
    Primitive(PrimitiveValue),
}

#[derive(Debug, Packer)]
#[packer(rule = Rule::higher_order_value)]
pub enum HigherOrderValue {
    JoinValue(JoinValue),
    IdentifierValue(IdentifierValue),
}

#[derive(Debug, Packer)]
#[packer(rule = Rule::primitive_value)]
pub enum PrimitiveValue {
    TimestampDate(TimestampDateValue),
    Literal(LiteralValue),
    Integer(IntegerValue),
    String(StringValue),
    Real(RealValue),
}

#[derive(Debug, Packer)]
#[packer(rule = Rule::timestamp_date_value)]
pub struct TimestampDateValue(pub DateLiteral, pub DateLiteral);

#[derive(Debug, Packer)]
#[packer(rule = Rule::literal_value)]
pub struct LiteralValue(pub StringLiteral);

#[derive(Debug, Packer)]
#[packer(rule = Rule::integer_value)]
pub struct IntegerValue(pub IntegerLiteral, pub Option<IntegerLiteral>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::string_value)]
pub struct StringValue(pub IntegerLiteral, pub IntegerLiteral);

#[derive(Debug, Packer)]
#[packer(rule = Rule::real_value)]
pub struct RealValue(pub RealLiteral, pub RealLiteral);

#[derive(Debug, Packer)]
#[packer(rule = Rule::join_value)]
pub struct JoinValue(pub Vec<Value>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::any_value)]
pub struct AnyValue;

#[derive(Debug, Packer)]
#[packer(rule = Rule::identifier_value)]
pub struct IdentifierValue(pub Identifier);


#[derive(Debug, Packer)]
#[packer(rule = Rule::PERCENTAGE_NUMBER)]
pub struct PercentageNumber(pub f64);

#[derive(Debug, Packer)]
#[packer(rule = Rule::DECIMAL_SUFFIX)]
pub struct DecimalSuffix(pub f64);

#[derive(Debug, Packer)]
#[packer(rule = Rule::DECIMAL_PERCENTAGE_NUMBER)]
pub struct DecimalPercentageNumber(pub Option<PercentageNumber>, pub DecimalSuffix);

#[derive(Debug, Packer)]
#[packer(rule = Rule::WEIGHTING)]
pub enum Weighting {
    PercentageNumber(PercentageNumber),
    DecimalPercentageNumber(DecimalPercentageNumber),
}

#[derive(Debug, Packer)]
#[packer(rule = Rule::WEIGHT)]
pub struct Weight(pub Weighting);

#[derive(Debug, Packer)]
#[packer(rule = Rule::DATE_LITERAL)]
pub struct DateLiteral(pub NaiveDate);

#[derive(Debug, Packer)]
#[packer(rule = Rule::REAL_LITERAL)]
pub struct RealLiteral(pub f64);

#[derive(Debug, Packer)]
#[packer(rule = Rule::INTEGER_LITERAL)]
pub struct IntegerLiteral(pub i64);

#[derive(Debug, Packer)]
#[packer(rule = Rule::IDENTIFIER)]
pub struct Identifier(pub String);

#[derive(Debug, Packer)]
#[packer(rule = Rule::STRING_LITERAL)]
pub struct StringLiteral(pub StringContent);

#[derive(Debug, Packer)]
#[packer(rule = Rule::string_content)]
pub struct StringContent(pub String);

#[derive(Debug, Packer)]
#[packer(rule = Rule::TAB)]
pub struct Tab;

impl Weight {
    pub fn get(&self) -> f64 {
        let percentage = match &self.0 {
            Weighting::PercentageNumber(PercentageNumber(perc)) =>
                *perc,

            Weighting::DecimalPercentageNumber(DecimalPercentageNumber(maybe_perc, DecimalSuffix(suffix))) =>
                maybe_perc.as_ref()
                    .map(|p| p.0)
                    .unwrap_or(0.0) + suffix / 100.0,
        };

        percentage / 100.0
    }
}
