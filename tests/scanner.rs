use cbc::{CBCScanner, Rule};
use pest::Parser;

#[derive(Debug, PartialEq, Eq)]
enum Token<'a> {
    Some(&'a str, Rule),
    None,
    Remaining(&'a str, &'a str, Rule),
    ParseError,
}

fn test_one_token<'a>(rule: Rule, input: &'a str) -> Token {
    let ret: Result<pest::iterators::Pairs<Rule>, pest::error::Error<Rule>> =
        CBCScanner::parse(rule, input);

    let Ok(mut ret) = ret else {
        return Token::ParseError;
    };

    let Some((string, rule)) = ret.next().map(|r| (r.as_str(), r.as_rule())) else {
        return Token::None;
    };

    if string.len() == input.len() {
        Token::Some(string, rule)
    } else {
        Token::Remaining(&string, &input[string.len()..], rule)
    }
}

#[test]
fn identifier() {
    assert_eq!(
        test_one_token(Rule::IDENTIFIER, "foo"),
        Token::Some("foo", Rule::IDENTIFIER)
    );
    assert_eq!(
        test_one_token(Rule::IDENTIFIER, "__FOO"),
        Token::Some("__FOO", Rule::IDENTIFIER)
    );
    assert_eq!(
        test_one_token(Rule::IDENTIFIER, "foo bar"),
        Token::Remaining("foo", " bar", Rule::IDENTIFIER)
    );

    assert_eq!(test_one_token(Rule::IDENTIFIER, "0"), Token::ParseError);
    assert_eq!(test_one_token(Rule::IDENTIFIER, "♥"), Token::ParseError);
}

#[test]
fn reserved_words() {
    assert_eq!(
        test_one_token(Rule::KEYWORD, "void"),
        Token::Some("void", Rule::KEYWORD)
    );
}

#[test]
fn integer() {
    assert_eq!(
        test_one_token(Rule::INTEGER, "14328"),
        Token::Some("14328", Rule::INTEGER)
    );
    assert_eq!(
        test_one_token(Rule::INTEGER, "0xa12c"),
        Token::Some("0xa12c", Rule::INTEGER)
    );
    assert_eq!(
        test_one_token(Rule::INTEGER, "0o1423762"),
        Token::Some("0o1423762", Rule::INTEGER)
    );
    assert_eq!(
        test_one_token(Rule::INTEGER, "0b011011011"),
        Token::Some("0b011011011", Rule::INTEGER)
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

#[test]
fn import_statement() {
    assert_eq!(
        test_one_token(Rule::IMPORT_STATEMENT, "import stdio;"),
        Token::Some("import stdio;", Rule::IMPORT_STATEMENT)
    );
    assert_eq!(
        test_one_token(Rule::IMPORT_STATEMENT, "import sys.types;"),
        Token::Some("import sys.types;", Rule::IMPORT_STATEMENT)
    );
    assert_eq!(
        test_one_token(Rule::IMPORT_STATEMENT, "importnet;"),
        Token::ParseError
    );
}
