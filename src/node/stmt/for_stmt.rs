use super::*;

pub fn parse_for_stmt(pair: Pair<Rule>) -> Result<StmtNode, NodeError> {
    let mut pairs = pair.into_inner();
    pairs.next().unwrap(); // for
    let init = parse_expr_node(pairs.next().unwrap())?;
    pairs.next().unwrap(); // semicolon
    let cond = parse_expr_node(pairs.next().unwrap())?;
    pairs.next().unwrap(); // semicolon
    let term = parse_expr_node(pairs.next().unwrap())?;
    let stmt = parse_stmt_node(pairs.next().unwrap())?;
    Ok(StmtNode::For {
        init,
        cond,
        term,
        stmt,
    })
}

#[test]
fn test_for() {
    assert!(parse_stmt_node(
        CBCScanner::parse(
            Rule::STMT,
            r#"for (i = 0; i < 0; i++) {
                    sum += i;
                }
                "#
        )
        .unwrap()
        .next()
        .unwrap()
    )
    .is_ok());
}
