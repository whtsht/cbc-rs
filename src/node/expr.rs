use super::*;
use pest::iterators::{Pair, Pairs};
use std::iter::Peekable;

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Mul,
    Div,
    Mod,
    Add,
    Sub,
    Shl,
    Shr,
    And,
    Or,
    BitAnd,
    BitOr,
    BitExOr,
    Ge,
    Le,
    Gt,
    Lt,
    Eq,
    Ne,
}

#[derive(Debug, Clone)]
pub enum TernaryOp {
    If,
}

#[derive(Debug, Clone)]
pub enum AssignOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Exor,
    Shl,
    Shr,
}

#[derive(Debug, Clone)]
pub enum ExprNode {
    Term(Node),
    Assign {
        term: Node,
        expr: Box<ExprNode>,
    },
    AssignOp {
        op: AssignOp,
        term: Node,
        expr: Box<ExprNode>,
    },
    BinaryOp {
        op: BinaryOp,
        lhs: Box<ExprNode>,
        rhs: Box<ExprNode>,
    },
    TernaryOp {
        op: TernaryOp,
        lhs: Box<ExprNode>,
        mhs: Box<ExprNode>,
        rhs: Box<ExprNode>,
    },
}

pub fn parse_expr_node(pair: Pair<Rule>) -> Result<Node, NodeError> {
    debug_assert_eq!(pair.as_rule(), Rule::EXPR);
    let mut pairs = pair.into_inner().peekable();

    match pairs.peek().unwrap().as_rule() {
        Rule::EXPR10 => Ok(Node::Expr(Box::new(expr10(
            &mut pairs.next().unwrap().into_inner(),
        )?))),
        Rule::TERM => Ok(Node::Expr(Box::new(assign_op(pairs)?))),
        _ => todo!(),
    }
}

pub fn assign_op(mut pairs: Peekable<Pairs<Rule>>) -> Result<ExprNode, NodeError> {
    let term = parse_term_node(pairs.next().unwrap())?;

    if pairs.peek().unwrap().as_rule() == Rule::EQ {
        pairs.next();
        let expr = Box::new(expr10(&mut pairs.next().unwrap().into_inner())?);
        return Ok(ExprNode::Assign { term, expr });
    }

    let op = match pairs.next().unwrap().as_str() {
        "+=" => AssignOp::Add,
        "-=" => AssignOp::Sub,
        "*=" => AssignOp::Mul,
        "/=" => AssignOp::Div,
        "%=" => AssignOp::Mod,
        "&=" => AssignOp::And,
        "|=" => AssignOp::Or,
        "^=" => AssignOp::Exor,
        "<<=" => AssignOp::Shl,
        ">>=" => AssignOp::Shr,
        _ => todo!(),
    };
    let expr = Box::new(expr10(&mut pairs.next().unwrap().into_inner())?);

    Ok(ExprNode::AssignOp { op, term, expr })
}

pub fn expr10(pairs: &mut Pairs<Rule>) -> Result<ExprNode, NodeError> {
    let expr = expr9(pairs.next().unwrap().into_inner())?;

    if let Some(pair) = pairs.next() {
        if pair.as_rule() != Rule::QUESTION {
            Err(NodeError {
                _type: NodeErrorType::Expr,
                message: "expected '?'".into(),
            })?
        }

        let _then = expr10(&mut pairs.next().unwrap().into_inner())?;

        if pairs.next().unwrap().as_rule() != Rule::COLON {
            Err(NodeError {
                _type: NodeErrorType::Expr,
                message: "expected ':'".into(),
            })?
        }

        let _else = expr10(&mut pairs.next().unwrap().into_inner())?;

        Ok(ExprNode::TernaryOp {
            op: TernaryOp::If,
            lhs: Box::new(expr),
            mhs: Box::new(_then),
            rhs: Box::new(_else),
        })
    } else {
        Ok(expr)
    }
}

pub fn expr9(mut pairs: Pairs<Rule>) -> Result<ExprNode, NodeError> {
    let mut expr = expr8(pairs.next().unwrap().into_inner())?;

    if let Some(pair) = pairs.next() {
        match pair.as_rule() {
            Rule::OOR => {
                expr = ExprNode::BinaryOp {
                    op: BinaryOp::Or,
                    lhs: Box::new(expr9(pairs)?),
                    rhs: Box::new(expr),
                };
            }
            _ => todo!(),
        }
    }

    Ok(expr)
}

pub fn expr8(mut pairs: Pairs<Rule>) -> Result<ExprNode, NodeError> {
    let mut expr = expr7(pairs.next().unwrap().into_inner())?;

    if let Some(pair) = pairs.next() {
        match pair.as_rule() {
            Rule::AAND => {
                expr = ExprNode::BinaryOp {
                    op: BinaryOp::And,
                    lhs: Box::new(expr8(pairs)?),
                    rhs: Box::new(expr),
                };
            }
            _ => todo!(),
        }
    }

    Ok(expr)
}

pub fn expr7(mut pairs: Pairs<Rule>) -> Result<ExprNode, NodeError> {
    let mut expr = expr6(pairs.next().unwrap().into_inner())?;

    if let Some(pair) = pairs.next() {
        match pair.as_rule() {
            Rule::GE => {
                expr = ExprNode::BinaryOp {
                    op: BinaryOp::Ge,
                    lhs: Box::new(expr7(pairs)?),
                    rhs: Box::new(expr),
                };
            }
            Rule::LE => {
                expr = ExprNode::BinaryOp {
                    op: BinaryOp::Le,
                    lhs: Box::new(expr7(pairs)?),
                    rhs: Box::new(expr),
                };
            }
            Rule::GT => {
                expr = ExprNode::BinaryOp {
                    op: BinaryOp::Gt,
                    lhs: Box::new(expr7(pairs)?),
                    rhs: Box::new(expr),
                };
            }
            Rule::LT => {
                expr = ExprNode::BinaryOp {
                    op: BinaryOp::Lt,
                    lhs: Box::new(expr7(pairs)?),
                    rhs: Box::new(expr),
                };
            }
            Rule::EEQ => {
                expr = ExprNode::BinaryOp {
                    op: BinaryOp::Eq,
                    lhs: Box::new(expr7(pairs)?),
                    rhs: Box::new(expr),
                };
            }
            Rule::NE => {
                expr = ExprNode::BinaryOp {
                    op: BinaryOp::Ne,
                    lhs: Box::new(expr7(pairs)?),
                    rhs: Box::new(expr),
                };
            }
            _ => todo!(),
        }
    }

    Ok(expr)
}

pub fn expr6(mut pairs: Pairs<Rule>) -> Result<ExprNode, NodeError> {
    let mut expr = expr5(pairs.next().unwrap().into_inner())?;

    if let Some(pair) = pairs.next() {
        match pair.as_rule() {
            Rule::OR => {
                expr = ExprNode::BinaryOp {
                    op: BinaryOp::BitOr,
                    lhs: Box::new(expr6(pairs)?),
                    rhs: Box::new(expr),
                };
            }
            _ => todo!(),
        }
    }

    Ok(expr)
}

pub fn expr5(mut pairs: Pairs<Rule>) -> Result<ExprNode, NodeError> {
    let mut expr = expr4(pairs.next().unwrap().into_inner())?;

    if let Some(pair) = pairs.next() {
        match pair.as_rule() {
            Rule::CARET => {
                expr = ExprNode::BinaryOp {
                    op: BinaryOp::BitExOr,
                    lhs: Box::new(expr5(pairs)?),
                    rhs: Box::new(expr),
                };
            }
            _ => todo!(),
        }
    }

    Ok(expr)
}

pub fn expr4(mut pairs: Pairs<Rule>) -> Result<ExprNode, NodeError> {
    let mut expr = expr3(pairs.next().unwrap().into_inner())?;

    if let Some(pair) = pairs.next() {
        match pair.as_rule() {
            Rule::AND => {
                expr = ExprNode::BinaryOp {
                    op: BinaryOp::BitAnd,
                    lhs: Box::new(expr4(pairs)?),
                    rhs: Box::new(expr),
                };
            }
            _ => todo!(),
        }
    }

    Ok(expr)
}

pub fn expr3(mut pairs: Pairs<Rule>) -> Result<ExprNode, NodeError> {
    let mut expr = expr2(pairs.next().unwrap().into_inner())?;

    if let Some(pair) = pairs.next() {
        match pair.as_rule() {
            Rule::SHL => {
                expr = ExprNode::BinaryOp {
                    op: BinaryOp::Shl,
                    lhs: Box::new(expr3(pairs)?),
                    rhs: Box::new(expr),
                };
            }
            Rule::SHR => {
                expr = ExprNode::BinaryOp {
                    op: BinaryOp::Shr,
                    lhs: Box::new(expr3(pairs)?),
                    rhs: Box::new(expr),
                };
            }
            _ => todo!(),
        }
    }

    Ok(expr)
}

pub fn expr2(mut pairs: Pairs<Rule>) -> Result<ExprNode, NodeError> {
    let mut expr = expr1(pairs.next().unwrap().into_inner())?;

    if let Some(pair) = pairs.next() {
        match pair.as_rule() {
            Rule::PLUS => {
                expr = ExprNode::BinaryOp {
                    op: BinaryOp::Add,
                    lhs: Box::new(expr2(pairs)?),
                    rhs: Box::new(expr),
                };
            }
            Rule::MINUS => {
                expr = ExprNode::BinaryOp {
                    op: BinaryOp::Sub,
                    lhs: Box::new(expr2(pairs)?),
                    rhs: Box::new(expr),
                };
            }
            _ => todo!(),
        }
    }

    Ok(expr)
}

pub fn expr1(mut pairs: Pairs<Rule>) -> Result<ExprNode, NodeError> {
    let mut expr = ExprNode::Term(parse_term_node(pairs.next().unwrap())?);

    if let Some(pair) = pairs.next() {
        match pair.as_rule() {
            Rule::STAR => {
                expr = ExprNode::BinaryOp {
                    op: BinaryOp::Mul,
                    lhs: Box::new(expr1(pairs)?),
                    rhs: Box::new(expr),
                };
            }
            Rule::SLASH => {
                expr = ExprNode::BinaryOp {
                    op: BinaryOp::Div,
                    lhs: Box::new(expr1(pairs)?),
                    rhs: Box::new(expr),
                };
            }
            Rule::PERCENT => {
                expr = ExprNode::BinaryOp {
                    op: BinaryOp::Mod,
                    lhs: Box::new(expr1(pairs)?),
                    rhs: Box::new(expr),
                };
            }
            _ => todo!(),
        }
    }

    Ok(expr)
}

#[test]
fn test_expr() {
    assert!(parse_expr_node(
        CBCScanner::parse(Rule::EXPR, "3 * 2 * 5")
            .unwrap()
            .next()
            .unwrap()
    )
    .is_ok());

    assert!(parse_expr_node(
        CBCScanner::parse(Rule::EXPR, "1 + 3 * 2 + 4")
            .unwrap()
            .next()
            .unwrap()
    )
    .is_ok());

    assert!(parse_expr_node(
        CBCScanner::parse(Rule::EXPR, "1 + 3 * 2 << 3 + 4 >> 6")
            .unwrap()
            .next()
            .unwrap()
    )
    .is_ok());

    assert!(parse_expr_node(
        CBCScanner::parse(Rule::EXPR, "1 & 4")
            .unwrap()
            .next()
            .unwrap()
    )
    .is_ok());

    assert!(parse_expr_node(
        CBCScanner::parse(Rule::EXPR, "1 & 4 ^ 5")
            .unwrap()
            .next()
            .unwrap()
    )
    .is_ok());

    assert!(parse_expr_node(
        CBCScanner::parse(Rule::EXPR, "1 & 4 ^ 5 | 6")
            .unwrap()
            .next()
            .unwrap()
    )
    .is_ok());

    assert!(parse_expr_node(
        CBCScanner::parse(Rule::EXPR, "a >= b >= c < d > e == f != g")
            .unwrap()
            .next()
            .unwrap()
    )
    .is_ok());

    assert!(parse_expr_node(
        CBCScanner::parse(Rule::EXPR, "1 < x && x < 9 || 10 > y && y > 9")
            .unwrap()
            .next()
            .unwrap()
    )
    .is_ok());

    assert!(parse_expr_node(
        CBCScanner::parse(Rule::EXPR, "x > 10 ? 10 : x")
            .unwrap()
            .next()
            .unwrap()
    )
    .is_ok());

    assert!(parse_expr_node(
        CBCScanner::parse(Rule::EXPR, "x *= 1")
            .unwrap()
            .next()
            .unwrap()
    )
    .is_ok());

    assert!(parse_expr_node(
        CBCScanner::parse(Rule::EXPR, "x = 'c'")
            .unwrap()
            .next()
            .unwrap()
    )
    .is_ok());
}
