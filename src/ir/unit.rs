use crate::ir::Const;
use crate::ir::GenError;
use crate::ir::IRInfo;
use crate::node::expr::BinaryOp;
use crate::node::primary::PrimaryNode;
use crate::node::term::TermNode;
use crate::node::unary::UnaryNode;
use crate::resolve::variable_scope::Scope;

use crate::node::{expr::ExprNode, stmt::StmtNode};

use super::Op;
use super::{Expr, Label, Stmt};

pub fn address_of(expr: Expr) -> Expr {
    match expr {
        Expr::Var(name, entity) => Expr::Addr(name, entity),
        Expr::Mem(expr) => *expr,
        e => panic!("{:?} is not have a address", e),
    }
}

pub fn transform_expr(expr: &ExprNode) -> Result<Expr, GenError> {
    match expr {
        ExprNode::Term(term) => transform_term(term),
        ExprNode::BinaryOp { op, lhs, rhs } => transform_binaryop(op, lhs.as_ref(), rhs.as_ref()),
        ExprNode::Assign { term, expr } => transform_assign(term, &expr),
        _ => Err(GenError {
            message: format!("{:?} is not a constant value", expr),
        }),
    }
}

pub fn transform_assign(term: &TermNode, expr: &ExprNode) -> Result<Expr, GenError> {
    Ok(Expr::Assign(
        Box::new(address_of(transform_term(term)?)),
        Box::new(transform_expr(expr)?),
    ))
}

pub fn transform_binaryop(op: &BinaryOp, lhs: &ExprNode, rhs: &ExprNode) -> Result<Expr, GenError> {
    let rhs = Box::new(transform_expr(rhs)?);
    let lhs = Box::new(transform_expr(lhs)?);
    match op {
        BinaryOp::Add => Ok(Expr::Bin(Op::Add, lhs, rhs)),
        BinaryOp::Sub => Ok(Expr::Bin(Op::Sub, lhs, rhs)),
        _ => todo!(),
    }
}

pub fn transform_term(term: &TermNode) -> Result<Expr, GenError> {
    match term {
        TermNode::Cast(_, _) => Err(GenError {
            message: format!("{:?} is not a constant value", term),
        }),
        TermNode::Unary(unary) => transform_unary(unary),
    }
}

pub fn transform_unary(unary: &UnaryNode) -> Result<Expr, GenError> {
    match unary {
        UnaryNode::Primary(primary) => transform_primary(primary),
        UnaryNode::Plus(term) => transform_term(term),
        UnaryNode::Minus(term) => Ok(Expr::Uni(Op::UMinus, Box::new(transform_term(term)?))),
        _ => Err(GenError {
            message: format!("{:?} is not a constant value", unary),
        }),
    }
}

pub fn transform_primary(primary: &PrimaryNode) -> Result<Expr, GenError> {
    match primary {
        PrimaryNode::Char(c) => Ok(Expr::Const(Const::Int(*c as i64))),
        PrimaryNode::String(s) => Ok(Expr::Const(Const::Str(s.clone()))),
        PrimaryNode::Integer(i) => Ok(Expr::Const(Const::Int(*i))),
        PrimaryNode::Identifier(name, entity) => {
            if let Some(entity) = entity {
                Ok(Expr::Var(name.clone(), entity.clone()))
            } else {
                Err(GenError {
                    message: format!("not found {}, this may be a compiler bug", name),
                })
            }
        }
    }
}

pub fn transform_stmt(
    stmt: &StmtNode,
    info: &mut IRInfo,
    scope: &Scope,
) -> Result<Vec<Stmt>, GenError> {
    let stmts = match stmt {
        StmtNode::If { cond, then, _else } => gen_if_stmt(cond, then, _else, info, scope)?,
        StmtNode::Expr(expr) => vec![Stmt::ExprStmt(transform_expr(expr)?)],
        StmtNode::Return { expr } => vec![_return(expr)?],
        StmtNode::Block { stmts } => {
            let mut ret = vec![];
            for stmt in stmts {
                ret.extend(transform_stmt(stmt, info, scope)?);
            }
            ret
        }
        StmtNode::While { cond, stmt } => gen_while_stmt(cond, stmt.as_ref(), info, scope)?,
        e => panic!("transform_stmt: {:?}", e),
    };

    Ok(stmts)
}

pub fn label(label: &Label) -> Stmt {
    Stmt::Label(label.clone())
}

pub fn jump(label: &Label) -> Stmt {
    Stmt::Jump {
        label: label.clone(),
    }
}

pub fn _return(expr: &Option<ExprNode>) -> Result<Stmt, GenError> {
    let ret = match expr {
        Some(expr) => Stmt::Return(Some(transform_expr(expr)?)),
        None => Stmt::Return(None),
    };
    Ok(ret)
}

pub fn cjump(cond: &ExprNode, then_label: &Label, else_label: &Label) -> Result<Stmt, GenError> {
    Ok(Stmt::CJump {
        cond: transform_expr(cond)?,
        then_label: then_label.clone(),
        else_label: else_label.clone(),
    })
}

pub fn gen_if_stmt(
    cond: &ExprNode,
    then_node: &StmtNode,
    else_node: &StmtNode,
    info: &mut IRInfo,
    scope: &Scope,
) -> Result<Vec<Stmt>, GenError> {
    let mut ir = vec![];
    let then_label = info.new_label();
    let else_label = info.new_label();
    let end_label = info.new_label();

    if let StmtNode::None = else_node {
        ir.push(cjump(cond, &then_label, &end_label)?);
        ir.push(label(&then_label));
        ir.extend(transform_stmt(then_node, info, scope)?);
        ir.push(label(&end_label));
    } else {
        ir.push(cjump(cond, &then_label, &else_label)?);
        ir.push(label(&then_label));
        ir.extend(transform_stmt(then_node, info, scope)?);
        ir.push(jump(&end_label));
        ir.push(label(&else_label));
        ir.extend(transform_stmt(else_node, info, scope)?);
        ir.push(label(&end_label));
    }

    Ok(ir)
}

pub fn gen_while_stmt(
    cond: &ExprNode,
    stmt: &StmtNode,
    info: &mut IRInfo,
    scope: &Scope,
) -> Result<Vec<Stmt>, GenError> {
    let mut ir = vec![];
    let beg_label = info.new_label();
    let body_label = info.new_label();
    let end_label = info.new_label();

    ir.push(label(&beg_label));
    ir.push(cjump(cond, &body_label, &end_label)?);
    ir.push(label(&body_label));
    info.push_continue(&beg_label);
    info.push_break(&end_label);

    ir.extend(transform_stmt(stmt, info, scope)?);

    info.pop_continue();
    info.pop_break();
    ir.push(jump(&beg_label));
    ir.push(label(&end_label));

    Ok(ir)
}
