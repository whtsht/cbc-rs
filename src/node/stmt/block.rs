use std::iter::Peekable;

use pest::iterators::Pairs;

use super::*;

#[derive(Debug)]
pub struct DefVars {
    _type: Node,
    is_static: bool,
    vars: Vec<Var>,
}

#[derive(Debug)]
pub enum Var {
    Uninit { name: String },
    Init { name: String, expr: Node },
}

pub fn parse_block(pair: Pair<Rule>) -> Result<StmtNode, NodeError> {
    let mut pairs = pair.into_inner();
    let def_var_list = def_var_list(pairs.next().unwrap())?;
    let stmts = parse_stmts(pairs.next().unwrap())?;
    Ok(StmtNode::Block {
        def_var_list,
        stmts,
    })
}

pub fn var(pairs: &mut Peekable<Pairs<Rule>>) -> Result<Var, NodeError> {
    let name = pairs.next().unwrap().as_str().into();
    if pairs.peek().map(|x| x.as_rule()) == Some(Rule::EQ) {
        pairs.next().unwrap(); // =
        Ok(Var::Init {
            name,
            expr: parse_expr_node(pairs.next().unwrap())?,
        })
    } else {
        Ok(Var::Uninit { name })
    }
}

pub fn def_vars(pair: Pair<Rule>) -> Result<DefVars, NodeError> {
    let mut pairs = pair.into_inner().peekable();
    let is_static = pairs.next().unwrap().into_inner().next().is_some();
    let _type = parse_type_node(pairs.next().unwrap())?;

    let mut vars = vec![];

    while pairs.peek().map(|x| x.as_rule()) == Some(Rule::NAME) {
        vars.push(var(&mut pairs)?);
    }

    pairs.next().unwrap(); // ;
    Ok(DefVars {
        _type,
        is_static,
        vars,
    })
}

pub fn def_var_list(pair: Pair<Rule>) -> Result<Vec<DefVars>, NodeError> {
    let mut pairs = pair.into_inner();
    let mut def_var_list = vec![];
    while let Some(pair) = pairs.next() {
        def_var_list.push(def_vars(pair)?);
    }

    Ok(def_var_list)
}

#[test]
fn test_block() {
    assert!(def_var_list(
        CBCScanner::parse(Rule::DEF_VAR_LIST, "static unsigned int a = 1, b;")
            .unwrap()
            .next()
            .unwrap(),
    )
    .is_ok());

    assert!(parse_block(
        CBCScanner::parse(Rule::BLOCK, "{ int a, b = 1; static long counter = 0;}")
            .unwrap()
            .next()
            .unwrap(),
    )
    .is_ok());
}