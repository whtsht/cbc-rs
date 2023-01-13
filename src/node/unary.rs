use super::*;
use crate::Rule;
use pest::iterators::{Pair, Pairs};
use std::iter::Peekable;

#[derive(Debug, Clone)]
pub enum UnaryNode {
    Increment(Box<UnaryNode>),
    Decrement(Box<UnaryNode>),
    Plus(TermNode),
    Minus(TermNode),
    Tilde(TermNode),
    Not(TermNode),
    Star(TermNode),
    And(TermNode),
    SizeofUnary(Box<UnaryNode>),
    SizeofType(TypeNode),
    Suffix(PrimaryNode, Vec<SuffixOp>),
}

#[derive(Debug, Clone)]
pub enum SuffixOp {
    Increment,
    Decrement,
    Dot(String),
    Arrow(String),
    Array(ExprNode),
    CallFu(Vec<ExprNode>),
}

pub fn parse_unary_node(pair: Pair<Rule>) -> Result<UnaryNode, NodeError> {
    assert_eq!(pair.as_rule(), Rule::UNARY);
    let mut pairs = pair.into_inner().peekable();
    let node = match pairs.peek().unwrap().as_rule() {
        Rule::PPLUS => UnaryNode::Increment(Box::new(parse_unary_node(pairs.nth(1).unwrap())?)),
        Rule::MMINUS => UnaryNode::Decrement(Box::new(parse_unary_node(pairs.nth(1).unwrap())?)),
        Rule::PLUS => UnaryNode::Plus(parse_term_node(pairs.nth(1).unwrap())?),
        Rule::MINUS => UnaryNode::Minus(parse_term_node(pairs.nth(1).unwrap())?),
        Rule::NOT => UnaryNode::Not(parse_term_node(pairs.nth(1).unwrap())?),
        Rule::TILDE => UnaryNode::Tilde(parse_term_node(pairs.nth(1).unwrap())?),
        Rule::AND => UnaryNode::And(parse_term_node(pairs.nth(1).unwrap())?),
        Rule::STAR => UnaryNode::Star(parse_term_node(pairs.nth(1).unwrap())?),
        Rule::SIZEOF => {
            pairs.next();
            parse_sizeof_node(pairs)?
        }
        Rule::POSTFIX => parse_suffix_node(pairs.next().unwrap().into_inner().peekable())?,
        _ => todo!(),
    };
    Ok(node)
}

pub fn parse_suffix_node(mut pairs: Peekable<Pairs<Rule>>) -> Result<UnaryNode, NodeError> {
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

    Ok(UnaryNode::Suffix(primary, ops))
}

pub fn parse_args(pair: Pair<Rule>) -> Result<Vec<ExprNode>, NodeError> {
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
