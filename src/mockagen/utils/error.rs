use crate::{mockagen::{packer::model::{PackingError, PackingErrorVariant, SyntaxChildren, SyntaxTree}, parser::Rule}, utils::packing::Providence};

pub fn make_tree_shape_error<T>(tree: SyntaxTree) -> Result<T, PackingError> {
    let error = PackingError::new(PackingErrorVariant::SyntaxUnhandledTreeShape(format!("{tree:?}")))
        .with_providence(tree.token.providence.clone());

    Err(error)
}

pub fn make_no_array_match_found_error<T, const N: usize>(nodes: [Option<(Rule, Providence, Option<SyntaxChildren>)>; N]) -> Result<T, PackingError> {
    let (providence, reformatted_vec) = reformat_rule_matcher_vec(nodes.to_vec());

    let error = PackingError::new(PackingErrorVariant::SyntaxNodeCountMismatch(reformatted_vec))
        .with_providence(providence);
        
    Err(error)
}

// Abandon all hope all ye who traverse this truly sinful code
pub fn reformat_rule_matcher_vec<'a>(vec: Vec<Option<(Rule, Providence<'a>, Option<SyntaxChildren<'a>>)>>) -> (Providence<'a>, Vec<Option<(Rule, String, Option<String>)>>) {
    let providence = vec.first().unwrap().as_ref().unwrap().1.clone();

    let reformatted_vec = vec.into_iter()
        .map(|opt| opt.map(|(rule, providence, maybe_children)|
            (rule
            , format!("{providence:?}")
            , maybe_children.map(|children| format!("{children:?}"))
            )))
        .collect();

    (providence, reformatted_vec)
}
