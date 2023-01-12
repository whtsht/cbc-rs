use super::DefNode;
use crate::node::expr::parse_expr_node;
use crate::node::type_::parse_type_node;
use crate::node::NodeError;
use crate::Rule;
use pest::iterators::Pair;

pub fn def_const(pair: Pair<Rule>) -> Result<DefNode, NodeError> {
    let mut pairs = pair.into_inner();

    pairs.next().unwrap(); // const
    let _type = parse_type_node(pairs.next().unwrap())?;
    let name = pairs.next().unwrap().as_str().into();
    pairs.next().unwrap(); // =
    let expr = parse_expr_node(pairs.next().unwrap())?;

    Ok(DefNode::Const { _type, name, expr })
}

#[test]
fn test_const() {
    use crate::node::def::parse_def_node;
    use crate::CBCScanner;
    use pest::Parser;
    assert!(parse_def_node(
        CBCScanner::parse(Rule::TOP_DEF, "const int PI = 3;",)
            .unwrap()
            .next()
            .unwrap()
    )
    .is_ok());
}
