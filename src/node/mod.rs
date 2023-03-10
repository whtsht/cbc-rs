#![allow(dead_code)]

pub mod def;
pub mod expr;
pub mod extern_;
pub mod import;
pub mod param;
pub mod primary;
pub mod sizeof;
pub mod stmt;
pub mod term;
pub mod type_;
pub mod unary;

use self::def::parse_topdef_node;
use self::def::DefNode;
use self::expr::*;
use self::extern_::parse_prototypefun;
use self::extern_::PrototypeFun;
use self::import::parse_import_node;
use self::import::ImportNode;
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

#[derive(Debug, Clone)]
pub enum Node {
    Def(Box<DefNode>),
    Import(Box<ImportNode>),
    Extern(Box<PrototypeFun>),
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
    Primary,
    Type,
    Unary,
    Term,
    Params,
    Expr,
    Stmt,
    Def,
    Import,
}

pub fn parse(src: &str) -> Result<Vec<Node>, NodeError> {
    let mut nodes = vec![];
    let mut pairs = CBCScanner::parse(Rule::FILE, src)
        .or_else(|e| {
            Err(NodeError {
                _type: NodeErrorType::Token,
                message: format!("failed to scan {:?}", e),
            })
        })?
        .next()
        .unwrap()
        .into_inner();

    while let Some(pair) = pairs.next() {
        match pair.as_rule() {
            Rule::IMPORT_STMT => nodes.push(parse_import_node(pair)?),
            Rule::TOP_DEF => nodes.push(parse_topdef_node(pair)?),
            Rule::EXTERN_STMT => nodes.push(parse_prototypefun(&mut pair.into_inner())?),
            Rule::EOI => break,
            e => panic!("{:?}", e),
        }
    }

    Ok(nodes)
}

#[test]
fn test_parse() {
    assert!(parse(
        r#" import stdio;
                int main (void) {
                    exit(1);
                    return 0;
                }"#
    )
    .is_ok());
    assert!(parse(
        r#" import stdio;
                import stdlib;
                int main (int argc, char **argv) {
                    int i, j = 5;
                    if (i) {
                        return (j * 1 - j);
                    } else {
                        exit(1);
                    }
                }"#
    )
    .is_ok());
}
