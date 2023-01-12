use self::{
    block::parse_block, break_stmt::parse_break_stmt, dowhile_stmt::parse_dowhile_stmt,
    for_stmt::parse_for_stmt, goto_stmt::parse_goto_stmt, if_stmt::parse_if_stmt,
    return_stmt::parse_return_stmt, switch_stmt::parse_switch_stmt, while_stmt::parse_while_stmt,
};
use super::def::def_var::*;
use super::*;
use pest::iterators::Pair;

pub mod block;
mod break_stmt;
mod continue_stmt;
mod dowhile_stmt;
mod for_stmt;
mod goto_stmt;
mod if_stmt;
mod return_stmt;
mod switch_stmt;
mod while_stmt;

#[derive(Debug)]
pub enum StmtNode {
    None,
    Expr(Node),
    Block {
        stmts: Vec<Node>,
    },
    If {
        cond: Node,
        then: Node,
        _else: Node,
    },
    While {
        cond: Node,
        stmt: Node,
    },
    DoWhile {
        cond: Node,
        stmt: Node,
    },
    For {
        init: Node,
        cond: Node,
        term: Node,
        stmt: Node,
    },
    Switch {
        cond: Node,
        cases: Vec<(Vec<Node>, Node)>,
        default: Option<Node>,
    },
    Break,
    Continue,
    Goto {
        label: String,
    },
    Return {
        expr: Option<Node>,
    },
    DefVarsList(Vec<DefVars>),
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
        Rule::WHILE_STMT => Ok(Node::Stmt(Box::new(parse_while_stmt(
            pairs.next().unwrap(),
        )?))),
        Rule::DOWHILE_STMT => Ok(Node::Stmt(Box::new(parse_dowhile_stmt(
            pairs.next().unwrap(),
        )?))),
        Rule::FOR_STMT => Ok(Node::Stmt(Box::new(parse_for_stmt(pairs.next().unwrap())?))),
        Rule::SWITCH_STMT => Ok(Node::Stmt(Box::new(parse_switch_stmt(
            pairs.next().unwrap(),
        )?))),
        Rule::BREAK_STMT => Ok(Node::Stmt(Box::new(parse_break_stmt(
            pairs.next().unwrap(),
        )?))),
        Rule::GOTO_STMT => Ok(Node::Stmt(Box::new(parse_goto_stmt(
            pairs.next().unwrap(),
        )?))),
        Rule::RETURN_STMT => Ok(Node::Stmt(Box::new(parse_return_stmt(
            pairs.next().unwrap(),
        )?))),
        Rule::DEF_VARS_LIST => Ok(Node::Stmt(Box::new(StmtNode::DefVarsList(
            parse_def_vars_list(pairs.next().unwrap())?,
        )))),
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
