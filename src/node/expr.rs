use super::*;
use pest::iterators::{Pair, Pairs};

#[derive(Debug)]
pub enum ExprType {
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
    If,
}

#[derive(Debug)]
pub enum ExprNode {
    Term(Node),
    BinaryOp {
        _type: ExprType,
        lhs: Box<ExprNode>,
        rhs: Box<ExprNode>,
    },
    TernaryOp {
        _type: ExprType,
        lhs: Box<ExprNode>,
        mhs: Box<ExprNode>,
        rhs: Box<ExprNode>,
    },
}

pub fn parse_expr_node(pair: Pair<Rule>) -> Result<Node, NodeError> {
    debug_assert_eq!(pair.as_rule(), Rule::EXPR);
    let mut pairs = pair.into_inner();
    Ok(Node::Expr(Box::new(expr10(
        &mut pairs.next().unwrap().into_inner(),
    )?)))
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
            _type: ExprType::If,
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
                    _type: ExprType::Or,
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
                    _type: ExprType::And,
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
                    _type: ExprType::Ge,
                    lhs: Box::new(expr7(pairs)?),
                    rhs: Box::new(expr),
                };
            }
            Rule::LE => {
                expr = ExprNode::BinaryOp {
                    _type: ExprType::Le,
                    lhs: Box::new(expr7(pairs)?),
                    rhs: Box::new(expr),
                };
            }
            Rule::GT => {
                expr = ExprNode::BinaryOp {
                    _type: ExprType::Gt,
                    lhs: Box::new(expr7(pairs)?),
                    rhs: Box::new(expr),
                };
            }
            Rule::LT => {
                expr = ExprNode::BinaryOp {
                    _type: ExprType::Lt,
                    lhs: Box::new(expr7(pairs)?),
                    rhs: Box::new(expr),
                };
            }
            Rule::EEQ => {
                expr = ExprNode::BinaryOp {
                    _type: ExprType::Eq,
                    lhs: Box::new(expr7(pairs)?),
                    rhs: Box::new(expr),
                };
            }
            Rule::NE => {
                expr = ExprNode::BinaryOp {
                    _type: ExprType::Ne,
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
                    _type: ExprType::BitOr,
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
                    _type: ExprType::BitExOr,
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
                    _type: ExprType::BitAnd,
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
                    _type: ExprType::Shl,
                    lhs: Box::new(expr3(pairs)?),
                    rhs: Box::new(expr),
                };
            }
            Rule::SHR => {
                expr = ExprNode::BinaryOp {
                    _type: ExprType::Shr,
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
                    _type: ExprType::Add,
                    lhs: Box::new(expr2(pairs)?),
                    rhs: Box::new(expr),
                };
            }
            Rule::MINUS => {
                expr = ExprNode::BinaryOp {
                    _type: ExprType::Sub,
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
                    _type: ExprType::Mul,
                    lhs: Box::new(expr1(pairs)?),
                    rhs: Box::new(expr),
                };
            }
            Rule::SLASH => {
                expr = ExprNode::BinaryOp {
                    _type: ExprType::Div,
                    lhs: Box::new(expr1(pairs)?),
                    rhs: Box::new(expr),
                };
            }
            Rule::PERCENT => {
                expr = ExprNode::BinaryOp {
                    _type: ExprType::Mod,
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
}
