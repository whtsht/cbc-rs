use super::*;

pub fn parse_if_stmt(pair: Pair<Rule>) -> Result<StmtNode, NodeError> {
    let mut pairs = pair.into_inner();
    pairs.next().unwrap(); // if
    let expr = parse_expr_node(pairs.next().unwrap())?;
    let then = parse_stmt_node(pairs.next().unwrap())?;
    pairs.next().unwrap(); // else
    let _else = parse_stmt_node(pairs.next().unwrap())?;
    Ok(StmtNode::If { expr, then, _else })
}

#[test]
fn test_if() {
    println!(
        "{:#?}",
        parse_stmt_node(
            CBCScanner::parse(
                Rule::STMT,
                r#"if (a == b) {
                       a += 1;
                   } else {
                       a = b;
                   }"#
            )
            .unwrap()
            .next()
            .unwrap()
        )
        .unwrap()
    );
}
