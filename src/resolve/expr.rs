use crate::node::def::def_fun::DefFun;
use crate::node::def::def_var::Var;
use crate::node::unary::SuffixOp;
use crate::node::{def::DefNode, stmt::StmtNode, Node};
use crate::node::{expr::ExprNode, primary::PrimaryNode, term::TermNode, unary::UnaryNode};

use super::variable_scope::{Entity, ResolverError};

pub fn dereference_check(nodes: &Vec<Node>) -> Result<(), ResolverError> {
    for node in nodes {
        match node {
            Node::Def(def) => match def.as_ref() {
                DefNode::Fun(DefFun { block, .. }) => {
                    for stmt in block {
                        match stmt {
                            StmtNode::Expr(expr) => {
                                if !assiment_check(&expr) {
                                    Err(ResolverError {
                                        message: format!(
                                            "invalid expression: LHS cannot be assigned.\n    {:?}",
                                            expr
                                        ),
                                    })?;
                                }
                                if !callable_check(&expr) {
                                    Err(ResolverError {
                                        message: format!("invalid expression: {:?}", expr),
                                    })?;
                                }
                            }
                            StmtNode::DefVars(defvar) => {
                                for var in defvar.vars.iter() {
                                    match var {
                                        Var::Init { expr, .. } => {
                                            if !assiment_check(&expr) {
                                                Err(ResolverError {
                                                    message: format!(
                                            "invalid expression: LHS cannot be assigned.\n    {:?}",
                                            expr
                                        ),
                                                })?;
                                            }
                                            if !callable_check(&expr) {
                                                Err(ResolverError {
                                                    message: format!(
                                                        "invalid expression: {:?}",
                                                        expr
                                                    ),
                                                })?;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    Ok(())
}

pub fn callable_check(expr: &ExprNode) -> bool {
    match expr {
        ExprNode::Term(term) => callable_term(term),
        ExprNode::Assign { term, expr } | ExprNode::AssignOp { term, expr, .. } => {
            callable_term(term) && callable_check(expr)
        }
        ExprNode::BinaryOp { lhs, rhs, .. } => callable_check(&lhs) && callable_check(&rhs),
        ExprNode::TernaryOp { lhs, mhs, rhs, .. } => {
            callable_check(&lhs) && callable_check(&mhs) && callable_check(&rhs)
        }
    }
}

pub fn callable_term(term: &TermNode) -> bool {
    match term {
        TermNode::Unary(unary) => callable_uanry(&unary),
        TermNode::Cast(_, term) => callable_term(term.as_ref()),
    }
}

pub fn callable_uanry(unary: &UnaryNode) -> bool {
    match unary {
        UnaryNode::Suffix(_, suffix) => match suffix.as_ref() {
            SuffixOp::CallFu(_, _, entity) => {
                if let Some(entity) = entity.as_ref() {
                    callable_entity(entity)
                } else {
                    false
                }
            }
            _ => false,
        },
        _ => false,
    }
}

pub fn callable_entity(entity: &Entity) -> bool {
    match entity {
        Entity::Function { .. } => true,
        Entity::Variable { _type, init, .. } => {
            if let Some(expr) = init {
                callable_check(expr)
            } else {
                false
            }
        }
        _ => false,
    }
}

pub fn assiment_check(expr: &ExprNode) -> bool {
    match expr {
        ExprNode::Assign { term, expr } | ExprNode::AssignOp { term, expr, .. } => {
            return is_variable_term(term) && assiment_check(expr);
        }
        _ => true,
    }
}

pub fn is_variable_term(term: &TermNode) -> bool {
    match term {
        TermNode::Unary(unary) => is_variable_unary(&unary),
        TermNode::Cast(_, term) => is_variable_term(term.as_ref()),
    }
}

pub fn is_variable_unary(unary: &UnaryNode) -> bool {
    match unary {
        UnaryNode::Primary(primary) => is_variable_primary(primary),
        UnaryNode::And(term) => is_variable_term(term),
        UnaryNode::Increment(unary) | UnaryNode::Decrement(unary) => is_variable_unary(unary),
        _ => todo!(),
    }
}

pub fn is_variable_primary(primary: &PrimaryNode) -> bool {
    match primary {
        PrimaryNode::Identifier(_, _) => true,
        _ => false,
    }
}

#[test]
fn test_validation() {
    use super::variable_scope::{gen_scope_toplevel, Scope};
    use std::rc::{Rc, Weak};

    let mut nodes = crate::node::parse(
        r#"
        void main(void) {
            1 = 0;
        }
           "#,
    )
    .unwrap();
    let scope =
        gen_scope_toplevel(&mut nodes, Rc::new(Scope::default()), Weak::new(), false).unwrap();
    let _scope_tree = gen_scope_toplevel(&mut nodes, scope, Weak::new(), true).unwrap();

    assert!(dereference_check(&nodes).is_err());

    let mut nodes = crate::node::parse(
        r#"
        void main(void) {
            int a = 0;
            int b = a;
            int c = b(2);
        }
           "#,
    )
    .unwrap();
    let scope =
        gen_scope_toplevel(&mut nodes, Rc::new(Scope::default()), Weak::new(), false).unwrap();
    let _scope_tree = gen_scope_toplevel(&mut nodes, scope, Weak::new(), true).unwrap();
    assert!(dereference_check(&nodes).is_err());

    let mut nodes = crate::node::parse(
        r#"
        int one(void) {
            return 1;
        }
        void main(void) {
            // ok
            int a = one();
            // error
            int b = "one"();
        }
           "#,
    )
    .unwrap();
    let scope =
        gen_scope_toplevel(&mut nodes, Rc::new(Scope::default()), Weak::new(), false).unwrap();
    let _scope_tree = gen_scope_toplevel(&mut nodes, scope, Weak::new(), true).unwrap();
    assert!(dereference_check(&nodes).is_err());
}
