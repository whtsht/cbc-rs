use super::*;

pub fn parse_break_stmt(pair: Pair<Rule>) -> Result<StmtNode, NodeError> {
    let mut pairs = pair.into_inner();
    pairs.next().unwrap(); // break
    pairs.next().unwrap(); // semicolon
    Ok(StmtNode::Break)
}

#[test]
fn test_break() {
    assert!(parse_stmt_node(
        CBCScanner::parse(
            Rule::STMT,
            r#"while (a > 0) {
                break;
            }"#
        )
        .unwrap()
        .next()
        .unwrap()
    )
    .is_ok());
}
