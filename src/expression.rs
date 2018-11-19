use builder::BindingKind;
use std::str::FromStr;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    BitAnd,
    BitXor,
    BitOr,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
    Index,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperatorParseError;

impl FromStr for Operator {
    type Err = OperatorParseError;

    fn from_str(x: &str) -> Result<Self, Self::Err> {
        Ok(match x {
            "+" => Operator::Add,
            "-" => Operator::Sub,
            "*" => Operator::Mul,
            "/" => Operator::Div,
            "&" => Operator::BitAnd,
            "^" => Operator::BitXor,
            "|" => Operator::BitOr,
            "<" => Operator::Lt,
            ">" => Operator::Gt,
            "<=" => Operator::Le,
            ">=" => Operator::Ge,
            "==" => Operator::Eq,
            "!=" => Operator::Ne,
            _ => return Err(OperatorParseError),
        })
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Expression {
    Number(i64),
    Boolean(bool),
    Array(Vec<Box<Expression>>),
    Function(String, Box<Expression>),
    Identifier(String),
    TypeIdentifier(String),
    Empty,
    Assign(Box<Expression>, Box<Expression>),
    Bind(BindingKind, String, Box<Expression>),
    Follow(Box<Expression>, Box<Expression>),
    BinOp(Operator, Box<Expression>, Box<Expression>),
    Apply(Box<Expression>, Box<Expression>),
    IfElse(Box<Expression>, Box<Expression>, Box<Expression>),
    Cast(Box<Expression>, Box<Expression>),
    Scope(Box<Expression>),
    Type(Vec<(Expression, Vec<Expression>)>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str() {
        assert_eq!("+".parse::<Operator>().unwrap(), Operator::Add)
    }

    #[test]
    fn from_str_err() {
        assert!("a".parse::<Operator>().is_err())
    }
}
