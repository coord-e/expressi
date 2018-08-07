use expression::Expression;

pub mod syntax {
    include!(concat!(env!("OUT_DIR"), "/syntax.rs"));
}

pub fn parse(x: &str) -> Result<Expression, syntax::ParseError> {
    syntax::expression(x)
}

#[cfg(test)]
mod tests {
    use expression::Operator;
    use super::*;

    #[test]
    fn number() {
        assert_eq!(parse("0"), Ok(Expression::Number(0)))
    }

    #[test]
    fn multidigit_number() {
        assert_eq!(parse("10"), Ok(Expression::Number(10)))
    }

    #[test]
    fn identifier() {
        assert_eq!(parse("abc"), Ok(Expression::Identifier("abc".to_string())))
    }

    #[test]
    fn onechar_identifier() {
        assert_eq!(parse("a"), Ok(Expression::Identifier("a".to_string())))
    }

    #[test]
    fn invalid_identifier() {
        assert!(parse("_a").is_err());
        assert!(parse("0a").is_err())
    }

    macro_rules! BinOp {
        ($x:expr, $op:expr) => {
            assert_eq!(parse($x), Ok(Expression::BinOp($op, Box::new(Expression::Number(0)), Box::new(Expression::Number(0)))))
        }
    }

    #[test]
    fn operator_add() {
        BinOp!("0+0", Operator::Add)
    }

    #[test]
    fn operator_sub() {
        BinOp!("0-0", Operator::Sub)
    }

    #[test]
    fn operator_mul() {
        BinOp!("0*0", Operator::Mul)
    }

    #[test]
    fn operator_div() {
        BinOp!("0/0", Operator::Div)
    }

    #[test]
    fn operator_precedence() {
        assert_eq!(parse("0+0*0"), Ok(Expression::BinOp(Operator::Add, Box::new(Expression::Number(0)), Box::new(Expression::BinOp(Operator::Mul, Box::new(Expression::Number(0)), Box::new(Expression::Number(0)))))))
    }
}
