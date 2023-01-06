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
        test_one_token(Rule::IDENTIFIER, "jasdfjoiqwefsdaf3812u2u390 o23ur"),
        Token::Remaining("jasdfjoiqwefsdaf3812u2u390", " o23ur", Rule::IDENTIFIER)
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
    assert_eq!(
        test_one_token(Rule::IDENTIFIER, "voidfunction"),
        Token::Some("voidfunction", Rule::IDENTIFIER)
    );
    assert_eq!(
        test_one_token(Rule::IDENTIFIER, "_void"),
        Token::Some("_void", Rule::IDENTIFIER)
    );
    assert_eq!(test_one_token(Rule::IDENTIFIER, "void"), Token::ParseError);
}

#[test]
fn reserved_words() {
    assert_eq!(
        test_one_token(Rule::KEYWORD, "void"),
        Token::Some("void", Rule::VOID)
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

    assert_eq!(
        test_one_token(Rule::INTEGER, "0xquienc"),
        Token::Remaining("0", "xquienc", Rule::INTEGER)
    );
    assert_eq!(
        test_one_token(Rule::INTEGER, "0o98619"),
        Token::Remaining("0", "o98619", Rule::INTEGER)
    );
    assert_eq!(
        test_one_token(Rule::INTEGER, "0b212410111"),
        Token::Remaining("0", "b212410111", Rule::INTEGER)
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
        test_one_token(Rule::IMPORT_STMT, "import stdio;"),
        Token::Some("import stdio;", Rule::IMPORT_STMT)
    );
    assert_eq!(
        test_one_token(Rule::IMPORT_STMT, "import sys.types;"),
        Token::Some("import sys.types;", Rule::IMPORT_STMT)
    );
    assert_eq!(
        test_one_token(Rule::IMPORT_STMT, "importnet;"),
        Token::ParseError
    );
}

#[test]
fn complex_type() {
    assert_eq!(
        test_one_token(Rule::TYPEREF, "int (char* a, ...)*[][]*"),
        Token::Some("int (char* a, ...)*[][]*", Rule::TYPEREF)
    );
}

#[test]
fn params() {
    assert_eq!(
        test_one_token(Rule::PARAMS, "void"),
        Token::Some("void", Rule::PARAMS)
    );
    assert_eq!(
        test_one_token(Rule::PARAMS, "void (int a, int b)* func"),
        Token::Some("void (int a, int b)* func", Rule::PARAMS)
    );
    assert_eq!(
        test_one_token(Rule::PARAMS, "int foo, long bar, char _foo_bar"),
        Token::Some("int foo, long bar, char _foo_bar", Rule::PARAMS)
    );
    assert_eq!(
        test_one_token(Rule::PARAMS, "unsigned short a, unsigned long b"),
        Token::Some("unsigned short a, unsigned long b", Rule::PARAMS)
    );
    assert_eq!(
        test_one_token(Rule::PARAMS, "union shape sphere, struct point x"),
        Token::Some("union shape sphere, struct point x", Rule::PARAMS)
    );
    assert_eq!(
        test_one_token(Rule::PARAMS, "int[5] a, char *b"),
        Token::Some("int[5] a, char *b", Rule::PARAMS)
    );
    assert_eq!(
        test_one_token(Rule::PARAMS, "char *format"),
        Token::Some("char *format", Rule::PARAMS)
    );
}

#[test]
fn def_function() {
    assert_eq!(
        test_one_token(Rule::DEF_FUNCTION, "int add(int x, int y);"),
        Token::Some("int add(int x, int y);", Rule::DEF_FUNCTION)
    );
    assert_eq!(
        test_one_token(Rule::DEF_FUNCTION, "int add(x, y)"),
        Token::ParseError
    );
}

#[test]
fn member_list() {
    assert_eq!(
        test_one_token(Rule::MEMBER_LIST, "{ int x; long y; short z; }"),
        Token::Some("{ int x; long y; short z; }", Rule::MEMBER_LIST)
    );
}

#[test]
fn def_struct() {
    assert_eq!(
        test_one_token(Rule::DEF_STRUCT, "struct point { int x; int y; }"),
        Token::Some("struct point { int x; int y; }", Rule::DEF_STRUCT)
    );
}

#[test]
fn def_union() {
    assert_eq!(
        test_one_token(Rule::DEF_UNION, "union value { int i; float f; }"),
        Token::Some("union value { int i; float f; }", Rule::DEF_UNION)
    );
}

#[test]
fn def_type() {
    assert_eq!(
        test_one_token(Rule::DEF_TYPE, "typedef struct point Point;"),
        Token::Some("typedef struct point Point;", Rule::DEF_TYPE)
    );
}

#[test]
fn primary() {
    assert_eq!(
        test_one_token(Rule::PRIMARY, "0"),
        Token::Some("0", Rule::PRIMARY)
    );
    assert_eq!(
        test_one_token(Rule::PRIMARY, "0x126"),
        Token::Some("0x126", Rule::PRIMARY)
    );
    assert_eq!(
        test_one_token(Rule::PRIMARY, r#""Hello""#),
        Token::Some(r#""Hello""#, Rule::PRIMARY)
    );
    assert_eq!(
        test_one_token(Rule::PRIMARY, "'a'"),
        Token::Some("'a'", Rule::PRIMARY)
    );
    assert_eq!(
        test_one_token(Rule::PRIMARY, "abc_def"),
        Token::Some("abc_def", Rule::PRIMARY)
    );
}

#[test]
fn postfix() {
    assert_eq!(
        test_one_token(Rule::POSTFIX, "foo++"),
        Token::Some("foo++", Rule::POSTFIX)
    );
    assert_eq!(
        test_one_token(Rule::POSTFIX, "foo->length"),
        Token::Some("foo->length", Rule::POSTFIX)
    );
}

#[test]
fn unary() {
    assert_eq!(
        test_one_token(Rule::UNARY, "--_foo"),
        Token::Some("--_foo", Rule::UNARY)
    );
    assert_eq!(
        test_one_token(Rule::UNARY, "!bar"),
        Token::Some("!bar", Rule::UNARY)
    );
}

#[test]
fn args() {
    assert_eq!(
        test_one_token(Rule::ARGS, "a, b, 13"),
        Token::Some("a, b, 13", Rule::ARGS)
    );
}

#[test]
fn term() {
    assert_eq!(
        test_one_token(Rule::TERM, "(int)f"),
        Token::Some("(int)f", Rule::TERM)
    );
    assert_eq!(
        test_one_token(Rule::TERM, "1"),
        Token::Some("1", Rule::TERM)
    );
}

#[test]
fn expr() {
    assert_eq!(
        test_one_token(Rule::EXPR, "2 * 3 + 4 + 6 || b"),
        Token::Some("2 * 3 + 4 + 6 || b", Rule::EXPR)
    );
}

#[test]
fn stmts() {
    assert_eq!(
        test_one_token(Rule::BLOCK, "{}"),
        Token::Some("{}", Rule::BLOCK)
    );

    assert_eq!(
        test_one_token(Rule::IF_STMT, "if (a == b) { a += 1; } else { a = b; }"),
        Token::Some("if (a == b) { a += 1; } else { a = b; }", Rule::IF_STMT)
    );

    assert_eq!(
        test_one_token(Rule::IF_STMT, "if (a == b) { a += 1; } else { a = b; }"),
        Token::Some("if (a == b) { a += 1; } else { a = b; }", Rule::IF_STMT)
    );

    assert_eq!(
        test_one_token(Rule::WHILE_STMT, "while (counter > 0) counter--;"),
        Token::Some("while (counter > 0) counter--;", Rule::WHILE_STMT)
    );

    assert_eq!(
        test_one_token(
            Rule::DOWHILE_STMT,
            "do { counter += 1; } while (counter <= 10);"
        ),
        Token::Some(
            "do { counter += 1; } while (counter <= 10);",
            Rule::DOWHILE_STMT
        )
    );

    assert_eq!(
        test_one_token(Rule::FOR_STMT, "for (i = 0; i < 100; i++) { sum += i; }"),
        Token::Some("for (i = 0; i < 100; i++) { sum += i; }", Rule::FOR_STMT)
    );
}
