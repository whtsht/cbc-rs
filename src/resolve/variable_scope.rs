#![allow(dead_code)]

use crate::node::def::def_var::Var;
use crate::node::def::DefNode;
use crate::node::Node;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Scope<'a> {
    pub parent: Option<&'a Scope<'a>>,
    pub localscope: Vec<Scope<'a>>,
    pub entities: BTreeMap<String, Entity>,
}

#[derive(Debug)]
pub enum Entity {
    Variable {
        _type: Node,
        is_static: bool,
        init: Option<Node>,
    },
}

pub fn gen_scope_tree<'a>(
    nodes: Vec<Node>,
    mut scope: Scope<'a>,
    parent: Option<&'a Scope<'a>>,
) -> Scope<'a> {
    scope.parent = parent;

    for node in nodes {
        match node {
            Node::Def(def_node) => match *def_node {
                DefNode::Vars(vars) => {
                    for var in vars.vars {
                        match var {
                            Var::Init { name, expr } => {
                                scope.entities.insert(
                                    name,
                                    Entity::Variable {
                                        _type: vars._type.clone(),
                                        is_static: vars.is_static,
                                        init: Some(expr),
                                    },
                                );
                            }
                            Var::Uninit { name } => {
                                scope.entities.insert(
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
                _ => todo!(),
            },
            e => panic!("{:#?}", e),
        }
    }

    scope
}

#[test]
fn test_scope() {
    let parent = None;
    let nodes = crate::node::parse("int a, b, c = 1;").unwrap();
    let scope = Scope {
        parent: None,
        localscope: vec![],
        entities: BTreeMap::new(),
    };

    let scope_tree = gen_scope_tree(nodes, scope, parent);
    assert!(scope_tree.entities.get("a").is_some());
    assert!(scope_tree.entities.get("b").is_some());
    assert!(scope_tree.entities.get("c").is_some());
}
