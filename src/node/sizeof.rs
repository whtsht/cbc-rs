use super::*;
use crate::Rule;
use pest::iterators::Pairs;
use std::iter::Peekable;

pub fn parse_sizeof_node(mut pairs: Peekable<Pairs<Rule>>) -> Result<UnaryNode, NodeError> {
    let node = match pairs.peek().unwrap().as_rule() {
        Rule::LPT => {
            let node = parse_type_node(pairs.nth(1).unwrap())?;
            pairs.next().unwrap(); // )
            UnaryNode::SizeofType(node)
        }
        Rule::UNARY => UnaryNode::SizeofUnary(Box::new(parse_unary_node(pairs.next().unwrap())?)),
        err => panic!("sizeof error: {:?}", err),
    };

    Ok(node)
}
