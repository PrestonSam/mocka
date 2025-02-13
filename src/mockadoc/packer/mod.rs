pub mod model;
mod packer;
mod packer2;
mod error;
mod utils;

use pest::iterators::Pairs;

pub use packer2::*;
use token_packer::{generic_utils::{PackingError, SyntaxTree}, pack_trees::{unpack_only_tree, TokenPacker}};

use crate::mockadoc::parser::Rule;

pub fn pack(pairs: Pairs<'_, Rule>) -> Result<Body, PackingError<Rule>> {
    let trees: Vec<_> = pairs.map(SyntaxTree::<'_, Rule>::from)
        .collect();

    unpack_only_tree(trees)
        .and_then(Body::pack)
}
