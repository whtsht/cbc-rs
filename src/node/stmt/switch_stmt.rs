use super::*;

pub fn parse_switch_stmt(pair: Pair<Rule>) -> Result<StmtNode, NodeError> {
    let mut pairs = pair.into_inner();
    pairs.next().unwrap(); // swtich
    let cond = parse_expr_node(pairs.next().unwrap())?;
    let (cases, default) = case_clauses(pairs.next().unwrap())?;

    Ok(StmtNode::Switch {
        cond,
        cases,
        default,
    })
}

pub fn case_clauses(
    pair: Pair<Rule>,
) -> Result<
    (
        Vec<(Vec<PrimaryNode>, Vec<StmtNode>)>,
        Option<Vec<StmtNode>>,
    ),
    NodeError,
> {
    let mut pairs = pair.into_inner().peekable();

    let mut clist = vec![];

    while pairs.peek().map(|x| x.as_rule()) == Some(Rule::CASE_CLAUSE) {
        clist.push(case_clause(pairs.next().unwrap())?);
    }

    if pairs.peek().map(|x| x.as_rule()) == Some(Rule::DEFAULT_CLAUSE) {
        let dcase = default_clause(pairs.next().unwrap())?;
        Ok((clist, Some(dcase)))
    } else {
        Ok((clist, None))
    }
}

pub fn case_clause(pair: Pair<Rule>) -> Result<(Vec<PrimaryNode>, Vec<StmtNode>), NodeError> {
    let mut pairs = pair.into_inner();
    let cases = cases(pairs.next().unwrap())?;
    let body = case_body(pairs.next().unwrap())?;

    Ok((cases, body))
}

pub fn default_clause(pair: Pair<Rule>) -> Result<Vec<StmtNode>, NodeError> {
    let mut pairs = pair.into_inner();
    pairs.next().unwrap();
    case_body(pairs.next().unwrap())
}

pub fn cases(pair: Pair<Rule>) -> Result<Vec<PrimaryNode>, NodeError> {
    let mut pairs = pair.into_inner();
    pairs.next().unwrap(); // case
    let mut plist = vec![];
    plist.push(parse_primary_node(pairs.next().unwrap())?);

    while let Some(pair) = pairs.next() {
        plist.push(parse_primary_node(pair)?);
    }

    Ok(plist)
}

pub fn case_body(pair: Pair<Rule>) -> Result<Vec<StmtNode>, NodeError> {
    Ok(parse_stmts(pair.into_inner().next().unwrap())?)
}

#[test]
fn test_switch() {
    assert!(parse_stmt_node(
        CBCScanner::parse(
            Rule::STMT,
            r#"switch (foo) {
                     case 1:
                        sum += 1;
                        break;
                     case 2:
                        sum += 2;
                        break;
                     default:
                        sum = 0;
                        break;
                 }
                 "#
        )
        .unwrap()
        .next()
        .unwrap()
    )
    .is_ok())
}
