use super::DefNode;
use crate::node::type_::parse_type_node;
use crate::node::NodeError;
use crate::Rule;
use pest::iterators::Pair;

pub fn def_type(pair: Pair<Rule>) -> Result<DefNode, NodeError> {
    let mut pairs = pair.into_inner();

    pairs.next().unwrap(); // typedef
    let _type = parse_type_node(pairs.next().unwrap())?;
    let ident = pairs.next().unwrap().as_str().into();

    Ok(DefNode::Type { _type, ident })
}

#[test]
fn test_type() {
    use crate::node::def::parse_topdef_node;
    use crate::CBCScanner;
    use pest::Parser;
    assert!(parse_topdef_node(
        CBCScanner::parse(Rule::TOP_DEF, "typedef int Point;",)
            .unwrap()
            .next()
            .unwrap()
    )
    .is_ok());
}
