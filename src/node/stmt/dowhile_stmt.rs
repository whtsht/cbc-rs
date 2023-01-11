use super::*;

pub fn parse_dowhile_stmt(pair: Pair<Rule>) -> Result<StmtNode, NodeError> {
    let mut pairs = pair.into_inner();
    pairs.next().unwrap(); // do
    let stmt = parse_stmt_node(pairs.next().unwrap())?;
    pairs.next().unwrap(); // while
    let cnd = parse_expr_node(pairs.next().unwrap())?;
    pairs.next().unwrap(); // semicolon
    Ok(StmtNode::DoWhile { cond: cnd, stmt })
}

#[test]
fn test_dowhile() {
    assert!(parse_stmt_node(
        CBCScanner::parse(
            Rule::STMT,
            r#"do {
                     foo -= 1;
                } while (foo > 0);
                "#
        )
        .unwrap()
        .next()
        .unwrap()
    )
    .is_ok());
}
