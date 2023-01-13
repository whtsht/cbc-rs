use super::*;

pub fn parse_while_stmt(pair: Pair<Rule>) -> Result<StmtNode, NodeError> {
    let mut pairs = pair.into_inner();
    pairs.next().unwrap(); // while
    let cond = parse_expr_node(pairs.next().unwrap())?;
    let stmt = Box::new(parse_stmt_node(pairs.next().unwrap())?);
    Ok(StmtNode::While { cond, stmt })
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
