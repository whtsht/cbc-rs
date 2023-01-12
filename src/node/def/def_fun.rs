use super::DefNode;
use crate::node::param::parse_params_node;
use crate::node::stmt::block::parse_block;
use crate::node::type_::parse_type_node;
use crate::node::{Node, NodeError};
use crate::Rule;
use pest::iterators::Pair;

pub fn def_fun(pair: Pair<Rule>) -> Result<DefNode, NodeError> {
    let mut pairs = pair.into_inner();

    let is_static = pairs.next().unwrap().into_inner().count() > 0;
    let _type = parse_type_node(pairs.next().unwrap())?;
    let name = pairs.next().unwrap().as_str().into();
    let params = parse_params_node(pairs.next().unwrap())?;
    let block = Node::Stmt(Box::new(parse_block(pairs.next().unwrap())?));

    Ok(DefNode::Fun {
        is_static,
        _type,
        name,
        params,
        block,
    })
}

#[test]
fn test_fun() {
    use crate::node::def::parse_def_node;
    use crate::CBCScanner;
    use pest::Parser;
    assert!(parse_def_node(
        CBCScanner::parse(Rule::TOP_DEF, "int main(void) { return 0; }",)
            .unwrap()
            .next()
            .unwrap()
    )
    .is_ok());
}
