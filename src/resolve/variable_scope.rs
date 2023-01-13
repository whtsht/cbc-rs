#![allow(dead_code)]
use crate::node::def::def_var::{DefVars, Var};
use crate::node::def::DefNode;
use crate::node::stmt::StmtNode;
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
        _type: Node,
        is_static: bool,
        init: Option<Node>,
    },
    Function {
        return_type: Node,
        is_static: bool,
        params: Node,
    },
}

pub fn gen_scope_tree(nodes: Vec<Node>, scope: Rc<Scope>, parent: Weak<Scope>) -> Rc<Scope> {
    if parent.upgrade().is_some() {
        *scope.parent.borrow_mut() = parent;
    }

    let apply_vars = |vars: DefVars| {
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
    };

    for node in nodes {
        match node {
            Node::Def(def_node) => match *def_node {
                DefNode::Vars(vars) => apply_vars(vars),
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
                        gen_scope_tree(block, Rc::new(Scope::default()), Rc::downgrade(&scope));
                    (*scope.localscope.borrow_mut()).push(local);
                }
                _ => todo!(),
            },
            Node::Stmt(stmt) => match *stmt {
                StmtNode::DefVars(vars) => {
                    apply_vars(vars);
                }
                e => panic!("{:#?}", e),
            },
            e => panic!("{:#?}", e),
        }
    }
    scope
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
    let scope_tree = gen_scope_tree(nodes, scope, Weak::new());
    assert!(scope_tree.localscope.borrow()[0]
        .entities
        .borrow()
        .get("d")
        .is_some());

    assert!(scope_tree.entities.borrow().get("a").is_some());
    assert!(scope_tree.entities.borrow().get("b").is_some());
    assert!(scope_tree.entities.borrow().get("c").is_some());
}
