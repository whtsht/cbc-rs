#![allow(dead_code)]
use crate::node::def::def_var::{DefVars, Var};
use crate::node::def::DefNode;
use crate::node::expr::ExprNode;
use crate::node::param::ParamsNode;
use crate::node::stmt::StmtNode;
use crate::node::type_::TypeNode;
use crate::node::Node;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::{Rc, Weak};

#[derive(Debug, Default)]
pub struct Scope {
    pub parent: RefCell<Weak<Scope>>,
    pub localscope: RefCell<Vec<Rc<Scope>>>,
    pub entities: RefCell<BTreeMap<String, Entity>>,
}

#[derive(Debug)]
pub enum Entity {
    Variable {
        _type: TypeNode,
        is_static: bool,
        init: Option<ExprNode>,
    },
    Function {
        return_type: TypeNode,
        is_static: bool,
        params: ParamsNode,
    },
}

pub fn gen_scope_toplevel(nodes: Vec<Node>, scope: Rc<Scope>, parent: Weak<Scope>) -> Rc<Scope> {
    if parent.upgrade().is_some() {
        *scope.parent.borrow_mut() = parent;
    }

    for node in nodes {
        match node {
            Node::Def(def_node) => match *def_node {
                DefNode::Vars(vars) => apply_vars(vars, &scope),
                DefNode::Fun {
                    is_static,
                    _type,
                    name,
                    params,
                    block,
                } => {
                    scope.entities.borrow_mut().insert(
                        name,
                        Entity::Function {
                            return_type: _type,
                            is_static,
                            params,
                        },
                    );

                    let local =
                        gen_scope_stmts(block, Rc::new(Scope::default()), Rc::downgrade(&scope));
                    (*scope.localscope.borrow_mut()).push(local);
                }
                _ => todo!(),
            },
            e => panic!("{:#?}", e),
        }
    }
    scope
}

pub fn gen_scope_stmts(nodes: Vec<StmtNode>, scope: Rc<Scope>, parent: Weak<Scope>) -> Rc<Scope> {
    if parent.upgrade().is_some() {
        *scope.parent.borrow_mut() = parent;
    }

    for node in nodes {
        match node {
            StmtNode::DefVars(vars) => {
                apply_vars(vars, &scope);
            }
            e => panic!("{:#?}", e),
        }
    }
    scope
}

pub fn apply_vars(vars: DefVars, scope: &Rc<Scope>) {
    for var in vars.vars {
        match var {
            Var::Init { name, expr } => {
                scope.entities.borrow_mut().insert(
                    name,
                    Entity::Variable {
                        _type: vars._type.clone(),
                        is_static: vars.is_static,
                        init: Some(expr),
                    },
                );
            }
            Var::Uninit { name } => {
                scope.entities.borrow_mut().insert(
                    name,
                    Entity::Variable {
                        _type: vars._type.clone(),
                        is_static: vars.is_static,
                        init: None,
                    },
                );
            }
        }
    }
}

#[test]
fn test_scope() {
    let nodes = crate::node::parse(
        r#"int a = 1, b = 2, c = 3;
        void main(void) {
            int d = 10;
            int e = a + d;
        }
           "#,
    )
    .unwrap();

    let scope = Rc::new(Scope::default());
    let scope_tree = gen_scope_toplevel(nodes, scope, Weak::new());
    assert!(scope_tree.localscope.borrow()[0]
        .entities
        .borrow()
        .get("d")
        .is_some());

    assert!(scope_tree.entities.borrow().get("a").is_some());
    assert!(scope_tree.entities.borrow().get("b").is_some());
    assert!(scope_tree.entities.borrow().get("c").is_some());
}
