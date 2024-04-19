use crate::mockagen::{model::{AnnotatedPackingError, Error, PackingError, Providence, SyntaxChildren, SyntaxTree}, parser::Rule};

pub fn make_error_from_providence<'a>(providence: Providence<'a>, error: PackingError<'a>) -> Error<'a> {
    Error::from(AnnotatedPackingError { error, providence })
}

pub fn make_tree_shape_error<T>(tree: SyntaxTree) -> Result<T, Error> {
    Err(make_error_from_providence(
        tree.token.providence.clone(),
        PackingError::SyntaxUnhandledTreeShape(tree))
    )
}

pub fn make_no_array_match_found_error<'a, T, const N: usize>(nodes: [Option<(Rule, Providence<'a>, Option<SyntaxChildren<'a>>)>; N]) -> Result<T, Error<'a>> {
    // TODO forbidden code
    let providence = nodes.first().unwrap().as_ref().unwrap().1.clone();

    Err(make_error_from_providence(providence, PackingError::SyntaxNodeCountMismatch(nodes.to_vec())))
}