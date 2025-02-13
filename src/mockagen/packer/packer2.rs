use lang_packer::Packer;

use crate::mockagen::parser::Rule2 as Rule;

#[derive(Debug, Packer)]
#[packer(rule = Rule::body)] // TODO nested special cases :(
pub struct Body(pub Option<IncludeStatements>, pub Vec<Definition>, pub EOI);

#[derive(Debug, Packer)]
#[packer(rule = Rule::EOI)]
pub struct EOI;

#[derive(Debug, Packer)]
#[packer(rule = Rule::include_statements)]
pub struct IncludeStatements(Vec<IncludeStatement>);

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
pub struct SingleValDef(Identifier, Value);

#[derive(Debug, Packer)]
#[packer(rule = Rule::multi_val_def)]
pub struct MultiValDef(Identifier, Value);

#[derive(Debug, Packer)]
#[packer(rule = Rule::nested_definition)]
pub struct NestedDefinition(Option<UsingIds>, AssignIds, NestedClauses);

#[derive(Debug, Packer)]
#[packer(rule = Rule::using_ids)]
pub struct UsingIds(Names);

#[derive(Debug, Packer)]
#[packer(rule = Rule::assign_ids)]
pub struct AssignIds(Names);

#[derive(Debug, Packer)]
#[packer(rule = Rule::names)]
pub struct Names(Vec<Identifier>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::nested_clauses)]
pub enum NestedClauses {
    MatchClausesWithWildcard(MatchClausesWithWildcard),
    MatchClauses(MatchClauses),
    AssignClauses(AssignClauses),
}

#[derive(Debug, Packer)]
#[packer(rule = Rule::match_clauses_with_wildcard)]
pub struct MatchClausesWithWildcard(MatchClauses, WildcardClause);

#[derive(Debug, Packer)]
#[packer(rule = Rule::match_clauses)]
pub struct MatchClauses(Vec<MatchClause>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::match_clause)]
pub struct MatchClause(Matchers, NestedClauses);

#[derive(Debug, Packer)]
#[packer(rule = Rule::wildcard_clause)]
pub struct WildcardClause(AnyValue, Box<NestedClauses>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::matchers)]
pub enum Matchers {
    MatchExpr(MatchExpr),
    MatcherSet(MatcherSet),
}

#[derive(Debug, Packer)]
#[packer(rule = Rule::matcher_set)]
pub struct MatcherSet(Vec<MatchExpr>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::match_expr)]
pub enum MatchExpr {
    LiteralValue(LiteralValue),
}

#[derive(Debug, Packer)]
#[packer(rule = Rule::assign_clauses)]
pub struct AssignClauses(Vec<AssignClause>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::assign_clause)]
pub struct AssignClause(WeightedValues, Option<AssignClauses>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::weighted_values)]
pub struct WeightedValues(Option<Weight>, Values);

#[derive(Debug, Packer)]
#[packer(rule = Rule::values)]
pub enum Values {
    Value(Value),
    ValueSet(ValueSet),
}

#[derive(Debug, Packer)]
#[packer(rule = Rule::value_set)]
pub struct ValueSet(Vec<WeightedValue>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::weighted_value)]
pub struct WeightedValue(Option<Weight>, Value);



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
pub struct TimestampDateValue(StringLiteral);

#[derive(Debug, Packer)]
#[packer(rule = Rule::literal_value)]
pub struct LiteralValue(StringLiteral);

#[derive(Debug, Packer)]
#[packer(rule = Rule::integer_value)]
pub struct IntegerValue(IntegerLiteral, Option<IntegerLiteral>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::string_value)]
pub struct StringValue(IntegerLiteral, IntegerLiteral);

#[derive(Debug, Packer)]
#[packer(rule = Rule::real_value)]
pub struct RealValue(RealLiteral, RealLiteral);

#[derive(Debug, Packer)]
#[packer(rule = Rule::join_value)]
pub struct JoinValue(Vec<Value>);

#[derive(Debug, Packer)]
#[packer(rule = Rule::any_value)]
pub struct AnyValue;

#[derive(Debug, Packer)]
#[packer(rule = Rule::identifier_value)]
pub struct IdentifierValue(Identifier);


#[derive(Debug, Packer)]
#[packer(rule = Rule::PERCENTAGE_NUMBER)]
pub struct PercentageNumber(usize);

#[derive(Debug, Packer)]
#[packer(rule = Rule::DECIMAL_SUFFIX)]
pub struct DecimalSuffix(usize);

#[derive(Debug, Packer)]
#[packer(rule = Rule::DECIMAL_PERCENTAGE_NUMBER)]
pub struct DecimalPercentageNumber(Option<PercentageNumber>, DecimalSuffix);

#[derive(Debug, Packer)]
#[packer(rule = Rule::WEIGHTING)]
pub enum Weighting {
    PercentageNumber(PercentageNumber),
    DecimalPercentageNumber(DecimalPercentageNumber),
}

#[derive(Debug, Packer)]
#[packer(rule = Rule::WEIGHT)]
pub struct Weight(Weighting);

#[derive(Debug, Packer)]
#[packer(rule = Rule::DATE_LITERAL)]
pub struct DateLiteral(String);

#[derive(Debug, Packer)]
#[packer(rule = Rule::REAL_LITERAL)]
pub struct RealLiteral(String);

#[derive(Debug, Packer)]
#[packer(rule = Rule::INTEGER_LITERAL)]
pub struct IntegerLiteral(String);

#[derive(Debug, Packer)]
#[packer(rule = Rule::IDENTIFIER)]
pub struct Identifier(String);

#[derive(Debug, Packer)]
#[packer(rule = Rule::STRING_LITERAL)]
pub struct StringLiteral(StringContent);

#[derive(Debug, Packer)]
#[packer(rule = Rule::string_content)]
pub struct StringContent(String);
