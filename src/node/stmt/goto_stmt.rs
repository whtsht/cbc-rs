use super::*;

pub fn parse_goto_stmt(pair: Pair<Rule>) -> Result<StmtNode, NodeError> {
    let mut pairs = pair.into_inner();
    pairs.next().unwrap(); // goto
    let label = pairs.next().unwrap().as_str().into();
    pairs.next().unwrap(); // semicolon
    Ok(StmtNode::Goto { label })
}

#[test]
fn test_goto() {
    assert!(parse_stmt_node(
        CBCScanner::parse(
            Rule::STMT,
            r#"while (true) {
                goto outer;
            }"#
        )
        .unwrap()
        .next()
        .unwrap()
    )
    .is_ok());
}
