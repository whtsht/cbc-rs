use std::collections::HashMap;

use crate::node::type_::{TypeBaseNode, TypeSuffix};

use super::variable_scope::{Entity, Scope};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum TypeDep {
    NewType(String),
    Algebraic(String),
}

fn dfs(map: &HashMap<TypeDep, Vec<TypeDep>>, v: TypeDep) -> (bool, Vec<TypeDep>) {
    let mut seen: HashMap<TypeDep, bool> =
        HashMap::from_iter(map.clone().into_keys().map(|k| (k, false)));

    let mut todo = Vec::new();

    let mut hist = vec![];
    *seen.get_mut(&v).unwrap() = true;
    todo.push(&v);
    hist.push(v.clone());

    while let Some(n) = todo.pop() {
        *seen.get_mut(&n).unwrap() = true;

        for next in map[&n].iter() {
            if seen[&next] {
                return (true, hist);
            }

            *seen.get_mut(&next).unwrap() = true;
            todo.push(&next);
            hist.push(next.clone());
        }
    }

    (false, hist)
}

pub fn check_recursive_definition(map: &HashMap<TypeDep, Vec<TypeDep>>) -> Option<Vec<TypeDep>> {
    let mut finished: HashMap<TypeDep, bool> =
        HashMap::from_iter(map.clone().into_keys().map(|k| (k, false)));

    for k in map.clone().keys() {
        if !finished[&k] {
            let (rec, hist) = dfs(&map, k.clone());
            if rec {
                return Some(hist);
            }
            for h in hist {
                *finished.get_mut(&h).unwrap() = true;
            }
        }
    }
    None
}

pub fn scope_to_typedep(toplevel_scope: &Scope) -> HashMap<TypeDep, Vec<TypeDep>> {
    let entities = toplevel_scope.entities.borrow();
    let mut map = HashMap::new();

    for (name, entity) in entities.iter() {
        match entity {
            Entity::Struct { member_list } | Entity::Union { member_list } => {
                let mut vec = vec![];

                for member in member_list {
                    let mut is_pointer = false;
                    for suf in member._type.suffixs.iter() {
                        match suf {
                            TypeSuffix::Pointer => is_pointer = true,
                            _ => {}
                        }
                    }
                    if is_pointer {
                        continue;
                    }
                    match &member._type.base {
                        TypeBaseNode::Struct(mname, _) | TypeBaseNode::Union(mname, _) => {
                            vec.push(TypeDep::Algebraic(mname.clone()))
                        }
                        TypeBaseNode::Identifier(mname, _) => {
                            vec.push(TypeDep::NewType(mname.clone()))
                        }
                        _ => {}
                    }
                }

                map.insert(TypeDep::Algebraic(name.clone()), vec);
            }
            Entity::TypeDef { _type } => {
                let deptype = match &_type.base {
                    TypeBaseNode::Struct(mname, _) | TypeBaseNode::Union(mname, _) => {
                        TypeDep::Algebraic(mname.clone())
                    }
                    TypeBaseNode::Identifier(mname, _) => TypeDep::NewType(mname.clone()),
                    _ => unreachable!(),
                };
                let vec = vec![deptype];

                map.insert(TypeDep::NewType(name.clone()), vec);
            }
            _ => {}
        }
    }

    map
}
#[test]
fn test_recursive() {
    #![allow(dead_code)]
    use super::variable_scope::gen_scope_toplevel;
    use std::rc::{Rc, Weak};
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
    let toplevel_scope =
        gen_scope_toplevel(&mut nodes, Rc::new(Scope::default()), Weak::new(), false).unwrap();

    let map = scope_to_typedep(toplevel_scope.as_ref());
    assert_eq!(check_recursive_definition(&map), None);

    let mut nodes = crate::node::parse(
        r#"
        struct point_x {
            struct point_y y;
            struct point_z z;
        }
        typedef struct point_x my_point_x;

        struct point_y {
            my_point_x x;
        }

        struct point_z {
            int x;
            int y;
        }
        "#,
    )
    .unwrap();
    println!("{:?}", nodes);

    let toplevel_scope =
        gen_scope_toplevel(&mut nodes, Rc::new(Scope::default()), Weak::new(), false).unwrap();

    let map = scope_to_typedep(toplevel_scope.as_ref());
    assert!(check_recursive_definition(&map).is_some());
}
