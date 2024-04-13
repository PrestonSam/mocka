use pest::iterators::Pair;

use crate::mockagen::parser::Rule;


pub fn is_not_tab(pair: &Pair<'_, Rule>) -> bool {
    pair.as_rule() != Rule::TAB
}
