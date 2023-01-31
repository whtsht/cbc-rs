use std::io;
use xten::asm::*;
use xten::jit;
use xten::jit::symbol_resolver;

fn fib_object() -> io::Result<Object> {
    let mut w = Writer::new();
    let fib = w.get_label("fib");
    let l1 = w.issue_label();
    let l2 = w.issue_label();

    w.define(fib, true);
    w.cmpl(Edi, 1i8)?;
    w.jle(Short(l2))?;
    w.movl(Edx, 1)?;
    w.movl(Eax, 1)?;
    w.xorl(Ecx, Ecx)?;

    w.define(l1, false);
    w.movl(Esi, Eax)?;
    w.addl(Edx, 1i8)?;
    w.addl(Eax, Ecx)?;
    w.movl(Ecx, Esi)?;
    w.cmpl(Edi, Edx)?;
    w.jne(Short(l1))?;
    w.retq()?;

    w.define(l2, false);
    w.movl(Eax, 1)?;

    w.retq()?;

    w.produce()
}

fn main() {
    let obj = fib_object().unwrap();

    let mut engine = jit::Engine::new(symbol_resolver::none);
    assert!(engine.add_object(&obj).is_ok());

    let fib = engine.get("fib").expect("fib not defined");
    let fib = unsafe { std::mem::transmute::<_, extern "C" fn(i32) -> i32>(fib) };
    assert_eq!(fib(10), 55);
    println!("{}", fib(20));
    let cos = engine.get("cos").expect("cos not defined");
    let cos = unsafe { std::mem::transmute::<_, extern "C" fn(i32) -> i32>(cos) };
    println!("{}", cos(1));
}
