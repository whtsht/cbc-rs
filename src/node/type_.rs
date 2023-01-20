use super::{
    param::{parse_params_node, ParamsNode},
    *,
};
use crate::resolve::variable_scope::Entity;
use crate::Rule;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
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
    Struct(String, Option<Box<Entity>>),
    Union(String, Option<Box<Entity>>),
    Identifier(String, Option<Box<Entity>>),
}

#[derive(Debug, Clone)]
pub struct TypeNode {
    pub base: TypeBaseNode,
    pub suffixs: Vec<TypeSuffix>,
}

#[derive(Debug, Clone)]
pub enum TypeSuffix {
    Array,
    ArrayWithValue(i32),
    Pointer,
    Params(ParamsNode),
}

pub fn parse_type_node(pair: Pair<Rule>) -> Result<TypeNode, NodeError> {
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

    Ok(TypeNode { base, suffixs })
}

pub fn parse_typebase_node(pair: Pair<Rule>) -> Result<TypeBaseNode, NodeError> {
    println!("{:?}", pair);
    let mut pairs = pair.into_inner().peekable();
    match pairs.peek().unwrap().as_rule() {
        Rule::VOID => Ok(TypeBaseNode::Void),
        Rule::CHAR => Ok(TypeBaseNode::Char),
        Rule::SHORT => Ok(TypeBaseNode::Short),
        Rule::INT => Ok(TypeBaseNode::Int),
        Rule::LONG => Ok(TypeBaseNode::Long),
        Rule::UNSIGNED_CHAR => Ok(TypeBaseNode::UnsignedChar),
        Rule::UNSIGNED_SHORT => Ok(TypeBaseNode::UnsignedShort),
        Rule::UNSIGNED_INT => Ok(TypeBaseNode::UnsignedInt),
        Rule::UNSIGNED_LONG => Ok(TypeBaseNode::UnsignedLong),
        Rule::STRUCT_IDENT => {
            let ident = pairs
                .next()
                .unwrap()
                .into_inner()
                .nth(1)
                .unwrap()
                .as_str()
                .into();

            Ok(TypeBaseNode::Struct(ident, None))
        }
        Rule::UNION_IDENT => {
            let ident = pairs
                .next()
                .unwrap()
                .into_inner()
                .nth(1)
                .unwrap()
                .as_str()
                .into();

            Ok(TypeBaseNode::Union(ident, None))
        }
        Rule::IDENTIFIER => Ok(TypeBaseNode::Identifier(
            pairs.next().unwrap().as_str().into(),
            None,
        )),
        e => panic!("{:?}", e),
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
