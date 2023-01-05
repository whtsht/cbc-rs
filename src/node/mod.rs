#![allow(dead_code)]
use pest::iterators::Pair;
use std::iter::Peekable;

use pest::{iterators::Pairs, Parser};

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
pub enum TermNode {
    Cast(Node, Box<Node>),
    Unary(Node),
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
pub struct SizeofExprNode {
    pub expr: Node,
}

#[derive(Debug)]
pub struct SizeofTypeNode {
    pub _type: Node,
}

#[derive(Debug)]
pub enum PrimaryNode {
    Integer(i64),
    String(String),
    Char(char),
    Identifier(String),
}

#[derive(Debug)]
pub struct TypeNode {
    base: TypeBaseNode,
    suffix: TypeSuffix,
}

#[derive(Debug)]
pub enum TypeSuffix {
    Array,
    ArrayWithValue(i32),
    Pointer,
    Params(ParamsNode),
}

#[derive(Debug)]
pub enum ParamsNode {
    Void,
    Some { fixed: Vec<Param>, variable: bool },
}

#[derive(Debug)]
pub struct Param {
    _type: TypeNode,
    name: String,
}

#[derive(Debug)]
pub enum TypeBaseNode {
    Void,
    Char,
    Short,
    Int,
    Long,
    UnsignedChar,
    UnsignedShort,
    UnsignedInt,
    UnsignedLong,
    Struct(String),
    Union(String),
    Identifier(String),
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
            Node::Primary(Box::new(PrimaryNode::Char(s[0])))
        }
        Rule::IDENTIFIER => {
            let s = pair.as_str().into();
            Node::Primary(Box::new(PrimaryNode::Identifier(s)))
        }
        _ => panic!("not primary, found {:?}", pair.as_rule()),
    };

    Ok(node)
}

pub fn parse_sizeof_node(mut pairs: Peekable<Pairs<Rule>>) -> Result<Node, NodeError> {
    let node = match pairs.peek().unwrap().as_rule() {
        Rule::LBRACKET => {
            let node = Node::SizeofTypeNode(Box::new(SizeofTypeNode {
                _type: parse_type_node(pairs.nth(1).unwrap())?,
            }));
            pairs.next().unwrap(); // Skip the right bracket
            node
        }
        Rule::UNARY => Node::SizeofExprNode(Box::new(SizeofExprNode {
            expr: parse_unary_node(pairs.next().unwrap())?,
        })),
        err => panic!("sizeof error: {:?}", err),
    };

    Ok(node)
}

pub fn parse_term_node(mut pairs: Peekable<Pairs<Rule>>) -> Result<Node, NodeError> {
    let mut pairs = pairs.next().unwrap().into_inner().peekable();
    let node = match pairs.peek().unwrap().as_rule() {
        Rule::LBRACKET => {
            pairs.next().unwrap(); // Skip the left bracket
            let type_node = pairs.next().unwrap();
            pairs.next().unwrap(); // Skip the right bracket
            let node = Node::Term(Box::new(TermNode::Cast(
                parse_type_node(type_node)?,
                Box::new(parse_term_node(pairs)?),
            )));
            node
        }
        Rule::UNARY => Node::SizeofExprNode(Box::new(SizeofExprNode {
            expr: parse_unary_node(pairs.next().unwrap())?,
        })),
        err => panic!("term error: {:?}", err),
    };

    Ok(node)
}

pub fn parse_type_node(pair: Pair<Rule>) -> Result<Node, NodeError> {
    let pair = pair.into_inner().next().unwrap();
    parse_typebase_node(pair)
}

pub fn parse_typebase_node(pair: Pair<Rule>) -> Result<Node, NodeError> {
    let mut pairs = pair.into_inner();
    let first = pairs.next().unwrap();
    let second = pairs.peek();
    match (first.as_rule(), second.map(|x| x.as_rule())) {
        (Rule::VOID, None) => Ok(Node::TypeBase(Box::new(TypeBaseNode::Void))),
        (Rule::CHAR, None) => Ok(Node::TypeBase(Box::new(TypeBaseNode::Char))),
        (Rule::SHORT, None) => Ok(Node::TypeBase(Box::new(TypeBaseNode::Short))),
        (Rule::INT, None) => Ok(Node::TypeBase(Box::new(TypeBaseNode::Int))),
        (Rule::LONG, None) => Ok(Node::TypeBase(Box::new(TypeBaseNode::Long))),
        (Rule::UNSIGNED, Some(Rule::CHAR)) => {
            pairs.next();
            Ok(Node::TypeBase(Box::new(TypeBaseNode::UnsignedChar)))
        }
        (Rule::UNSIGNED, Some(Rule::SHORT)) => {
            pairs.next();
            Ok(Node::TypeBase(Box::new(TypeBaseNode::UnsignedShort)))
        }
        (Rule::UNSIGNED, Some(Rule::INT)) => {
            pairs.next();
            Ok(Node::TypeBase(Box::new(TypeBaseNode::UnsignedInt)))
        }
        (Rule::UNSIGNED, Some(Rule::LONG)) => {
            pairs.next();
            Ok(Node::TypeBase(Box::new(TypeBaseNode::UnsignedLong)))
        }
        (Rule::STRUCT, Some(Rule::IDENTIFIER)) => {
            let ident = pairs.next().unwrap().into_inner().next().unwrap().as_str();
            Ok(Node::TypeBase(Box::new(TypeBaseNode::Struct(ident.into()))))
        }
        err => Err(NodeError {
            _type: NodeErrorType::TypeBase,
            message: format!("typebase error: {:?}", err),
        }),
    }
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
