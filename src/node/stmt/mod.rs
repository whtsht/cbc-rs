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

#[derive(Debug, Clone)]
pub enum StmtNode {
    None,
    Expr(ExprNode),
    Block {
        stmts: Vec<StmtNode>,
    },
    If {
        cond: ExprNode,
        then: Box<StmtNode>,
        _else: Box<StmtNode>,
    },
    While {
        cond: ExprNode,
        stmt: Box<StmtNode>,
    },
    DoWhile {
        cond: ExprNode,
        stmt: Box<StmtNode>,
    },
    For {
        init: ExprNode,
        cond: ExprNode,
        term: ExprNode,
        stmt: Box<StmtNode>,
    },
    Switch {
        cond: ExprNode,
        cases: Vec<(Vec<PrimaryNode>, Vec<StmtNode>)>,
        default: Option<Vec<StmtNode>>,
    },
    Break,
    Continue,
    Goto {
        label: String,
    },
    Return {
        expr: Option<ExprNode>,
    },
    DefVars(DefVars),
}

pub fn parse_stmts(pair: Pair<Rule>) -> Result<Vec<StmtNode>, NodeError> {
    let mut pairs = pair.into_inner();

    let mut stmts = vec![];
    while let Some(pair) = pairs.next() {
        stmts.push(parse_stmt_node(pair)?);
    }
    Ok(stmts)
}

pub fn parse_stmt_node(pair: Pair<Rule>) -> Result<StmtNode, NodeError> {
    let mut pairs = pair.into_inner().peekable();

    match pairs.peek().unwrap().as_rule() {
        Rule::SCOLON => {
            pairs.next();
            Ok(StmtNode::None)
        }
        Rule::BLOCK => Ok(parse_block(pairs.next().unwrap())?),
        Rule::EXPR => {
            let node = Ok(StmtNode::Expr(parse_expr_node(pairs.next().unwrap())?));
            pairs.next(); // Skip a semicolon
            node
        }
        Rule::IF_STMT => Ok(parse_if_stmt(pairs.next().unwrap())?),
        Rule::WHILE_STMT => Ok(parse_while_stmt(pairs.next().unwrap())?),
        Rule::DOWHILE_STMT => Ok(parse_dowhile_stmt(pairs.next().unwrap())?),
        Rule::FOR_STMT => Ok(parse_for_stmt(pairs.next().unwrap())?),
        Rule::SWITCH_STMT => Ok(parse_switch_stmt(pairs.next().unwrap())?),
        Rule::BREAK_STMT => Ok(parse_break_stmt(pairs.next().unwrap())?),
        Rule::GOTO_STMT => Ok(parse_goto_stmt(pairs.next().unwrap())?),
        Rule::RETURN_STMT => Ok(parse_return_stmt(pairs.next().unwrap())?),
        Rule::DEF_VARS => Ok(StmtNode::DefVars(parse_def_vars(pairs.next().unwrap())?)),
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
