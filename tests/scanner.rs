use cbc::{CBCScanner, Rule};
use pest::Parser;

#[derive(Debug, PartialEq, Eq)]
enum Token<'a> {
    Some(&'a str),
    None,
    Remaining,
    ParseError,
}

fn test_one_token<'a>(rule: Rule, input: &'a str) -> Token {
    let ret: Result<pest::iterators::Pairs<Rule>, pest::error::Error<Rule>> =
        CBCScanner::parse(rule, input);

    let Ok(mut ret) = ret else {
        return Token::ParseError;
    };

    let Some(ret) = ret.next().map(|r| r.as_str()) else {
        return Token::None;
    };

    if ret.len() == input.len() {
        Token::Some(ret)
    } else {
        Token::Remaining
    }
}

#[test]
fn identifier() {
    assert_eq!(test_one_token(Rule::IDENTIFIER, "foo"), Token::Some("foo"));
    assert_eq!(
        test_one_token(Rule::IDENTIFIER, "__FOO"),
        Token::Some("__FOO")
    );
    assert_eq!(
        test_one_token(Rule::IDENTIFIER, "foo_bar34812"),
        Token::Some("foo_bar34812")
    );

    assert_eq!(test_one_token(Rule::IDENTIFIER, "0"), Token::ParseError);
    assert_eq!(test_one_token(Rule::IDENTIFIER, "♥"), Token::ParseError);
}

#[test]
fn reserved_words() {
    assert_eq!(test_one_token(Rule::KEYWORD, "void"), Token::Some("void"));
}

#[test]
fn integer() {
    assert_eq!(test_one_token(Rule::INTEGER, "14328"), Token::Some("14328"));
    assert_eq!(
        test_one_token(Rule::INTEGER, "0xa12c"),
        Token::Some("0xa12c")
    );
    assert_eq!(
        test_one_token(Rule::INTEGER, "0o1423762"),
        Token::Some("0o1423762")
    );
    assert_eq!(
        test_one_token(Rule::INTEGER, "0b011011011"),
        Token::Some("0b011011011")
    );

    assert_eq!(test_one_token(Rule::INTEGER, "0xquienc"), Token::ParseError);
    assert_eq!(test_one_token(Rule::INTEGER, "0o98619"), Token::ParseError);
    assert_eq!(
        test_one_token(Rule::INTEGER, "0b212410111"),
        Token::ParseError
    );
}

#[test]
fn comment() {
    assert_eq!(
        test_one_token(Rule::LINE_COMMENT, "// hello world"),
        Token::None
    );
    assert_eq!(
        test_one_token(
            Rule::BLOCK_COMMENT,
            r#"/*
        block comment
        */"#
        ),
        Token::None
    );
}
