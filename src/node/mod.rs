#![allow(dead_code)]

mod param;
mod primary;
mod sizeof;
mod term;
mod type_;
mod unary;

use self::param::ParamsNode;
use self::primary::*;
use self::sizeof::*;
use self::term::*;
use self::type_::*;
use self::unary::*;

use self::term::parse_term_node;
use pest::Parser;

use crate::{CBCScanner, Rule};

pub const SIZEOF: &str = "sizeof";
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
    TypeBase(Box<TypeBaseNode>),
    SizeofExprNode(Box<SizeofExprNode>),
    SizeofTypeNode(Box<SizeofTypeNode>),
    Term(Box<TermNode>),
    Params(Box<ParamsNode>),
}

#[derive(Debug)]
pub struct NodeError {
    _type: NodeErrorType,
    message: String,
}

#[derive(Debug)]
pub enum NodeErrorType {
    Token,
    BinaryOp,
    SuffixOp,
    PrefixOp,
    Primary,
    TypeBase,
    SizeofExprNode,
    SizeofTypeNode,
}

#[derive(Debug)]
pub struct BinaryOpNode {
    operator: &'static str,
    lhs: Node,
    rhs: Node,
}

pub fn parse(src: &str) -> Result<Vec<Node>, NodeError> {
    let mut nodes = vec![];
    let pairs = CBCScanner::parse(Rule::EXPR, src).or_else(|_| {
        Err(NodeError {
            _type: NodeErrorType::Token,
            message: String::from("failed to scan"),
        })
    })?;

    for pair in pairs {
        match pair.as_rule() {
            Rule::EXPR => nodes.push(parse_term_node(pair.into_inner().peekable())?),
            _ => {}
        }
    }
    Ok(nodes)
}

#[test]
fn parse_unary() {
    assert!(parse("++1").is_ok());
    assert!(parse("1++").is_ok());
    assert!(parse("sizeof 1").is_ok());
    assert!(parse("sizeof (int)").is_ok());
    assert!(parse("sizeof(unsigned long)").is_ok());
    assert!(parse("(int)2").is_ok());
}
