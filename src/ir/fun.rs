use crate::ir::DefinedFun;
use crate::ir::GenError;
use crate::node::def::def_fun::DefFun;

use super::unit::transform_stmt;
use super::IRInfo;

pub fn gen_def_fun(fun: &DefFun, info: &mut IRInfo) -> Result<DefinedFun, GenError> {
    let mut stmts = vec![];

    info.push_scope(fun.scope.as_ref().unwrap().clone());

    for stmt in fun.block.iter() {
        stmts.extend(transform_stmt(stmt, info)?);
    }

    Ok(DefinedFun {
        name: fun.name.clone(),
        _type: (fun._type.clone(), fun.params.clone()),
        is_private: fun.is_static,
        body: stmts,
    })
}
