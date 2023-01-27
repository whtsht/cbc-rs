use super::DefNode;
use crate::node::param::parse_params_node;
use crate::node::param::ParamsNode;
use crate::node::stmt::block::parse_block_stmts;
use crate::node::stmt::StmtNode;
use crate::node::type_::parse_type_node;
use crate::node::NodeError;
use crate::node::TypeNode;
use crate::resolve::variable_scope::Scope;
use crate::Rule;
use pest::iterators::Pair;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct DefFun {
    pub is_static: bool,
    pub _type: TypeNode,
    pub name: String,
    pub params: ParamsNode,
    pub block: Vec<StmtNode>,
    pub scope: Option<Rc<Scope>>,
}

pub fn def_fun(pair: Pair<Rule>) -> Result<DefNode, NodeError> {
    let mut pairs = pair.into_inner();

    let is_static = pairs.next().unwrap().into_inner().count() > 0;
    let _type = parse_type_node(pairs.next().unwrap())?;
    let name = pairs.next().unwrap().as_str().into();
    let params = parse_params_node(pairs.next().unwrap())?;
    let block = parse_block_stmts(pairs.next().unwrap())?;

    Ok(DefNode::Fun(DefFun {
        is_static,
        _type,
        name,
        params,
        block,
        scope: None,
    }))
}

#[test]
fn test_fun() {
    use crate::node::def::parse_topdef_node;
    use crate::CBCScanner;
    use pest::Parser;
    assert!(parse_topdef_node(
        CBCScanner::parse(Rule::TOP_DEF, "int main(void) { return 0; }",)
            .unwrap()
            .next()
            .unwrap()
    )
    .is_ok());
}
