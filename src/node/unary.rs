use super::*;
use crate::resolve::variable_scope::Entity;
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
    Suffix(PrimaryNode, Box<SuffixOp>),
    Primary(PrimaryNode),
}

#[derive(Debug, Clone)]
pub enum SuffixOp {
    SuffixNone,
    Increment(Box<SuffixOp>),
    Decrement(Box<SuffixOp>),
    Dot(String, Box<SuffixOp>),
    Arrow(String, Box<SuffixOp>),
    Array(ExprNode, Box<SuffixOp>),
    CallFu(Vec<ExprNode>, Box<SuffixOp>, Option<Entity>),
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

fn suffix_op(mut pairs: Peekable<Pairs<Rule>>) -> Result<Box<SuffixOp>, NodeError> {
    let op = if let Some(pair) = pairs.next() {
        match pair.as_rule() {
            Rule::PPLUS => SuffixOp::Increment(suffix_op(pairs)?),
            Rule::MMINUS => SuffixOp::Decrement(suffix_op(pairs)?),
            Rule::DOT => {
                let name = pairs.next().unwrap().as_str().into();
                SuffixOp::Dot(name, suffix_op(pairs)?)
            }
            Rule::ARROW => {
                let name = pairs.next().unwrap().as_str().into();
                SuffixOp::Arrow(name, suffix_op(pairs)?)
            }
            Rule::LSB => {
                let idx = parse_expr_node(pairs.next().unwrap())?;
                pairs.next();
                SuffixOp::Array(idx, suffix_op(pairs)?)
            }
            Rule::LPT => {
                let args = pairs.next().unwrap();
                if args.as_rule() != Rule::RPT {
                    pairs.next();
                }
                SuffixOp::CallFu(parse_args(args)?, suffix_op(pairs)?, None)
            }
            e => panic!("{:?}", e),
        }
    } else {
        SuffixOp::SuffixNone
    };

    Ok(Box::new(op))
}

pub fn parse_suffix_node(mut pairs: Peekable<Pairs<Rule>>) -> Result<UnaryNode, NodeError> {
    let primary = parse_primary_node(pairs.next().unwrap())?;
    if pairs.peek().is_none() {
        Ok(UnaryNode::Primary(primary))
    } else {
        Ok(UnaryNode::Suffix(primary, suffix_op(pairs)?))
    }
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
