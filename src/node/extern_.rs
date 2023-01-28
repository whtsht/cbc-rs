use super::{param::ParamsNode, type_::TypeNode, Node, NodeError};
use crate::node::param::parse_params_node;
use crate::node::parse_type_node;
use crate::Rule;
use pest::iterators::Pairs;

#[derive(Debug, Clone)]
pub struct PrototypeFun {
    pub name: String,
    pub params: ParamsNode,
    pub return_type: TypeNode,
}

pub fn parse_prototypefun(pairs: &mut Pairs<Rule>) -> Result<Node, NodeError> {
    let mut pairs = pairs.nth(1).unwrap().into_inner();
    pairs.next();
    let return_type = parse_type_node(pairs.next().unwrap())?;
    let name = pairs.next().unwrap().as_str().into();
    let params = parse_params_node(pairs.next().unwrap())?;

    Ok(Node::Extern(Box::new(PrototypeFun {
        return_type,
        name,
        params,
    })))
}
