pub mod model;
mod packer;
mod packer2;

pub use packer::pack_mockagen;
use packer2::Body;
use pest::iterators::Pairs;
use token_packer::{generic_utils::SyntaxTree, pack_trees::{unpack_only_tree, TokenPacker}};

use super::{parser::Rule2, MockagenError};

pub fn pack_mockagen2(pairs: Pairs<'_, Rule2>) -> Result<Body, MockagenError> {
    let trees = pairs.map(SyntaxTree::<'_, Rule2>::from).collect();

    unpack_only_tree(trees)
        .and_then(Body::pack)
        .map_err(MockagenError::from)
}
