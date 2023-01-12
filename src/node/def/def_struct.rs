use super::{parse_member_list, DefNode};
use crate::node::NodeError;
use crate::Rule;
use pest::iterators::Pair;

pub fn def_struct(pair: Pair<Rule>) -> Result<DefNode, NodeError> {
    let mut pairs = pair.into_inner();

    pairs.next().unwrap(); // struct
    let name = pairs.next().unwrap().as_str().into();
    let member_list = parse_member_list(pairs.next().unwrap())?;

    Ok(DefNode::Struct { name, member_list })
}

#[test]
fn test_struct() {
    use crate::node::def::parse_def_node;
    use crate::CBCScanner;
    use pest::Parser;
    assert!(parse_def_node(
        CBCScanner::parse(Rule::TOP_DEF, "struct Point { int x; int y; }",)
            .unwrap()
            .next()
            .unwrap()
    )
    .is_ok());
}
