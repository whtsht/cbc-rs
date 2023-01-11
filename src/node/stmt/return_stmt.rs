use super::*;

pub fn parse_return_stmt(pair: Pair<Rule>) -> Result<StmtNode, NodeError> {
    let mut pairs = pair.into_inner();
    pairs.next().unwrap(); // return
    let pair = pairs.next().unwrap();
    let node = match pair.as_rule() {
        Rule::SCOLON => StmtNode::Return { expr: None },
        Rule::EXPR => StmtNode::Return {
            expr: Some(parse_expr_node(pair)?),
        },
        _ => unreachable!(),
    };
    Ok(node)
}

#[test]
fn test_return() {
    assert!(parse_stmt_node(
        CBCScanner::parse(
            Rule::STMT,
            r#"while (a > 0) {
                return 0;
            }"#
        )
        .unwrap()
        .next()
        .unwrap()
    )
    .is_ok());
}
