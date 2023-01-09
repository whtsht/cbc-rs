use pest::iterators::Pair;

use self::{block::parse_block, if_stmt::parse_if_stmt};

use super::*;

mod block;
mod if_stmt;
use block::DefVars;

#[derive(Debug)]
pub enum StmtNode {
    None,
    Expr(Node),
    Block {
        def_var_list: Vec<DefVars>,
        stmts: Vec<Node>,
    },
    If {
        expr: Node,
        then: Node,
        _else: Node,
    },
}

pub fn parse_stmts(pair: Pair<Rule>) -> Result<Vec<Node>, NodeError> {
    let mut pairs = pair.into_inner();

    let mut stmts = vec![];
    while let Some(pair) = pairs.next() {
        stmts.push(parse_stmt_node(pair)?);
    }
    Ok(stmts)
}

pub fn parse_stmt_node(pair: Pair<Rule>) -> Result<Node, NodeError> {
    let mut pairs = pair.into_inner().peekable();

    match pairs.peek().unwrap().as_rule() {
        Rule::SCOLON => {
            pairs.next();
            Ok(Node::Stmt(Box::new(StmtNode::None)))
        }
        Rule::BLOCK => Ok(Node::Stmt(Box::new(parse_block(pairs.next().unwrap())?))),
        Rule::EXPR => {
            let node = Ok(Node::Stmt(Box::new(StmtNode::Expr(parse_expr_node(
                pairs.next().unwrap(),
            )?))));
            pairs.next(); // Skip a semicolon
            node
        }
        Rule::IF_STMT => Ok(Node::Stmt(Box::new(parse_if_stmt(pairs.next().unwrap())?))),
        e => todo!("{:?}", e),
    }
}

#[test]
fn test_stmt() {
    assert!(parse_stmt_node(CBCScanner::parse(Rule::STMT, ";").unwrap().next().unwrap()).is_ok());
    assert!(parse_stmt_node(
        CBCScanner::parse(Rule::STMT, "a = 10;")
            .unwrap()
            .next()
            .unwrap()
    )
    .is_ok());
}
