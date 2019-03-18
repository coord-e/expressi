use crate::error::ParseError;
use crate::expression::Expression;

use failure::Error;

#[allow(clippy::all)]
pub mod syntax {
    include!(concat!(env!("OUT_DIR"), "/syntax.rs"));
}

pub fn parse(x: &str) -> Result<Expression, Error> {
    syntax::expression(x).map_err(|e| {
        ParseError {
            message: e.to_string(),
        }
        .into()
    })
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn skip_space() {
        assert_eq!(parse("1 +\n1 ").unwrap(), parse("1+1").unwrap())
    }

    #[test]
    fn skip_comment() {
        assert_eq!(
            parse("1+/* This is a comment */1").unwrap(),
            parse("1+1").unwrap()
        )
    }

    #[test]
    fn skip_comment_multiline() {
        assert_eq!(
            parse("1+/* This is \na comment */1").unwrap(),
            parse("1+1").unwrap()
        )
    }

    #[test]
    fn skip_line_comment() {
        assert_eq!(
            parse("1+1// This is a comment").unwrap(),
            parse("1+1").unwrap()
        )
    }

    #[test]
    fn skip_line_comment_follow() {
        assert_eq!(
            parse("1+1;// This is a comment\n1+1").unwrap(),
            parse("1+1;1+1").unwrap()
        )
    }

    mod literal {
        use super::parse;
        use crate::expression::Expression;

        #[test]
        fn number() {
            assert_eq!(parse("0").unwrap(), Expression::Number(0))
        }

        #[test]
        fn boolean() {
            assert_eq!(parse("true").unwrap(), Expression::Boolean(true));
            assert_eq!(parse("false").unwrap(), Expression::Boolean(false));
        }

        #[test]
        fn multidigit_number() {
            assert_eq!(parse("10").unwrap(), Expression::Number(10))
        }

        #[test]
        fn identifier() {
            assert_eq!(
                parse("abc").unwrap(),
                Expression::Identifier("abc".to_string())
            )
        }

        #[test]
        fn onechar_identifier() {
            assert_eq!(parse("a").unwrap(), Expression::Identifier("a".to_string()))
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
                    parse($x).unwrap(),
                    Expression::BinOp(
                        $op,
                        Box::new(Expression::Number(0)),
                        Box::new(Expression::Number(0))
                    )
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
                parse("a[0][0]").unwrap(),
                Expression::BinOp(
                    Operator::Index,
                    Box::new(Expression::BinOp(
                        Operator::Index,
                        Box::new(Expression::Identifier("a".to_owned())),
                        Box::new(Expression::Number(0))
                    )),
                    Box::new(Expression::Number(0))
                )
            )
        }

        #[test]
        fn apply() {
            assert_eq!(
                parse("a(0)").unwrap(),
                Expression::Apply(
                    Box::new(Expression::Identifier("a".to_owned())),
                    Box::new(Expression::Number(0))
                )
            )
        }

        #[test]
        fn apply_spaced() {
            assert_eq!(
                parse("a (0)").unwrap(),
                Expression::Apply(
                    Box::new(Expression::Identifier("a".to_owned())),
                    Box::new(Expression::Number(0))
                )
            )
        }

        #[test]
        fn apply_with_outer_bracket() {
            assert_eq!(
                parse("( a(0) )").unwrap(),
                Expression::Apply(
                    Box::new(Expression::Identifier("a".to_owned())),
                    Box::new(Expression::Number(0))
                )
            )
        }

        #[test]
        fn apply_multi() {
            assert_eq!(
                parse("a(0)(0)").unwrap(),
                Expression::Apply(
                    Box::new(Expression::Apply(
                        Box::new(Expression::Identifier("a".to_owned())),
                        Box::new(Expression::Number(0))
                    )),
                    Box::new(Expression::Number(0))
                )
            )
        }

        #[test]
        fn assign() {
            assert_eq!(
                parse("a=0").unwrap(),
                Expression::Assign(
                    Box::new(Expression::Identifier("a".to_owned())),
                    Box::new(Expression::Number(0))
                )
            )
        }

        #[test]
        fn follow() {
            assert_eq!(
                parse("0;0").unwrap(),
                Expression::Follow(
                    Box::new(Expression::Number(0)),
                    Box::new(Expression::Number(0))
                )
            )
        }

        #[test]
        fn ifelse() {
            assert_eq!(
                parse("if true 0 else 0").unwrap(),
                Expression::IfElse(
                    Box::new(Expression::Boolean(true)),
                    Box::new(Expression::Number(0)),
                    Box::new(Expression::Number(0))
                )
            )
        }

        #[test]
        fn precedence_arith() {
            assert_eq!(
                parse("0+0*0").unwrap(),
                Expression::BinOp(
                    Operator::Add,
                    Box::new(Expression::Number(0)),
                    Box::new(Expression::BinOp(
                        Operator::Mul,
                        Box::new(Expression::Number(0)),
                        Box::new(Expression::Number(0))
                    ))
                )
            )
        }

        #[test]
        fn precedence_arith_bracket() {
            assert_eq!(
                parse("(0+0)*0").unwrap(),
                Expression::BinOp(
                    Operator::Mul,
                    Box::new(Expression::BinOp(
                        Operator::Add,
                        Box::new(Expression::Number(0)),
                        Box::new(Expression::Number(0))
                    )),
                    Box::new(Expression::Number(0))
                )
            )
        }

        #[test]
        fn precedence_assign_follow() {
            assert_eq!(
                parse("a=0;0").unwrap(),
                Expression::Follow(
                    Box::new(Expression::Assign(
                        Box::new(Expression::Identifier("a".to_owned())),
                        Box::new(Expression::Number(0))
                    )),
                    Box::new(Expression::Number(0))
                )
            )
        }

        #[test]
        fn precedence_ifelse_follow() {
            assert_eq!(
                parse("if true 0 else 0;0").unwrap(),
                Expression::Follow(
                    Box::new(Expression::IfElse(
                        Box::new(Expression::Boolean(true)),
                        Box::new(Expression::Number(0)),
                        Box::new(Expression::Number(0))
                    )),
                    Box::new(Expression::Number(0))
                )
            )
        }

        #[test]
        fn precedence_function_follow() {
            assert_eq!(
                parse("a -> a; 0").unwrap(),
                Expression::Follow(
                    Box::new(Expression::Function(
                        "a".to_string(),
                        Box::new(Expression::Identifier("a".to_string()))
                    )),
                    Box::new(Expression::Number(0))
                )
            )
        }
    }

    mod sugar {
        use super::parse;

        #[test]
        fn function_params() {
            assert_eq!(parse("(a,b,c)->0").unwrap(), parse("a->b->c->0").unwrap())
        }

        #[test]
        fn application() {
            assert_eq!(parse("f(a,b,c)").unwrap(), parse("f(a)(b)(c)").unwrap())
        }
    }
}
