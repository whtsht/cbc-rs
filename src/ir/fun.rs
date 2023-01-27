use crate::ir::DefinedFun;
use crate::ir::GenError;
use crate::node::def::def_fun::DefFun;
use crate::resolve::variable_scope::Scope;

use super::unit::transform_stmt;
use super::IRInfo;

pub fn gen_def_fun(fun: &DefFun, scope: &Scope, info: &mut IRInfo) -> Result<DefinedFun, GenError> {
    let mut stmts = vec![];

    for stmt in fun.block.iter() {
        stmts.extend(transform_stmt(stmt, info, scope)?);
    }

    Ok(DefinedFun {
        name: fun.name.clone(),
        _type: (fun._type.clone(), fun.params.clone()),
        is_private: fun.is_static,
        body: stmts,
    })
}
