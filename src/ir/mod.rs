use crate::resolve::variable_scope::{get_ref, Entity};
use std::rc::Rc;

pub mod fun;
pub mod unit;
pub mod var;

use crate::{
    node::{def::DefNode, param::ParamsNode, type_::TypeNode, Node},
    resolve::variable_scope::Scope,
};

use self::{fun::gen_def_fun, var::gen_def_var};

#[derive(Debug)]
pub struct GenError {
    pub message: String,
}

#[derive(Debug)]
pub struct DefinedFun {
    pub name: String,
    pub _type: (TypeNode, ParamsNode),
    pub is_private: bool,
    pub body: Vec<Stmt>,
}

#[derive(Debug)]
pub struct DefinedVar {
    pub name: String,
    pub _type: TypeNode,
    pub is_private: bool,
    pub init: Option<Const>,
}

#[derive(Debug)]
pub struct IR {
    pub var: Vec<DefinedVar>,
    pub fun: Vec<DefinedFun>,
}

#[derive(Debug)]
pub enum Stmt {
    Return(Option<Expr>),
    Jump {
        label: Label,
    },
    CJump {
        cond: Expr,
        then_label: Label,
        else_label: Label,
    },
    Switch,
    Label(Label),
    ExprStmt(Expr),
}

#[derive(Debug)]
pub enum Const {
    Int(i64),
    Str(String),
}

#[derive(Debug)]
pub enum Expr {
    Uni(Op, Box<Expr>),
    Bin(Op, Box<Expr>, Box<Expr>),
    Assign(Box<Expr>, Box<Expr>),
    Call,
    Addr(String, Entity),
    Mem(Box<Expr>),
    Var(String, Entity),
    Const(Const),
}

#[derive(Debug)]
pub enum Type {
    I8,
    I16,
    I32,
    I64,
}

#[derive(Debug)]
pub enum Op {
    Add,
    Sub,
    Mul,
    SDiv,
    UDiv,
    SMod,
    UMod,
    Not,
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    BitLShift,
    BitRShift,
    ArithRShift,
    EQ,
    NEQ,
    SGt,
    UGt,
    SGteq,
    UGteq,
    SLt,
    ULt,
    SLteq,
    ULteq,
    UMinus,
    SCast,
    UCast,
}

#[derive(Debug, Clone)]
pub struct Label(String);

#[derive(Debug)]
pub struct JumpEntry {}

#[derive(Debug)]
pub struct LabelGenerator {
    counter: u32,
}

impl LabelGenerator {
    pub fn new() -> Self {
        Self { counter: 0 }
    }

    pub fn new_label(&mut self) -> Label {
        self.counter += 1;
        Label(format!(".L{}", self.counter.to_string()))
    }
}

pub struct TmpVarGenerator {
    counter: u32,
}

impl TmpVarGenerator {
    pub fn new() -> Self {
        Self { counter: 0 }
    }

    pub fn new_tmpvar(&mut self, scope: &Rc<Scope>) -> String {
        loop {
            let name = format!("__tmp{}", self.counter);
            if get_ref(scope, &name).is_none() {
                return name;
            }
        }
    }
}

#[derive(Debug)]
pub struct IRInfo {
    pub scope_stack: Vec<Scope>,
    pub break_stack: Vec<Label>,
    pub continue_stack: Vec<Label>,
    pub label_gen: LabelGenerator,
}

impl IRInfo {
    pub fn new() -> Self {
        let scope_stack: Vec<Scope> = vec![];
        let break_stack: Vec<Label> = vec![];
        let continue_stack: Vec<Label> = vec![];
        let label_gen = LabelGenerator::new();

        IRInfo {
            scope_stack,
            break_stack,
            continue_stack,
            label_gen,
        }
    }

    pub fn new_label(&mut self) -> Label {
        self.label_gen.counter += 1;
        Label(format!(".L{}", self.label_gen.counter.to_string()))
    }

    pub fn push_continue(&mut self, label: &Label) {
        self.continue_stack.push(label.clone());
    }

    pub fn push_break(&mut self, label: &Label) {
        self.break_stack.push(label.clone());
    }

    pub fn pop_continue(&mut self) -> Label {
        self.continue_stack.pop().expect("continue stack is empty")
    }

    pub fn pop_break(&mut self) -> Label {
        self.break_stack.pop().expect("break stack is empty")
    }
}

pub fn gen_ir(nodes: Vec<Node>, scope: Rc<Scope>) -> Result<IR, GenError> {
    let mut ir = IR {
        fun: vec![],
        var: vec![],
    };
    let mut info = IRInfo::new();

    for node in nodes {
        match node {
            Node::Def(def) => match def.as_ref() {
                DefNode::Vars(def_var) => ir.var.extend(gen_def_var(def_var)?),
                DefNode::Fun(fun) => ir.fun.push(gen_def_fun(fun, scope.as_ref(), &mut info)?),
                _ => todo!(),
            },
            Node::Import(_) => {}
        }
    }

    Ok(ir)
}

#[test]
fn test_gen_ir() {
    use crate::resolve::variable_scope::{gen_scope_toplevel, Scope};
    use std::rc::{Rc, Weak};
    let mut nodes = crate::node::parse(
        r#"
        int a = 0;
        int main(void) {
            while (1) {

            }
            a = 2;
            if (a) {
                return 1 + 2;
            } else {
                return a + 2;
            }
        }
           "#,
    )
    .unwrap();

    let scope = Rc::new(Scope::default());
    let scope_tree = gen_scope_toplevel(&mut nodes, scope, Weak::new(), true).unwrap();

    let ir = gen_ir(nodes, scope_tree);
    println!("{:#?}", ir);

    let mut nodes = crate::node::parse(
        r#"
        int a = 0;
        int main(void) {
            a = 2;
            return a;
        }
           "#,
    )
    .unwrap();

    let scope = Rc::new(Scope::default());
    let scope_tree = gen_scope_toplevel(&mut nodes, scope, Weak::new(), true).unwrap();

    let ir = gen_ir(nodes, scope_tree);
    println!("{:#?}", ir);
}
