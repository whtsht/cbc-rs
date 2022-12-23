use cbc::{CBCParser, Rule};
use pest::Parser;

pub fn test_one_token(rule: Rule, input: &str) -> Option<&str> {
    let ret = CBCParser::parse(rule, input);
    let ret = ret.ok()?.next()?.as_str();

    if ret.len() == input.len() {
        Some(ret)
    } else {
        None
    }
}

#[test]
fn identifier() {
    assert!(test_one_token(Rule::IDENTIFIER, "foo").is_some());
    assert!(test_one_token(Rule::IDENTIFIER, "__foo").is_some());
    assert!(test_one_token(Rule::IDENTIFIER, "foo_bar34812").is_some());

    assert!(test_one_token(Rule::IDENTIFIER, "0").is_none());
    assert!(test_one_token(Rule::IDENTIFIER, "♥").is_none());
}

#[test]
fn reserved_words() {
    assert!(test_one_token(Rule::KEYWORD, "void").is_some());
}

#[test]
fn integer() {
    assert!(test_one_token(Rule::INTEGER, "14328").is_some());
    assert!(test_one_token(Rule::INTEGER, "0xa12c").is_some());
    assert!(test_one_token(Rule::INTEGER, "0o1423762").is_some());
    assert!(test_one_token(Rule::INTEGER, "0b011011011").is_some());

    assert!(test_one_token(Rule::INTEGER, "0xquienc").is_none());
    assert!(test_one_token(Rule::INTEGER, "0o198619").is_none());
    assert!(test_one_token(Rule::INTEGER, "0b011121241").is_none());
}
