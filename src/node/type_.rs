use super::param::ParamsNode;
use super::*;
use crate::Rule;
use pest::iterators::Pair;

#[derive(Debug)]
pub enum TypeBaseNode {
    Void,
    Char,
    Short,
    Int,
    Long,
    UnsignedChar,
    UnsignedShort,
    UnsignedInt,
    UnsignedLong,
    Struct(String),
    Union(String),
    Identifier(String),
}

#[derive(Debug)]
pub struct TypeNode {
    base: TypeBaseNode,
    suffix: TypeSuffix,
}

#[derive(Debug)]
pub enum TypeSuffix {
    Array,
    ArrayWithValue(i32),
    Pointer,
    Params(ParamsNode),
}

pub fn parse_type_node(pair: Pair<Rule>) -> Result<Node, NodeError> {
    let pair = pair.into_inner().next().unwrap();
    parse_typebase_node(pair)
}

pub fn parse_typebase_node(pair: Pair<Rule>) -> Result<Node, NodeError> {
    let mut pairs = pair.into_inner();
    let first = pairs.next().unwrap();
    let second = pairs.peek();
    match (first.as_rule(), second.map(|x| x.as_rule())) {
        (Rule::VOID, None) => Ok(Node::TypeBase(Box::new(TypeBaseNode::Void))),
        (Rule::CHAR, None) => Ok(Node::TypeBase(Box::new(TypeBaseNode::Char))),
        (Rule::SHORT, None) => Ok(Node::TypeBase(Box::new(TypeBaseNode::Short))),
        (Rule::INT, None) => Ok(Node::TypeBase(Box::new(TypeBaseNode::Int))),
        (Rule::LONG, None) => Ok(Node::TypeBase(Box::new(TypeBaseNode::Long))),
        (Rule::UNSIGNED, Some(Rule::CHAR)) => {
            pairs.next();
            Ok(Node::TypeBase(Box::new(TypeBaseNode::UnsignedChar)))
        }
        (Rule::UNSIGNED, Some(Rule::SHORT)) => {
            pairs.next();
            Ok(Node::TypeBase(Box::new(TypeBaseNode::UnsignedShort)))
        }
        (Rule::UNSIGNED, Some(Rule::INT)) => {
            pairs.next();
            Ok(Node::TypeBase(Box::new(TypeBaseNode::UnsignedInt)))
        }
        (Rule::UNSIGNED, Some(Rule::LONG)) => {
            pairs.next();
            Ok(Node::TypeBase(Box::new(TypeBaseNode::UnsignedLong)))
        }
        (Rule::STRUCT, Some(Rule::IDENTIFIER)) => {
            let ident = pairs.next().unwrap().into_inner().next().unwrap().as_str();
            Ok(Node::TypeBase(Box::new(TypeBaseNode::Struct(ident.into()))))
        }
        err => Err(NodeError {
            _type: NodeErrorType::TypeBase,
            message: format!("typebase error: {:?}", err),
        }),
    }
}
