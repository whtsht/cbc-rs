use super::*;
use crate::Rule;
use pest::iterators::Pair;

#[derive(Debug)]
pub enum PrimaryNode {
    Integer(i64),
    String(String),
    Char(char),
    Identifier(String),
}

pub fn parse_primary_node(pair: Pair<Rule>) -> Result<Node, NodeError> {
    debug_assert_eq!(pair.as_rule(), Rule::PRIMARY);
    let pair = pair.into_inner().next().unwrap();
    let node = match pair.as_rule() {
        Rule::INTEGER => {
            let n = pair.as_str().parse().unwrap();
            Node::Primary(Box::new(PrimaryNode::Integer(n)))
        }
        Rule::STRING => {
            let s = pair.as_str().into();
            Node::Primary(Box::new(PrimaryNode::String(s)))
        }
        Rule::CHARACTER => {
            let s = pair.as_str().to_string().chars().collect::<Vec<_>>();
            Node::Primary(Box::new(PrimaryNode::Char(s[1])))
        }
        Rule::IDENTIFIER => {
            let s = pair.as_str().into();
            Node::Primary(Box::new(PrimaryNode::Identifier(s)))
        }
        _ => panic!("not primary, found {:?}", pair.as_rule()),
    };

    Ok(node)
}
