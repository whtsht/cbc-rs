use crate::node::{def::DefNode, stmt::StmtNode, Node};
use crate::node::{expr::ExprNode, primary::PrimaryNode, term::TermNode, unary::UnaryNode};

use super::variable_scope::ResolverError;

pub fn assignmentable(expr: &ExprNode) -> Result<(), ResolverError> {
    match expr {
        ExprNode::Assign { term, .. } | ExprNode::AssignOp { term, .. } => {
            if !is_variable_term(term) {
                Err(ResolverError {
                    message: format!(
                        "invalid expression: LHS cannot be assigned.\n    {:?}",
                        expr
                    ),
                })?;
            }
        }
        _ => {}
    }
    Ok(())
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

pub fn dereference_check(nodes: &Vec<Node>) -> Result<(), ResolverError> {
    for node in nodes {
        match node {
            Node::Def(def) => match def.as_ref() {
                DefNode::Fun { block, .. } => {
                    for stmt in block {
                        match stmt {
                            StmtNode::Expr(expr) => {
                                assignmentable(&expr)?;
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
}
