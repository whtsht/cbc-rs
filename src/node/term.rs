use super::*;
use crate::Rule;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub enum TermNode {
    Cast(TypeNode, Box<TermNode>),
    Unary(Box<UnaryNode>),
}

pub fn parse_term_node(pair: Pair<Rule>) -> Result<TermNode, NodeError> {
    let mut pairs = pair.into_inner().peekable();
    let node = match pairs.peek().unwrap().as_rule() {
        Rule::LPT => {
            pairs.next().unwrap(); // Skip the left bracket
            let type_node = pairs.next().unwrap();
            pairs.next().unwrap(); // Skip the right bracket
            let node = TermNode::Cast(
                parse_type_node(type_node)?,
                Box::new(parse_term_node(pairs.next().unwrap())?),
            );
            node
        }
        Rule::UNARY => TermNode::Unary(Box::new(parse_unary_node(pairs.next().unwrap())?)),
        err => panic!("term error: {:?}", err),
    };

    Ok(node)
}
