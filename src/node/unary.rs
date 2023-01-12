use super::*;
use crate::Rule;
use pest::iterators::{Pair, Pairs};
use std::iter::Peekable;

#[derive(Debug)]
pub enum UnaryNode {
    Increment(Node),
    Decrement(Node),
    PLUS(Node),
    Minus(Node),
    Tilde(Node),
    Not(Node),
    Star(Node),
    And(Node),
    SizeofExpr(Node),
    SizeofType(Node),
    Suffix(Node, Vec<SuffixOp>),
}

#[derive(Debug)]
pub enum SuffixOp {
    Increment,
    Decrement,
    Dot(String),
    Arrow(String),
    Array(Node),
    CallFu(Vec<Node>),
}

pub fn parse_unary_node(pair: Pair<Rule>) -> Result<Node, NodeError> {
    assert_eq!(pair.as_rule(), Rule::UNARY);
    let mut pairs = pair.into_inner().peekable();
    let node = match pairs.peek().unwrap().as_rule() {
        Rule::PPLUS => Node::Unary(Box::new(UnaryNode::Increment(parse_unary_node(
            pairs.nth(1).unwrap(),
        )?))),
        Rule::MMINUS => Node::Unary(Box::new(UnaryNode::Decrement(parse_unary_node(
            pairs.nth(1).unwrap(),
        )?))),
        Rule::PLUS => Node::Unary(Box::new(UnaryNode::And(parse_term_node(
            pairs.nth(1).unwrap(),
        )?))),
        Rule::MINUS => Node::Unary(Box::new(UnaryNode::Minus(parse_term_node(
            pairs.nth(1).unwrap(),
        )?))),
        Rule::NOT => Node::Unary(Box::new(UnaryNode::Not(parse_term_node(
            pairs.nth(1).unwrap(),
        )?))),
        Rule::TILDE => Node::Unary(Box::new(UnaryNode::Tilde(parse_term_node(
            pairs.nth(1).unwrap(),
        )?))),
        Rule::AND => Node::Unary(Box::new(UnaryNode::And(parse_term_node(
            pairs.nth(1).unwrap(),
        )?))),
        Rule::STAR => Node::Unary(Box::new(UnaryNode::Star(parse_term_node(
            pairs.nth(1).unwrap(),
        )?))),
        Rule::SIZEOF => {
            pairs.next();
            parse_sizeof_node(pairs)?
        }
        _ => parse_suffix_node(pairs.next().unwrap().into_inner().peekable())?,
    };
    Ok(node)
}

pub fn parse_suffix_node(mut pairs: Peekable<Pairs<Rule>>) -> Result<Node, NodeError> {
    let primary = parse_primary_node(pairs.next().unwrap())?;

    let mut ops = vec![];

    while let Some(pair) = pairs.next() {
        match pair.as_rule() {
            Rule::PPLUS => ops.push(SuffixOp::Increment),
            Rule::MMINUS => ops.push(SuffixOp::Decrement),
            Rule::DOT => {
                let name = pairs.next().unwrap().as_str().into();
                ops.push(SuffixOp::Dot(name))
            }
            Rule::ARROW => {
                let name = pairs.next().unwrap().as_str().into();
                ops.push(SuffixOp::Arrow(name))
            }
            Rule::LSB => {
                let idx = parse_expr_node(pairs.next().unwrap())?;
                ops.push(SuffixOp::Array(idx));
                pairs.next(); // ]
            }
            Rule::LPT => {
                ops.push(SuffixOp::CallFu(parse_args(pairs.next().unwrap())?));
                pairs.next(); // )
            }
            e => panic!("{:?}", e),
        }
    }

    Ok(Node::Unary(Box::new(UnaryNode::Suffix(primary, ops))))
}

pub fn parse_args(pair: Pair<Rule>) -> Result<Vec<Node>, NodeError> {
    let mut pairs = pair.into_inner();
    let mut args = vec![];

    while let Some(pair) = pairs.next() {
        args.push(parse_expr_node(pair)?);
    }

    Ok(args)
}

#[test]
fn test_unary() {
    assert!(parse_unary_node(
        CBCScanner::parse(Rule::UNARY, "year->month->week.day++")
            .unwrap()
            .next()
            .unwrap()
    )
    .is_ok());
}
