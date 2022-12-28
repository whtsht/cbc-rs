#![allow(dead_code)]
use pest::{iterators::Pair, Parser};

use crate::{CBCScanner, Rule};

pub const PLUS: &str = "+";
pub const MINUS: &str = "-";
pub const STAR: &str = "*";
pub const SLASH: &str = "/";
pub const PERCENT: &str = "%";
pub const CARET: &str = "^";
pub const NOT: &str = "!";
pub const AND: &str = "&";
pub const OR: &str = "|";
pub const TILDE: &str = "~";
pub const AAND: &str = "&&";
pub const OOR: &str = "||";
pub const SHL: &str = "<<";
pub const SHR: &str = ">>";
pub const PLUSEQ: &str = "+=";
pub const MINUSEQ: &str = "-=";
pub const STAREQ: &str = "*=";
pub const SLASHEQ: &str = "/=";
pub const PERCENTEQ: &str = "%=";
pub const CARETEQ: &str = "^=";
pub const ANDEQ: &str = "&=";
pub const OREQ: &str = "|=";
pub const SHLEQ: &str = "<<=";
pub const RHLEQ: &str = ">>=";
pub const EQ: &str = "=";
pub const EEQ: &str = "==";
pub const NE: &str = "!=";
pub const GT: &str = ">";
pub const LT: &str = "<";
pub const GE: &str = ">=";
pub const LE: &str = "<=";
pub const QUESTION: &str = "?";
pub const PPLUS: &str = "++";
pub const MMINUS: &str = "--";
pub const DDDOT: &str = "...";

#[derive(Debug)]
pub enum Node {
    BinaryOp(Box<BinaryOpNode>),
    SuffixOp(Box<SuffixOpNode>),
    PrefixOp(Box<PrefixOpNode>),
    Primary(Box<PrimaryNode>),
}

#[derive(Debug)]
pub enum UnaryOpNode {
    Suffix(SuffixOpNode),
    Prefix(PrefixOpNode),
}

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

#[derive(Debug)]
pub struct BinaryOpNode {
    operator: &'static str,
    lhs: Node,
    rhs: Node,
}

#[derive(Debug)]
pub enum PrimaryNode {
    Integer(i64),
    String(String),
    Char(char),
    Identifier(String),
}

pub fn parse(src: &str) -> Result<Vec<Node>, pest::error::Error<Rule>> {
    let mut nodes = vec![];
    let pairs = CBCScanner::parse(Rule::EXPR, src)?;
    for pair in pairs {
        match pair.as_rule() {
            Rule::EXPR => nodes.push(parse_unary_node(pair.into_inner().next().unwrap())),
            _ => {}
        }
    }

    Ok(nodes)
}

pub fn parse_unary_node(pair: Pair<Rule>) -> Node {
    parse_prefix_node(pair)
}

pub fn parse_prefix_node(pair: Pair<Rule>) -> Node {
    let mut pair = pair.into_inner().peekable();
    match pair.peek().unwrap().as_str() {
        PPLUS => Node::PrefixOp(Box::new(PrefixOpNode {
            operator: PPLUS,
            expr: parse_unary_node(pair.nth(1).unwrap()),
        })),
        MMINUS => Node::PrefixOp(Box::new(PrefixOpNode {
            operator: MMINUS,
            expr: parse_unary_node(pair.nth(1).unwrap()),
        })),
        _ => parse_suffix_node(pair.nth(0).unwrap()),
    }
}

pub fn parse_suffix_node(pair: Pair<Rule>) -> Node {
    let mut pair = pair.into_inner();
    let expr = pair.next().unwrap();
    match pair.next().map(|x| x.as_str()) {
        Some(PPLUS) => Node::SuffixOp(Box::new(SuffixOpNode {
            operator: PPLUS,
            expr: parse_primary_node(expr),
        })),
        Some(MMINUS) => Node::SuffixOp(Box::new(SuffixOpNode {
            operator: MMINUS,
            expr: parse_primary_node(expr),
        })),
        Some(_) => todo!(),
        None => parse_primary_node(expr),
    }
}

pub fn parse_primary_node(pair: Pair<Rule>) -> Node {
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
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
            Node::Primary(Box::new(PrimaryNode::Char(s[0])))
        }
        Rule::IDENTIFIER => {
            let s = pair.as_str().into();
            Node::Primary(Box::new(PrimaryNode::Identifier(s)))
        }
        _ => panic!("not primary, found {:?}", pair.as_rule()),
    }
}

#[test]
fn test_parse() {
    assert!(parse("++1").is_ok());
    assert!(parse("1++").is_ok())
}
