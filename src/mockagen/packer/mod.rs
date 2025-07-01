pub mod packer;

use packer::Body;
use pest::iterators::Pairs;
use lang_packer_model::{generic_utils::SyntaxTree, pack_trees::{unpack_only_tree, TokenPacker}};

use super::{parser::Rule, MockagenError};

pub fn pack_mockagen(pairs: Pairs<'_, Rule>) -> Result<Body, MockagenError> {
    let trees: Vec<_> = pairs.map(SyntaxTree::from)
        .collect();

    unpack_only_tree(&trees)
        .and_then(Body::pack)
        .map_err(MockagenError::from)
}
