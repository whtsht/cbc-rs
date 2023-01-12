use super::*;

pub fn parse_block(pair: Pair<Rule>) -> Result<StmtNode, NodeError> {
    let mut pairs = pair.into_inner();
    let stmts = parse_stmts(pairs.next().unwrap())?;
    Ok(StmtNode::Block { stmts })
}

#[test]
fn test_block() {
    assert!(parse_block(
        CBCScanner::parse(
            Rule::BLOCK,
            "{
                t = b;
                b = a;
                a = t;
            }"
        )
        .unwrap()
        .next()
        .unwrap(),
    )
    .is_ok());
}
