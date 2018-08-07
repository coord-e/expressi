use expression::Expression;
use expression::Operator;

pub mod syntax {
    include!(concat!(env!("OUT_DIR"), "/syntax.rs"));
}

pub fn parse(x: &str) -> Result<Expression, syntax::ParseError> {
    syntax::expression(x)
}

#[test]
fn test_number() {
    assert_eq!(parse("0"), Ok(Expression::Number(0)))
}

#[test]
fn test_multidigit_number() {
    assert_eq!(parse("10"), Ok(Expression::Number(10)))
}

#[test]
fn test_identifier() {
    assert_eq!(parse("abc"), Ok(Expression::Identifier("abc".to_string())))
}

#[test]
fn test_onechar_identifier() {
    assert_eq!(parse("a"), Ok(Expression::Identifier("a".to_string())))
}

#[test]
fn test_invalid_identifier() {
    assert!(parse("_a").is_err());
    assert!(parse("0a").is_err())
}

macro_rules! test_BinOp {
    ($x:expr, $op:expr) => {
        assert_eq!(parse($x), Ok(Expression::BinOp($op, Box::new(Expression::Number(0)), Box::new(Expression::Number(0)))))
    }
}

#[test]
fn test_operator_add() {
    test_BinOp!("0+0", Operator::Add)
}

#[test]
fn test_operator_sub() {
    test_BinOp!("0-0", Operator::Sub)
}

#[test]
fn test_operator_mul() {
    test_BinOp!("0*0", Operator::Mul)
}

#[test]
fn test_operator_div() {
    test_BinOp!("0/0", Operator::Div)
}
