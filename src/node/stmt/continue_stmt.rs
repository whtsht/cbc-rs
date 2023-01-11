use super::*;

pub fn parse_continue_stmt(pair: Pair<Rule>) -> Result<StmtNode, NodeError> {
    let mut pairs = pair.into_inner();
    pairs.next().unwrap(); // continue
    pairs.next().unwrap(); // semicolon
    Ok(StmtNode::Continue)
}

#[test]
fn test_continue() {
    assert!(parse_stmt_node(
        CBCScanner::parse(
            Rule::STMT,
            r#"while (a > 0) {
                continue;
            }"#
        )
        .unwrap()
        .next()
        .unwrap()
    )
    .is_ok());
}
