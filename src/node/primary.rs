use crate::resolve::variable_scope::Entity;

use super::*;
use crate::Rule;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub enum PrimaryNode {
    Integer(i64),
    String(String),
    Char(char),
    Identifier(String, Option<Entity>),
}

pub fn parse_primary_node(pair: Pair<Rule>) -> Result<PrimaryNode, NodeError> {
    debug_assert_eq!(pair.as_rule(), Rule::PRIMARY);
    let pair = pair.into_inner().next().unwrap();
    let node = match pair.as_rule() {
        Rule::INTEGER => {
            let n = pair.as_str().parse().unwrap();
            PrimaryNode::Integer(n)
        }
        Rule::STRING => {
            let s = pair.as_str().into();
            PrimaryNode::String(s)
        }
        Rule::CHARACTER => {
            let s = pair.as_str().to_string().chars().collect::<Vec<_>>();
            PrimaryNode::Char(s[1])
        }
        Rule::IDENTIFIER => {
            let s = pair.as_str().into();
            PrimaryNode::Identifier(s, None)
        }
        _ => panic!("not primary, found {:?}", pair.as_rule()),
    };

    Ok(node)
}
