use crate::ir::Const;
use crate::ir::GenError;
use crate::ir::IRInfo;
use crate::node::def::def_var::DefVars;
use crate::node::def::def_var::Var;
use crate::node::expr::AssignOp;
use crate::node::expr::BinaryOp;
use crate::node::primary::PrimaryNode;
use crate::node::term::TermNode;
use crate::node::type_::TypeBaseNode;
use crate::node::unary::SuffixOp;
use crate::node::unary::UnaryNode;

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

pub fn transform_expr(expr: &ExprNode, info: &mut IRInfo) -> Result<(Vec<Stmt>, Expr), GenError> {
    let ret = match expr {
        ExprNode::Term(term) => transform_term(term, info),
        ExprNode::BinaryOp { op, lhs, rhs } => {
            transform_binaryop(op, lhs.as_ref(), rhs.as_ref(), info)
        }
        ExprNode::Assign { term, expr } => transform_assign(term, &expr, info),
        ExprNode::AssignOp { op, term, expr } => transform_assignop(op, term, expr, info),
        _ => Err(GenError {
            message: format!("{:?} is not a constant value", expr),
        }),
    };
    ret
}

pub fn transform_assign(
    term: &TermNode,
    expr: &ExprNode,
    info: &mut IRInfo,
) -> Result<(Vec<Stmt>, Expr), GenError> {
    let mut stmts = vec![];
    let (s, t) = transform_term(term, info)?;
    stmts.extend(s);
    let (s, e) = transform_expr(expr, info)?;
    stmts.extend(s);
    stmts.push(Stmt::Assign(address_of(t), e.clone()));
    Ok((stmts, e))
}

pub fn transform_assignop(
    op: &AssignOp,
    term: &TermNode,
    expr: &ExprNode,
    info: &mut IRInfo,
) -> Result<(Vec<Stmt>, Expr), GenError> {
    let mut stmts = vec![];
    let (s, t) = transform_term(term, info)?;
    stmts.extend(s);
    let (s, e) = transform_expr(expr, info)?;
    stmts.extend(s);

    match op {
        AssignOp::Add => stmts.push(Stmt::Assign(
            address_of(t.clone()),
            Expr::Bin(Op::Add, Box::new(t), Box::new(e.clone())),
        )),
        e => panic!("not yet implemented {:?}", e),
    }
    Ok((stmts, e))
}

pub fn transform_binaryop(
    op: &BinaryOp,
    lhs: &ExprNode,
    rhs: &ExprNode,
    info: &mut IRInfo,
) -> Result<(Vec<Stmt>, Expr), GenError> {
    let mut stmts = vec![];

    match op {
        BinaryOp::And => {
            // lhs && rhs
            // =>
            // int tmp = 0;
            // if (lhs) {
            //  tmp = rhs;
            // }
            // tmp
            //

            let scope = info.current_scope();
            let var = info.get_tmpvar(scope, TypeBaseNode::Int);
            stmts.push(Stmt::Assign(
                address_of(var.clone()),
                Expr::Const(Const::Int(0)),
            ));

            let (_stmts, expr) = transform_expr(rhs, info)?;
            stmts.extend(_stmts);
            let then_node = Stmt::Assign(address_of(var.clone()), expr);
            let _stmts = gen_if_stmt_(&lhs, &then_node, &None, info)?;
            stmts.extend(_stmts);

            return Ok((stmts, var));
        }
        _ => {}
    };

    let (s, rhs) = transform_expr(rhs, info)?;
    stmts.extend(s);
    let (s, lhs) = transform_expr(lhs, info)?;
    stmts.extend(s);

    match op {
        BinaryOp::Add => Ok((stmts, Expr::Bin(Op::Add, Box::new(lhs), Box::new(rhs)))),
        BinaryOp::Sub => Ok((stmts, Expr::Bin(Op::Sub, Box::new(lhs), Box::new(rhs)))),
        BinaryOp::Mod => Ok((stmts, Expr::Bin(Op::SMod, Box::new(lhs), Box::new(rhs)))),
        BinaryOp::Eq => Ok((stmts, Expr::Bin(Op::EQ, Box::new(lhs), Box::new(rhs)))),
        BinaryOp::Le => Ok((stmts, Expr::Bin(Op::SLteq, Box::new(lhs), Box::new(rhs)))),
        BinaryOp::Lt => Ok((stmts, Expr::Bin(Op::SLt, Box::new(lhs), Box::new(rhs)))),
        e => panic!("not yet implemented {:?}", e),
    }
}

pub fn transform_term(term: &TermNode, info: &mut IRInfo) -> Result<(Vec<Stmt>, Expr), GenError> {
    match term {
        TermNode::Cast(_, _) => Err(GenError {
            message: format!("{:?} is not a constant value", term),
        })?,
        TermNode::Unary(unary) => transform_unary(unary, info),
    }
}

pub fn transform_unary(
    unary: &UnaryNode,
    info: &mut IRInfo,
) -> Result<(Vec<Stmt>, Expr), GenError> {
    match unary {
        UnaryNode::Primary(primary) => Ok((vec![], transform_primary(primary)?)),
        UnaryNode::Plus(term) => transform_term(term, info),
        UnaryNode::Minus(term) => {
            let (stmts, expr) = transform_term(term, info)?;
            Ok((stmts, Expr::Uni(Op::UMinus, Box::new(expr))))
        }
        UnaryNode::Suffix(primary, suffix) => transform_suffix(primary, &suffix, info),
        _ => Err(GenError {
            message: format!("unary: {:?} is not a constant value", unary),
        }),
    }
}

pub fn transform_suffix(
    primary: &PrimaryNode,
    suffix: &SuffixOp,
    info: &mut IRInfo,
) -> Result<(Vec<Stmt>, Expr), GenError> {
    match suffix {
        SuffixOp::SuffixNone => Ok((vec![], transform_primary(primary)?)),
        SuffixOp::CallFu(args, _, entity) => {
            if let PrimaryNode::Identifier(name, _) = primary {
                if let Some(entity) = entity {
                    let mut stmts = vec![];
                    let mut nargs = vec![];
                    for arg in args {
                        let (s, a) = transform_expr(arg, info)?;
                        nargs.push(a);
                        stmts.extend(s);
                    }
                    Ok((stmts, Expr::Call(name.clone(), nargs, entity.clone())))
                } else {
                    panic!("{name} transform_suffix is not have a function entity",);
                }
            } else {
                panic!("transform_suffix failed");
            }
        }
        _ => todo!(),
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

pub fn transform_stmt(stmt: &StmtNode, info: &mut IRInfo) -> Result<Vec<Stmt>, GenError> {
    let stmts = match stmt {
        StmtNode::If { cond, then, _else } => gen_if_stmt(cond, then, _else, info)?,
        StmtNode::Expr(expr) => {
            let (mut stmts, expr) = transform_expr(expr, info)?;
            stmts.push(Stmt::ExprStmt(expr));
            stmts
        }
        StmtNode::Return { expr } => _return(expr, info)?,
        StmtNode::Block { stmts } => {
            let mut ret = vec![];
            for stmt in stmts {
                ret.extend(transform_stmt(stmt, info)?);
            }
            ret
        }
        StmtNode::While { cond, stmt } => gen_while_stmt(cond, stmt.as_ref(), info)?,
        StmtNode::DefVars(defvars) => gen_defvars_stmt(defvars, info)?,
        e => panic!("transform_stmt: {:?}", e),
    };

    Ok(stmts)
}

pub fn gen_defvars_stmt(defvars: &DefVars, info: &mut IRInfo) -> Result<Vec<Stmt>, GenError> {
    let mut stmts = vec![];

    for var in defvars.vars.iter() {
        match var {
            Var::Uninit { .. } => {}
            Var::Init { name, expr } => {
                let (mut _stmts, expr) = transform_expr(expr, info)?;
                _stmts.push(Stmt::Assign(
                    address_of(Expr::Var(
                        name.clone(),
                        info.current_scope()
                            .entities
                            .borrow()
                            .get(name)
                            .unwrap()
                            .clone(),
                    )),
                    expr,
                ));
                stmts.extend(_stmts);
            }
        }
    }

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

pub fn _return(expr: &Option<ExprNode>, info: &mut IRInfo) -> Result<Vec<Stmt>, GenError> {
    let ret = match expr {
        Some(expr) => {
            let (mut stmts, expr) = transform_expr(expr, info)?;
            stmts.push(Stmt::Return(Some(expr)));
            stmts
        }
        None => vec![Stmt::Return(None)],
    };
    Ok(ret)
}

pub fn cjump(
    cond: &ExprNode,
    then_label: &Label,
    else_label: &Label,
    info: &mut IRInfo,
) -> Result<Vec<Stmt>, GenError> {
    let (mut stmts, expr) = transform_expr(cond, info)?;
    stmts.push(Stmt::CJump {
        cond: expr,
        then_label: then_label.clone(),
        else_label: else_label.clone(),
    });
    Ok(stmts)
}

pub fn gen_if_stmt(
    cond: &ExprNode,
    then_node: &StmtNode,
    else_node: &StmtNode,
    info: &mut IRInfo,
) -> Result<Vec<Stmt>, GenError> {
    let mut ir = vec![];
    let then_label = info.new_label();
    let else_label = info.new_label();
    let end_label = info.new_label();

    if let StmtNode::None = else_node {
        ir.extend(cjump(cond, &then_label, &end_label, info)?);
        ir.push(label(&then_label));
        ir.extend(transform_stmt(then_node, info)?);
        ir.push(label(&end_label));
    } else {
        ir.extend(cjump(cond, &then_label, &else_label, info)?);
        ir.push(label(&then_label));
        ir.extend(transform_stmt(then_node, info)?);
        ir.push(jump(&end_label));
        ir.push(label(&else_label));
        ir.extend(transform_stmt(else_node, info)?);
        ir.push(label(&end_label));
    }

    Ok(ir)
}

pub fn gen_if_stmt_(
    cond: &ExprNode,
    then_node: &Stmt,
    else_node: &Option<Stmt>,
    info: &mut IRInfo,
) -> Result<Vec<Stmt>, GenError> {
    let mut ir = vec![];
    let then_label = info.new_label();
    let else_label = info.new_label();
    let end_label = info.new_label();

    if let Some(else_node) = else_node {
        ir.extend(cjump(cond, &then_label, &else_label, info)?);
        ir.push(label(&then_label));
        ir.push(then_node.clone());
        ir.push(jump(&end_label));
        ir.push(label(&else_label));
        ir.push(else_node.clone());
        ir.push(label(&end_label));
    } else {
        ir.extend(cjump(cond, &then_label, &end_label, info)?);
        ir.push(label(&then_label));
        ir.push(then_node.clone());
        ir.push(label(&end_label));
    }

    Ok(ir)
}

pub fn gen_while_stmt(
    cond: &ExprNode,
    stmt: &StmtNode,
    info: &mut IRInfo,
) -> Result<Vec<Stmt>, GenError> {
    let mut ir = vec![];
    let beg_label = info.new_label();
    let body_label = info.new_label();
    let end_label = info.new_label();

    ir.push(label(&beg_label));
    ir.extend(cjump(cond, &body_label, &end_label, info)?);
    ir.push(label(&body_label));
    info.push_continue(&beg_label);
    info.push_break(&end_label);

    ir.extend(transform_stmt(stmt, info)?);

    info.pop_continue();
    info.pop_break();
    ir.push(jump(&beg_label));
    ir.push(label(&end_label));

    Ok(ir)
}
