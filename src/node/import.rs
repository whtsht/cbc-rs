use super::{Node, NodeError};
use crate::Rule;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct ImportNode {
    names: Vec<String>,
}

pub fn parse_import_node(pair: Pair<Rule>) -> Result<Node, NodeError> {
    let mut pairs = pair.into_inner();

    pairs.next().unwrap(); // import
    let mut names = vec![];
    names.push(pairs.next().unwrap().as_str().into());

    while let Some(pair) = pairs.next() {
        names.push(pair.as_str().into());
    }

    Ok(Node::Import(Box::new(ImportNode { names })))
}

#[test]
fn test_import() {
    use crate::CBCScanner;
    use pest::Parser;
    assert!(parse_import_node(
        CBCScanner::parse(Rule::IMPORT_STMT, "import std.file.open;")
            .unwrap()
            .next()
            .unwrap()
    )
    .is_ok());
}
