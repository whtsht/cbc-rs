use super::{param::parse_params_node, *};
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
    suffixs: Vec<TypeSuffix>,
}

#[derive(Debug)]
pub enum TypeSuffix {
    Array,
    ArrayWithValue(i32),
    Pointer,
    Params(Node),
}

pub fn parse_type_node(pair: Pair<Rule>) -> Result<Node, NodeError> {
    let mut pairs = pair.into_inner();
    let base = parse_typebase_node(pairs.next().unwrap())?;

    let mut suffixs = vec![];
    while let Some(pair) = pairs.next() {
        match pair.as_rule() {
            Rule::LSB => {
                let next = pairs.next().unwrap();
                if next.as_rule() == Rule::INTEGER {
                    suffixs.push(TypeSuffix::ArrayWithValue(next.as_str().parse().unwrap()));
                    pairs.next(); // Skip the right bracket
                } else {
                    suffixs.push(TypeSuffix::Array);
                }
            }
            Rule::STAR => suffixs.push(TypeSuffix::Pointer),
            Rule::LPT => {
                let params = parse_params_node(pairs.next().unwrap())?;
                pairs.next(); // Skip the right bracket
                suffixs.push(TypeSuffix::Params(params));
            }
            _ => todo!(),
        }
    }

    Ok(Node::Type(Box::new(TypeNode { base, suffixs })))
}

pub fn parse_typebase_node(pair: Pair<Rule>) -> Result<TypeBaseNode, NodeError> {
    let mut pairs = pair.into_inner();
    let first = pairs.next().unwrap();
    let second = pairs.peek();
    match (first.as_rule(), second.map(|x| x.as_rule())) {
        (Rule::VOID, None) => Ok(TypeBaseNode::Void),
        (Rule::CHAR, None) => Ok(TypeBaseNode::Char),
        (Rule::SHORT, None) => Ok(TypeBaseNode::Short),
        (Rule::INT, None) => Ok(TypeBaseNode::Int),
        (Rule::LONG, None) => Ok(TypeBaseNode::Long),
        (Rule::UNSIGNED, Some(Rule::CHAR)) => {
            pairs.next();
            Ok(TypeBaseNode::UnsignedChar)
        }
        (Rule::UNSIGNED, Some(Rule::SHORT)) => {
            pairs.next();
            Ok(TypeBaseNode::UnsignedShort)
        }
        (Rule::UNSIGNED, Some(Rule::INT)) => {
            pairs.next();
            Ok(TypeBaseNode::UnsignedInt)
        }
        (Rule::UNSIGNED, Some(Rule::LONG)) => {
            pairs.next();
            Ok(TypeBaseNode::UnsignedLong)
        }
        (Rule::STRUCT, Some(Rule::IDENTIFIER)) => {
            let ident = pairs.next().unwrap().into_inner().next().unwrap().as_str();
            Ok(TypeBaseNode::Struct(ident.into()))
        }
        err => Err(NodeError {
            _type: NodeErrorType::TypeBase,
            message: format!("typebase error: {:?}", err),
        }),
    }
}

#[test]
fn test_type() {
    assert!(parse_type_node(
        CBCScanner::parse(Rule::TYPEREF, "int (int a, long b, ...)*[][]*")
            .unwrap()
            .next()
            .unwrap()
    )
    .is_ok());
}
