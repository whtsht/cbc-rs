use super::*;
use crate::Rule;
use pest::iterators::Pairs;
use std::iter::Peekable;

#[derive(Debug)]
pub enum TermNode {
    Cast(Node, Box<Node>),
    Unary(Node),
}

pub fn parse_term_node(mut pairs: Peekable<Pairs<Rule>>) -> Result<Node, NodeError> {
    let mut pairs = pairs.next().unwrap().into_inner().peekable();
    let node = match pairs.peek().unwrap().as_rule() {
        Rule::LBRACKET => {
            pairs.next().unwrap(); // Skip the left bracket
            let type_node = pairs.next().unwrap();
            pairs.next().unwrap(); // Skip the right bracket
            let node = Node::Term(Box::new(TermNode::Cast(
                parse_type_node(type_node)?,
                Box::new(parse_term_node(pairs)?),
            )));
            node
        }
        Rule::UNARY => Node::SizeofExprNode(Box::new(SizeofExprNode {
            expr: parse_unary_node(pairs.next().unwrap())?,
        })),
        err => panic!("term error: {:?}", err),
    };

    Ok(node)
}
