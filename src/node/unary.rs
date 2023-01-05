use super::*;
use crate::Rule;
use pest::iterators::{Pair, Pairs};
use std::iter::Peekable;

#[derive(Debug)]
pub struct SuffixOpNode {
    pub operator: &'static str,
    pub expr: Node,
}

#[derive(Debug)]
pub struct PrefixOpNode {
    pub operator: &'static str,
    pub expr: Node,
}

pub fn parse_unary_node(pair: Pair<Rule>) -> Result<Node, NodeError> {
    assert_eq!(pair.as_rule(), Rule::UNARY);
    let mut pairs = pair.into_inner().peekable();
    let node = match pairs.peek().unwrap().as_rule() {
        Rule::PPLUS => Node::PrefixOp(Box::new(PrefixOpNode {
            operator: PPLUS,
            expr: parse_unary_node(pairs.nth(1).unwrap())?,
        })),
        Rule::MMINUS => Node::PrefixOp(Box::new(PrefixOpNode {
            operator: MMINUS,
            expr: parse_unary_node(pairs.nth(1).unwrap())?,
        })),
        Rule::SIZEOF => {
            pairs.next();
            parse_sizeof_node(pairs)?
        }
        _ => parse_suffix_node(pairs.next().unwrap().into_inner().peekable())?,
    };
    Ok(node)
}

pub fn parse_suffix_node(mut pairs: Peekable<Pairs<Rule>>) -> Result<Node, NodeError> {
    let first = pairs.next().unwrap();
    let second = pairs.next();

    let node = match second.map(|x| x.as_rule()) {
        Some(Rule::PPLUS) => Node::SuffixOp(Box::new(SuffixOpNode {
            operator: PPLUS,
            expr: parse_primary_node(first)?,
        })),
        Some(Rule::MMINUS) => Node::SuffixOp(Box::new(SuffixOpNode {
            operator: MMINUS,
            expr: parse_primary_node(first)?,
        })),
        _ => parse_primary_node(first)?,
    };

    Ok(node)
}
