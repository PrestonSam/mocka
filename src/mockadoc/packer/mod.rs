pub mod model;
// mod legacy_packer;
mod packer;
mod error;
mod utils;

use pest::iterators::Pairs;

pub use packer::*;
use lang_packer_model::{generic_utils::{PackingError, SyntaxTree}, pack_trees::{unpack_only_tree, HasRule, TokenPacker}};

use crate::mockadoc::parser::Rule;

pub fn pack(pairs: Pairs<'_, Rule>) -> Result<Body, PackingError<Rule>> {
    let trees: Vec<_> = pairs.map(SyntaxTree::from)
        .collect();

    unpack_only_tree(&trees)
        .and_then(Body::pack)
}
