use logos::Logos;


// TODO I'm not confident that the output of Logos is actually compatible with Pest. Maybe I should use pest without a tokeniser?
// Will that have performance implications?

#[derive(Logos, Debug, PartialEq)]
enum Token {
    #[regex(" {4}")]
    Tab,

    #[regex("[=?|#,]")]
    Symbol,

    #[regex("INCLUDE|ONEOF|USING|DEF")]
    Keyword,

    #[regex("timestamp/date|integer|real|string|join|any")]
    Type,
}
