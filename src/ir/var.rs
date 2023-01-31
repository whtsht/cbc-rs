use crate::node::{
    def::def_var::{DefVars, Var},
    expr::ExprNode,
    primary::PrimaryNode,
    term::TermNode,
    unary::UnaryNode,
};

use super::{Const, DefinedVar, GenError};

pub fn gen_def_var(var: &DefVars) -> Result<Vec<DefinedVar>, GenError> {
    let mut dvars = vec![];
    for v in var.vars.iter() {
        match v {
            Var::Init { name, expr } => dvars.push(DefinedVar {
                name: name.clone(),
                _type: var._type.clone(),
                is_private: var.is_static,
                init: Some(get_const_expr(expr)?),
            }),
            Var::Uninit { name } => dvars.push(DefinedVar {
                name: name.clone(),
                _type: var._type.clone(),
                is_private: var.is_static,
                init: None,
            }),
        }
    }

    Ok(dvars)
}

pub fn get_const_expr(expr: &ExprNode) -> Result<Const, GenError> {
    match expr {
        ExprNode::Term(term) => get_const_term(term),
        _ => Err(GenError {
            message: format!("{:?} is not a constant value", expr),
        }),
    }
}

pub fn get_const_term(term: &TermNode) -> Result<Const, GenError> {
    match term {
        TermNode::Cast(_, _) => Err(GenError {
            message: format!("{:?} is not a constant value", term),
        }),
        TermNode::Unary(unary) => get_const_unary(unary),
    }
}

pub fn get_const_unary(unary: &UnaryNode) -> Result<Const, GenError> {
    match unary {
        UnaryNode::Primary(primary) => get_const_primary(primary),
        _ => Err(GenError {
            message: format!("{:?} is not a constant value", unary),
        }),
    }
}

pub fn get_const_primary(primary: &PrimaryNode) -> Result<Const, GenError> {
    match primary {
        PrimaryNode::Char(c) => Ok(Const::Int(*c as i32)),
        PrimaryNode::String(s) => Ok(Const::Str(s.clone())),
        PrimaryNode::Integer(i) => Ok(Const::Int(*i as i32)),
        PrimaryNode::Identifier(_, _) => Err(GenError {
            message: format!("{:?} is not a constant value", primary),
        }),
    }
}
