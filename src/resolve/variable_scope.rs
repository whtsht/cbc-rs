#![allow(dead_code)]
use crate::node::def::def_var::{DefVars, Var};
use crate::node::def::{DefNode, Member};
use crate::node::expr::ExprNode;
use crate::node::param::ParamsNode;
use crate::node::primary::PrimaryNode;
use crate::node::stmt::StmtNode;
use crate::node::term::TermNode;
use crate::node::type_::{TypeBaseNode, TypeNode};
use crate::node::unary::{SuffixOp, UnaryNode};
use crate::node::Node;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::{Rc, Weak};

#[derive(Debug)]
pub struct ResolverError {
    pub message: String,
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
    Struct {
        member_list: Vec<Member>,
    },
    Union {
        member_list: Vec<Member>,
    },
    TypeDef {
        _type: TypeNode,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum EntityType {
    Variable,
    Function,
    Struct,
    Union,
    TypeDef,
}

impl Entity {
    pub fn get_type(&self) -> EntityType {
        match self {
            Entity::Variable { .. } => EntityType::Variable,
            Entity::Function { .. } => EntityType::Function,
            Entity::Struct { .. } => EntityType::Struct,
            Entity::Union { .. } => EntityType::Union,
            Entity::TypeDef { .. } => EntityType::TypeDef,
        }
    }
}

pub fn contain(scope: &Rc<Scope>, name: &str) -> Result<(), ResolverError> {
    if scope.entities.borrow().contains_key(name) {
        Err(ResolverError {
            message: format!("{} is already defined", name),
        })
    } else {
        Ok(())
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
                DefNode::Fun(fun) => {
                    if recursive {
                        let local = Rc::new(Scope::default());

                        match &fun.params {
                            ParamsNode::Void => {}
                            ParamsNode::Some { fixed, .. } => {
                                for param in fixed.iter() {
                                    local.entities.borrow_mut().insert(
                                        param.name.clone(),
                                        Entity::Variable {
                                            _type: param._type.clone(),
                                            is_static: false,
                                            init: None,
                                        },
                                    );
                                }
                            }
                        }

                        let local = gen_scope_stmts(&mut fun.block, local, Rc::downgrade(&scope))?;

                        scope.localscope.borrow_mut().push(local.clone());
                        fun.scope = Some(local);
                    } else {
                        contain(&scope, &fun.name)?;

                        scope.entities.borrow_mut().insert(
                            fun.name.clone(),
                            Entity::Function {
                                return_type: fun._type.clone(),
                                is_static: fun.is_static,
                                params: fun.params.clone(),
                            },
                        );
                    }
                }
                DefNode::Struct { name, member_list } => {
                    if !recursive {
                        contain(&scope, name)?;
                        scope.entities.borrow_mut().insert(
                            name.clone(),
                            Entity::Struct {
                                member_list: member_list.clone(),
                            },
                        );
                    } else {
                        for Member { _type, name: _ } in member_list.iter_mut() {
                            get_type_ref(&scope, _type)?;
                        }
                    }
                }
                DefNode::Union { name, member_list } => {
                    if !recursive {
                        contain(&scope, name)?;
                        scope.entities.borrow_mut().insert(
                            name.clone(),
                            Entity::Union {
                                member_list: member_list.clone(),
                            },
                        );
                    } else {
                        for Member { _type, name: _ } in member_list.iter_mut() {
                            get_type_ref(&scope, _type)?;
                        }
                    }
                }
                DefNode::Type { _type, ident } => {
                    if !recursive {
                        contain(&scope, &ident)?;
                        scope.entities.borrow_mut().insert(
                            ident.clone(),
                            Entity::TypeDef {
                                _type: _type.clone(),
                            },
                        );
                    } else {
                        get_type_ref(&scope, _type)?;
                    }
                }
                _ => todo!(),
            },
            _ => {}
        }
    }
    Ok(scope)
}

pub fn get_type_ref(scope: &Rc<Scope>, type_node: &mut TypeNode) -> Result<(), ResolverError> {
    match &mut type_node.base {
        TypeBaseNode::Struct(name, entity) => {
            if let Some(e) = get_ref(&scope, &name) {
                *entity = Some(Box::new(e));
            } else {
                Err(ResolverError {
                    message: format!("struct {} is not defined", name),
                })?;
            }
        }
        TypeBaseNode::Union(name, entity) => {
            if let Some(e) = get_ref(&scope, &name) {
                *entity = Some(Box::new(e));
            } else {
                Err(ResolverError {
                    message: format!("union {} is not defined", name),
                })?;
            }
        }
        TypeBaseNode::Identifier(name, entity) => {
            if let Some(e) = get_ref(&scope, &name) {
                *entity = Some(Box::new(e));
            } else {
                Err(ResolverError {
                    message: format!("type {} is not defined", name),
                })?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub fn get_ref(scope: &Rc<Scope>, name: &str) -> Option<Entity> {
    match scope.entities.borrow().get(name) {
        Some(e) => Some(e.clone()),
        None => {
            if let Some(parent) = scope.parent.borrow().upgrade() {
                get_ref(&parent, name)
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
        gen_scope_stmt(node, scope.clone(), Weak::new())?;
    }
    Ok(scope)
}

pub fn gen_scope_stmt(
    node: &mut StmtNode,
    scope: Rc<Scope>,
    parent: Weak<Scope>,
) -> Result<Rc<Scope>, ResolverError> {
    if parent.upgrade().is_some() {
        *scope.parent.borrow_mut() = parent;
    }

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
        StmtNode::If { cond, then, _else } => {
            get_variables_expr(cond, &scope)?;
            gen_scope_stmt(then, Rc::new(Scope::default()), Rc::downgrade(&scope))?;
            gen_scope_stmt(_else, Rc::new(Scope::default()), Rc::downgrade(&scope))?;
        }
        StmtNode::Block { stmts } => {
            gen_scope_stmts(stmts, Rc::new(Scope::default()), Rc::downgrade(&scope))?;
        }
        StmtNode::While { cond, stmt } => {
            get_variables_expr(cond, &scope)?;
            gen_scope_stmt(stmt, Rc::new(Scope::default()), Rc::downgrade(&scope))?;
        }
        e => panic!("{:#?}", e),
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
        SuffixOp::SuffixNone => Ok(()),
        SuffixOp::CallFu(args, s, entity) => {
            for arg in args {
                get_variables_expr(arg, scope)?;
            }
            if let PrimaryNode::Identifier(name, _) = primary {
                if let Some(e) = get_ref(scope, name) {
                    *entity = Some(e);
                }
            }
            resolve_suffixop(primary, s, scope)
        }
        SuffixOp::Array(idx, suffix) => {
            get_variables_expr(idx, scope)?;
            resolve_suffixop(primary, suffix, scope)
        }
        SuffixOp::Dot(_, suffix) => resolve_suffixop(primary, suffix, scope),
        e => panic!("{:?}", e),
    }
}

pub fn get_variables_primary(
    primary: &mut PrimaryNode,
    scope: &Rc<Scope>,
) -> Result<(), ResolverError> {
    match primary {
        PrimaryNode::Identifier(name, _) => {
            if let Some(entity) = get_ref(scope, name) {
                *primary = PrimaryNode::Identifier(name.clone(), Some(entity));
            } else {
                Err(ResolverError {
                    message: format!("{} is not defined", name),
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
                get_type_ref(scope, &mut vars._type)?;
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
                get_type_ref(scope, &mut vars._type)?;
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
            int[4] a;
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
        int add(int a, int b) {
            return a + b;
        }
        void main(void) {
            int a = add(2, 4);
        }"#,
    )
    .unwrap();
    let scope = Rc::new(Scope::default());
    let scope_tree = gen_scope_toplevel(&mut nodes, scope, Weak::new(), true);
    println!("{:#?}", scope_tree);
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
        /* hello */
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

#[test]
fn test_scope_struct_union() {
    let mut nodes = crate::node::parse(
        r#"
        struct A {
            long a;
            int b;
        }

        union B {
            int a;
            unsigned long b;
        }

        typedef union B unionC;
        typedef unionC unionD;

        void main(void) {
            struct A a;
            a.a = 1;
            a.b = 2;


            union B b;
            b.a = 2;
            b.b = 3;

            unionC c;
            c.a = 2;

            unionD d;
            d.b = 3;
        }
        "#,
    )
    .unwrap();
    let scope =
        gen_scope_toplevel(&mut nodes, Rc::new(Scope::default()), Weak::new(), false).unwrap();

    let scope_tree = gen_scope_toplevel(&mut nodes, scope, Weak::new(), true).unwrap();
    assert!(scope_tree.entities.borrow().get("A").is_some());
    assert!(scope_tree.entities.borrow().get("B").is_some());
    assert!(scope_tree.entities.borrow().get("unionC").is_some());
    assert!(scope_tree.entities.borrow().get("unionD").is_some());
}
