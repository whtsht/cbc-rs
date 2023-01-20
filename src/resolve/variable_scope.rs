#![allow(dead_code)]
use crate::node::def::def_var::{DefVars, Var};
use crate::node::def::DefNode;
use crate::node::expr::ExprNode;
use crate::node::param::ParamsNode;
use crate::node::primary::PrimaryNode;
use crate::node::stmt::StmtNode;
use crate::node::term::TermNode;
use crate::node::type_::TypeNode;
use crate::node::unary::{SuffixOp, UnaryNode};
use crate::node::Node;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::{Rc, Weak};

#[derive(Debug)]
pub struct ResolverError {
    message: String,
}

#[derive(Debug, Default, Clone)]
pub struct Scope {
    pub parent: RefCell<Weak<Scope>>,
    pub localscope: RefCell<Vec<Rc<Scope>>>,
    pub entities: RefCell<BTreeMap<String, Entity>>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, PartialEq, Eq)]
pub enum EntityType {
    Variable,
    Function,
}

impl Entity {
    pub fn get_type(&self) -> EntityType {
        match self {
            Entity::Variable { .. } => EntityType::Variable,
            Entity::Function { .. } => EntityType::Function,
        }
    }
}

pub fn gen_scope_toplevel(
    nodes: &mut Vec<Node>,
    scope: Rc<Scope>,
    parent: Weak<Scope>,
    recursive: bool,
) -> Result<Rc<Scope>, ResolverError> {
    if parent.upgrade().is_some() {
        *scope.parent.borrow_mut() = parent;
    }

    for node in nodes {
        match node {
            Node::Def(def_node) => match def_node.as_mut() {
                DefNode::Vars(vars) => apply_vars(vars, &scope)?,
                DefNode::Fun {
                    is_static,
                    _type,
                    name,
                    params,
                    block,
                } => {
                    scope.entities.borrow_mut().insert(
                        name.clone(),
                        Entity::Function {
                            return_type: _type.clone(),
                            is_static: *is_static,
                            params: params.clone(),
                        },
                    );

                    if recursive {
                        let local = gen_scope_stmts(
                            block,
                            Rc::new(Scope::default()),
                            Rc::downgrade(&scope),
                        )?;
                        (*scope.localscope.borrow_mut()).push(local);
                    }
                }
                _ => todo!(),
            },
            e => panic!("{:#?}", e),
        }
    }
    Ok(scope)
}

pub fn get_ref(scope: &Rc<Scope>, name: &str, entity_type: EntityType) -> Option<Entity> {
    match scope.entities.borrow().get(name) {
        Some(e) if e.get_type() == entity_type => Some(e.clone()),
        Some(_) | None => {
            if let Some(parent) = scope.parent.borrow().upgrade() {
                get_ref(&parent, name, entity_type)
            } else {
                None
            }
        }
    }
}

pub fn gen_scope_stmts(
    nodes: &mut Vec<StmtNode>,
    scope: Rc<Scope>,
    parent: Weak<Scope>,
) -> Result<Rc<Scope>, ResolverError> {
    if parent.upgrade().is_some() {
        *scope.parent.borrow_mut() = parent;
    }

    for node in nodes {
        match node {
            StmtNode::DefVars(vars) => apply_vars(vars, &scope)?,
            StmtNode::Expr(expr) => {
                get_variables_expr(expr, &scope)?;
            }
            StmtNode::Return { expr } => {
                if let Some(expr) = expr {
                    get_variables_expr(expr, &scope)?;
                }
            }
            e => panic!("{:#?}", e),
        }
    }
    Ok(scope)
}

pub fn get_variables_expr(expr: &mut ExprNode, scope: &Rc<Scope>) -> Result<(), ResolverError> {
    match expr {
        ExprNode::Term(term) => {
            get_variables_term(term, scope)?;
        }
        ExprNode::Assign { term, expr } => {
            get_variables_term(term, scope)?;
            get_variables_expr(expr, scope)?;
        }
        ExprNode::AssignOp { op: _, term, expr } => {
            get_variables_term(term, scope)?;
            get_variables_expr(expr, scope)?;
        }
        ExprNode::BinaryOp { op: _, lhs, rhs } => {
            get_variables_expr(lhs, scope)?;
            get_variables_expr(rhs, scope)?;
        }
        ExprNode::TernaryOp {
            op: _,
            lhs,
            mhs,
            rhs,
        } => {
            get_variables_expr(lhs, scope)?;
            get_variables_expr(mhs, scope)?;
            get_variables_expr(rhs, scope)?;
        }
    }

    Ok(())
}

pub fn get_variables_term(term: &mut TermNode, scope: &Rc<Scope>) -> Result<(), ResolverError> {
    match term {
        TermNode::Cast(_, term) => get_variables_term(term, scope),
        TermNode::Unary(unary) => get_variables_unary(unary, scope),
    }
}

pub fn get_variables_unary(unary: &mut UnaryNode, scope: &Rc<Scope>) -> Result<(), ResolverError> {
    match unary {
        UnaryNode::Increment(unary)
        | UnaryNode::Decrement(unary)
        | UnaryNode::SizeofUnary(unary) => get_variables_unary(unary.as_mut(), scope),
        UnaryNode::Plus(term)
        | UnaryNode::Not(term)
        | UnaryNode::Minus(term)
        | UnaryNode::Tilde(term)
        | UnaryNode::Star(term)
        | UnaryNode::And(term) => get_variables_term(term, scope),
        UnaryNode::Suffix(primary, suffix) => resolve_suffixop(primary, suffix, scope),
        UnaryNode::Primary(primary) => get_variables_primary(primary, scope),
        _ => todo!(),
    }
}

pub fn resolve_suffixop(
    primary: &mut PrimaryNode,
    suffix: &mut SuffixOp,
    scope: &Rc<Scope>,
) -> Result<(), ResolverError> {
    match suffix {
        SuffixOp::None => Ok(()),
        SuffixOp::CallFu(args, suffix) => {
            for arg in args {
                get_variables_expr(arg, scope)?;
            }
            resolve_suffixop(primary, suffix, scope)
        }
        SuffixOp::Array(idx, suffix) => {
            get_variables_expr(idx, scope)?;
            resolve_suffixop(primary, suffix, scope)
        }
        _ => todo!(),
    }
}

pub fn get_variables_primary(
    primary: &mut PrimaryNode,
    scope: &Rc<Scope>,
) -> Result<(), ResolverError> {
    match primary {
        PrimaryNode::Identifier(name, _) => {
            if let Some(entity) = get_ref(scope, name, EntityType::Variable) {
                *primary = PrimaryNode::Identifier(name.clone(), Some(entity));
            } else {
                Err(ResolverError {
                    message: format!("{} not defined", name),
                })?;
            }
        }
        _ => {}
    }

    Ok(())
}

pub fn apply_vars(vars: &mut DefVars, scope: &Rc<Scope>) -> Result<(), ResolverError> {
    for var in vars.vars.iter_mut() {
        match var {
            Var::Init { name, expr } => {
                get_variables_expr(expr, scope)?;
                scope.entities.borrow_mut().insert(
                    name.clone(),
                    Entity::Variable {
                        _type: vars._type.clone(),
                        is_static: vars.is_static,
                        init: Some(expr.clone()),
                    },
                );
            }
            Var::Uninit { name } => {
                scope.entities.borrow_mut().insert(
                    name.clone(),
                    Entity::Variable {
                        _type: vars._type.clone(),
                        is_static: vars.is_static,
                        init: None,
                    },
                );
            }
        }
    }
    Ok(())
}

#[test]
fn test_scope_var() {
    let mut nodes = crate::node::parse(
        r#"int a = 1;
        int b = 2, c = 3;
        void main(void) {
            int d = 10;
            int f = 1, g = 2;
            a = d + f + g + b + c;
        }
           "#,
    )
    .unwrap();

    let scope = Rc::new(Scope::default());
    let scope_tree = gen_scope_toplevel(&mut nodes, scope, Weak::new(), true).unwrap();
    assert!(scope_tree.localscope.borrow()[0]
        .entities
        .borrow()
        .get("d")
        .is_some());

    assert!(scope_tree.entities.borrow().get("a").is_some());
    assert!(scope_tree.entities.borrow().get("b").is_some());
    assert!(scope_tree.entities.borrow().get("c").is_some());
}

#[test]
fn test_scope_fun() {
    let mut nodes = crate::node::parse(
        r#"
        int d(void) {
            return 1;
        }
        void main(void) {
            int a = d();
        }"#,
    )
    .unwrap();
    let scope = Rc::new(Scope::default());
    let scope_tree = gen_scope_toplevel(&mut nodes, scope, Weak::new(), true);
    assert!(scope_tree.is_ok());
}

#[test]
fn test_scope_top() {
    let mut nodes = crate::node::parse(
        r#"int a = 1;
        void main(void) {
            int d = 10;
            int f = 1, g = 2;
            a = d + f + g + b + c;
        }
        int b = 2, c = 3;
           "#,
    )
    .unwrap();

    let scope =
        gen_scope_toplevel(&mut nodes, Rc::new(Scope::default()), Weak::new(), false).unwrap();

    let scope_tree = gen_scope_toplevel(&mut nodes, scope, Weak::new(), true).unwrap();

    assert!(scope_tree.localscope.borrow()[0]
        .entities
        .borrow()
        .get("d")
        .is_some());

    assert!(scope_tree.entities.borrow().get("a").is_some());
    assert!(scope_tree.entities.borrow().get("b").is_some());
    assert!(scope_tree.entities.borrow().get("c").is_some());
}
