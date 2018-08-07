use expression::Expression;

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
