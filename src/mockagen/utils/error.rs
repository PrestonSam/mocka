use crate::mockagen::{model::{AnnotatedPackingError, Error, PackingError, Providence, SyntaxChildren, SyntaxTree}, parser::Rule};

pub fn to_debug<T>(value: T) -> String where T: core::fmt::Debug {
    format!("{:?}", value)
}

pub fn make_error_from_providence<'a>(providence: Providence<'a>, error: PackingError) -> Error {
    Error::from(AnnotatedPackingError { error, providence: to_debug(providence) })
}

pub fn make_tree_shape_error<T>(tree: SyntaxTree) -> Result<T, Error> {
    Err(make_error_from_providence(
        tree.token.providence.clone(),
        PackingError::SyntaxUnhandledTreeShape(to_debug(tree)))
    )
}

pub fn make_no_array_match_found_error<T, const N: usize>(nodes: [Option<(Rule, Providence, Option<SyntaxChildren>)>; N]) -> Result<T, Error> {
    let (providence, reformatted_vec) = reformat_rule_matcher_vec(nodes.to_vec());

    Err(make_error_from_providence(providence, PackingError::SyntaxNodeCountMismatch(reformatted_vec)))
}

// Abandon all hope all ye who traverse this truly sinful code
pub fn reformat_rule_matcher_vec<'a>(vec: Vec<Option<(Rule, Providence<'a>, Option<SyntaxChildren<'a>>)>>) -> (Providence<'a>, Vec<Option<(Rule, String, Option<String>)>>) {
    let providence = vec.first().unwrap().as_ref().unwrap().1.clone();

    let reformatted_vec = vec.into_iter()
        .map(|opt| opt.map(|(rule, providence, maybe_children)|
            (rule
            , to_debug(providence)
            , maybe_children.map(to_debug)
            )))
        .collect();

    (providence, reformatted_vec)
}