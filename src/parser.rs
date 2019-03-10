use crate::expression::Expression;

#[allow(clippy::all)]
pub mod syntax {
    include!(concat!(env!("OUT_DIR"), "/syntax.rs"));
}

pub fn parse(x: &str) -> Result<Expression, syntax::ParseError> {
    syntax::expression(x)
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn skip_space() {
        assert_eq!(parse("1 +\n1 "), parse("1+1"))
    }

    #[test]
    fn skip_comment() {
        assert_eq!(parse("1+/* This is a comment */1"), parse("1+1"))
    }

    #[test]
    fn skip_comment_multiline() {
        assert_eq!(parse("1+/* This is \na comment */1"), parse("1+1"))
    }

    #[test]
    fn skip_line_comment() {
        assert_eq!(parse("1+1// This is a comment"), parse("1+1"))
    }

    #[test]
    fn skip_line_comment_follow() {
        assert_eq!(parse("1+1;// This is a comment\n1+1"), parse("1+1;1+1"))
    }

    mod literal {
        use super::parse;
        use crate::expression::Expression;

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
        use super::parse;
        use crate::expression::{Expression, Operator};

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
        fn operator_index() {
            test_binop!("0[0]", Operator::Index);
        }

        #[test]
        fn operator_index_spaced() {
            test_binop!("0 [0]", Operator::Index);
        }

        #[test]
        fn operator_index_multi() {
            assert_eq!(
                parse("a[0][0]"),
                Ok(Expression::BinOp(
                    Operator::Index,
                    Box::new(Expression::BinOp(
                        Operator::Index,
                        Box::new(Expression::Identifier("a".to_owned())),
                        Box::new(Expression::Number(0))
                    )),
                    Box::new(Expression::Number(0))
                ))
            )
        }

        #[test]
        fn apply() {
            assert_eq!(
                parse("a(0)"),
                Ok(Expression::Apply(
                    Box::new(Expression::Identifier("a".to_owned())),
                    Box::new(Expression::Number(0))
                ))
            )
        }

        #[test]
        fn apply_spaced() {
            assert_eq!(
                parse("a (0)"),
                Ok(Expression::Apply(
                    Box::new(Expression::Identifier("a".to_owned())),
                    Box::new(Expression::Number(0))
                ))
            )
        }

        #[test]
        fn apply_with_outer_bracket() {
            assert_eq!(
                parse("( a(0) )"),
                Ok(Expression::Apply(
                    Box::new(Expression::Identifier("a".to_owned())),
                    Box::new(Expression::Number(0))
                ))
            )
        }

        #[test]
        fn apply_multi() {
            assert_eq!(
                parse("a(0)(0)"),
                Ok(Expression::Apply(
                    Box::new(Expression::Apply(
                        Box::new(Expression::Identifier("a".to_owned())),
                        Box::new(Expression::Number(0))
                    )),
                    Box::new(Expression::Number(0))
                ))
            )
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

        #[test]
        fn precedence_function_follow() {
            assert_eq!(
                parse("a -> a; 0"),
                Ok(Expression::Follow(
                    Box::new(Expression::Function(
                        "a".to_string(),
                        Box::new(Expression::Identifier("a".to_string()))
                    )),
                    Box::new(Expression::Number(0))
                ))
            )
        }
    }

    mod sugar {
        use super::parse;

        #[test]
        fn function_params() {
            assert_eq!(parse("(a,b,c)->0"), parse("a->b->c->0"))
        }

        #[test]
        fn application() {
            assert_eq!(parse("f(a,b,c)"), parse("f(a)(b)(c)"))
        }
    }
}
