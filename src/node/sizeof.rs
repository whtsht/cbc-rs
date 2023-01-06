use super::*;
use crate::Rule;
use pest::iterators::Pairs;
use std::iter::Peekable;

#[derive(Debug)]
pub struct SizeofExprNode {
    pub expr: Node,
}

#[derive(Debug)]
pub struct SizeofTypeNode {
    pub _type: Node,
}

pub fn parse_sizeof_node(mut pairs: Peekable<Pairs<Rule>>) -> Result<Node, NodeError> {
    let node = match pairs.peek().unwrap().as_rule() {
        Rule::LPT => {
            let node = Node::Unary(Box::new(UnaryNode::SizeofType(parse_type_node(
                pairs.nth(1).unwrap(),
            )?)));
            pairs.next().unwrap(); // Skip the right bracket
            node
        }
        Rule::UNARY => Node::Unary(Box::new(UnaryNode::SizeofExpr(parse_unary_node(
            pairs.next().unwrap(),
        )?))),
        err => panic!("sizeof error: {:?}", err),
    };

    Ok(node)
}
