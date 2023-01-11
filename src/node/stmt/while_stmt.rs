use super::*;

pub fn parse_while_stmt(pair: Pair<Rule>) -> Result<StmtNode, NodeError> {
    let mut pairs = pair.into_inner();
    pairs.next().unwrap(); // while
    let cnd = parse_expr_node(pairs.next().unwrap())?;
    let stmt = parse_stmt_node(pairs.next().unwrap())?;
    Ok(StmtNode::While { cond: cnd, stmt })
}

#[test]
fn test_while() {
    assert!(parse_stmt_node(
        CBCScanner::parse(
            Rule::STMT,
            r#"while (foo > 0) {
                     foo += 1;
                }
                "#
        )
        .unwrap()
        .next()
        .unwrap()
    )
    .is_ok());
}
