use expression::Expression;

pub mod syntax {
    include!(concat!(env!("OUT_DIR"), "/syntax.rs"));
}

pub fn parse(x: &str) -> Result<Expression, syntax::ParseError> {
    syntax::expression(x)
}

#[cfg(test)]
mod tests {
    mod literal {
        use expression::{Expression, Operator};
        use parser::parse;

        #[test]
        fn number() {
            assert_eq!(parse("0"), Ok(Expression::Number(0)))
        }

        #[test]
        fn boolean() {
            assert_eq!(parse("true"), Ok(Expression::Boolean(true)));
            assert_eq!(parse("false"), Ok(Expression::Boolean(false)));
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
    }

    mod operator {
        use expression::{Expression, Operator};
        use parser::parse;

        macro_rules! test_binop {
            ($x:expr, $op:expr) => {
                assert_eq!(
                    parse($x),
                    Ok(Expression::BinOp(
                        $op,
                        Box::new(Expression::Number(0)),
                        Box::new(Expression::Number(0))
                    ))
                )
            };
        }

        #[test]
        fn operator_sum() {
            test_binop!("0+0", Operator::Add);
            test_binop!("0-0", Operator::Sub);
        }

        #[test]
        fn operator_mul() {
            test_binop!("0*0", Operator::Mul);
            test_binop!("0/0", Operator::Div);
        }

        #[test]
        fn operator_bit() {
            test_binop!("0&0", Operator::BitAnd);
            test_binop!("0^0", Operator::BitXor);
            test_binop!("0|0", Operator::BitOr);
        }

        #[test]
        fn operator_comp() {
            test_binop!("0<0", Operator::Lt);
            test_binop!("0>0", Operator::Gt);
            test_binop!("0<=0", Operator::Le);
            test_binop!("0>=0", Operator::Ge);
            test_binop!("0==0", Operator::Eq);
            test_binop!("0!=0", Operator::Ne);
        }

        #[test]
        fn assign() {
            assert_eq!(
                parse("a=0"),
                Ok(Expression::Assign(
                    Box::new(Expression::Identifier("a".to_owned())),
                    Box::new(Expression::Number(0))
                ))
            )
        }

        #[test]
        fn follow() {
            assert_eq!(
                parse("0;0"),
                Ok(Expression::Follow(
                    Box::new(Expression::Number(0)),
                    Box::new(Expression::Number(0))
                ))
            )
        }

        #[test]
        fn ifelse() {
            assert_eq!(
                parse("if true 0 else 0"),
                Ok(Expression::IfElse(
                    Box::new(Expression::Boolean(true)),
                    Box::new(Expression::Number(0)),
                    Box::new(Expression::Number(0))
                ))
            )
        }

        #[test]
        fn precedence_arith() {
            assert_eq!(
                parse("0+0*0"),
                Ok(Expression::BinOp(
                    Operator::Add,
                    Box::new(Expression::Number(0)),
                    Box::new(Expression::BinOp(
                        Operator::Mul,
                        Box::new(Expression::Number(0)),
                        Box::new(Expression::Number(0))
                    ))
                ))
            )
        }

        #[test]
        fn precedence_arith_bracket() {
            assert_eq!(
                parse("(0+0)*0"),
                Ok(Expression::BinOp(
                    Operator::Mul,
                    Box::new(Expression::BinOp(
                        Operator::Add,
                        Box::new(Expression::Number(0)),
                        Box::new(Expression::Number(0))
                    )),
                    Box::new(Expression::Number(0))
                ))
            )
        }

        #[test]
        fn precedence_assign_follow() {
            assert_eq!(
                parse("a=0;0"),
                Ok(Expression::Follow(
                    Box::new(Expression::Assign(
                        Box::new(Expression::Identifier("a".to_owned())),
                        Box::new(Expression::Number(0))
                    )),
                    Box::new(Expression::Number(0))
                ))
            )
        }

        #[test]
        fn precedence_ifelse_follow() {
            assert_eq!(
                parse("if true 0 else 0;0"),
                Ok(Expression::Follow(
                    Box::new(Expression::IfElse(
                        Box::new(Expression::Boolean(true)),
                        Box::new(Expression::Number(0)),
                        Box::new(Expression::Number(0))
                    )),
                    Box::new(Expression::Number(0))
                ))
            )
        }
    }
}
