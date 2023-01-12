use super::super::*;
use pest::iterators::Pair;
use pest::iterators::Pairs;
use std::iter::Peekable;

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

pub fn parse_def_vars_list(pair: Pair<Rule>) -> Result<Vec<DefVars>, NodeError> {
    let mut pairs = pair.into_inner();
    let mut def_var_list = vec![];
    while let Some(pair) = pairs.next() {
        def_var_list.push(def_vars(pair)?);
    }

    Ok(def_var_list)
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

#[test]
fn test_def_var() {
    assert!(parse_def_vars_list(
        CBCScanner::parse(Rule::DEF_VARS_LIST, "static unsigned int a = 1, b;")
            .unwrap()
            .next()
            .unwrap(),
    )
    .is_ok());

    assert!(parse_stmt_node(
        CBCScanner::parse(
            Rule::STMT,
            r#"while (foo > 0) {
                    int y0 = y;
                    y = x;
                    x += y0;
                }
                "#
        )
        .unwrap()
        .next()
        .unwrap()
    )
    .is_ok());

    use crate::node::def::parse_def_node;
    assert!(parse_def_node(
        CBCScanner::parse(Rule::TOP_DEF, r#"int global = 10;"#)
            .unwrap()
            .next()
            .unwrap()
    )
    .is_ok());
}
