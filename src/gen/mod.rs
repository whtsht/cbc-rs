use std::io;
use xten::asm::*;

use crate::ir::{Const, DefinedFun, Expr, Stmt, IR};

pub fn main_object() -> io::Result<Object> {
    let mut w = Writer::new();
    let main = w.get_label("main");

    w.define(main, true);
    w.pushq(Rbp)?;
    w.movq(Rbp, Rsp)?;
    w.movl(Eax, 42)?;

    w.popq(Rbp)?;
    w.retq()?;

    w.produce()
}

pub fn compile(ir: IR) -> Result<Vec<Object>, io::Error> {
    let mut objects = vec![];
    for fun in ir.fun {
        objects.push(compile_function(fun)?);
    }
    Ok(objects)
}

pub fn compile_function(fun: DefinedFun) -> io::Result<Object> {
    let mut w = Writer::new();
    let main = w.get_label(&fun.name);

    w.define(main, true);

    for stmt in fun.body {
        match stmt {
            Stmt::ExprStmt(expr) | Stmt::Return(Some(expr)) => match expr {
                Expr::Const(con) => match con {
                    Const::Int(i) => {
                        w.movl(Eax, i)?;
                    }
                    _ => todo!(),
                },
                _ => todo!(),
            },
            _ => todo!(),
        }
    }
    w.retq()?;
    w.produce()
}

pub fn compile_from_source(source: &str) -> Result<Vec<Object>, io::Error> {
    use super::ir::gen_ir;
    use crate::resolve::variable_scope::{gen_scope_toplevel, Scope};
    use std::rc::{Rc, Weak};

    let mut nodes = crate::node::parse(source).unwrap();

    let scope =
        gen_scope_toplevel(&mut nodes, Rc::new(Scope::default()), Weak::new(), false).unwrap();

    gen_scope_toplevel(&mut nodes, scope, Weak::new(), true).unwrap();

    let ir = gen_ir(nodes);
    assert!(ir.is_ok());
    println!("{:#?}", ir);

    let objects = compile(ir.unwrap()).unwrap();

    Ok(objects)
}

#[test]
pub fn test_simple() {
    use xten::jit;
    use xten::jit::symbol_resolver;

    let mut engine = jit::Engine::new(symbol_resolver::none);
    let objects = compile_from_source(
        r#"
        int main(void) {
            return 20;
        }
           "#,
    )
    .unwrap();

    for obj in objects {
        engine.add_object(&obj).unwrap();
    }

    let main = engine.get("main").expect("main not defined");
    let main = unsafe { std::mem::transmute::<_, extern "C" fn() -> i32>(main) };
    println!("{}", main());
}

// #[test]
// fn test_fib() {
//     use xten::jit;
//     use xten::jit::symbol_resolver;

//     let mut engine = jit::Engine::new(symbol_resolver::none);
//     let objects = compile_from_source(
//         r#"
//         int fib(int n) {
//             if (n < 2) {
//                 return n;
//             } else {
//                 return fib(n - 1) + fib(n - 2);
//             }
//         }

//         int main(void) {
//             return fib(10);
//         }
//            "#,
//     )
//     .unwrap();

//     for obj in objects {
//         engine.add_object(&obj).unwrap();
//     }

//     let main = engine.get("main").expect("main not defined");
//     let main = unsafe { std::mem::transmute::<_, extern "C" fn() -> i32>(main) };
//     println!("{}", main());
// }
