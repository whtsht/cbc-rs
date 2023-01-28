use pest::iterators::Pair;

use super::*;

#[derive(Debug, Clone)]
pub enum ParamsNode {
    Void,
    Some { fixed: Vec<Param>, variable: bool },
}

#[derive(Debug, Clone)]
pub struct Param {
    pub _type: TypeNode,
    pub name: String,
}

pub fn parse_param(pair: Pair<Rule>) -> Result<Param, NodeError> {
    debug_assert_eq!(pair.as_rule(), Rule::PARAM);

    let mut pairs = pair.into_inner();
    let _type = parse_type_node(pairs.next().unwrap())?;
    let name = pairs.next().unwrap().as_str().into();

    Ok(Param { _type, name })
}

pub fn parse_params_node(pair: Pair<Rule>) -> Result<ParamsNode, NodeError> {
    debug_assert_eq!(pair.as_rule(), Rule::PARAMS);
    let mut pairs = pair.into_inner().peekable();

    if let Some(Rule::VOID) = pairs.peek().map(|p| p.as_rule()) {
        return Ok(ParamsNode::Void);
    }

    let mut pairs = pairs.next().unwrap().into_inner();
    let mut fixed = vec![];

    while let Some(pair) = pairs.next() {
        match pair.as_rule() {
            Rule::PARAM => fixed.push(parse_param(pair)?),
            Rule::VAR_PARAMS => {
                return Ok(ParamsNode::Some {
                    fixed,
                    variable: true,
                })
            }
            _ => todo!(),
        }
    }

    Ok(ParamsNode::Some {
        fixed,
        variable: false,
    })
}

#[test]
fn test_param() {
    assert!(parse_param(
        CBCScanner::parse(Rule::PARAM, "int a")
            .unwrap()
            .next()
            .unwrap(),
    )
    .is_ok());
}

#[test]
fn test_params() {
    assert!(parse_params_node(
        CBCScanner::parse(Rule::PARAMS, "void")
            .unwrap()
            .next()
            .unwrap()
    )
    .is_ok());
    assert!(parse_params_node(
        CBCScanner::parse(Rule::PARAMS, "int foo, long bar, char _foo_bar")
            .unwrap()
            .next()
            .unwrap()
    )
    .is_ok());
}
