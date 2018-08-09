#[derive(Clone, PartialEq, Eq, Debug)]
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
    Unknown,
}

impl Operator {
    pub fn from_str(x: &str) -> Operator {
        match x {
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
            _ => Operator::Unknown,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Expression {
    Number(i64),
    Boolean(bool),
    Identifier(String),
    Assign(Box<Expression>, Box<Expression>),
    Follow(Box<Expression>, Box<Expression>),
    BinOp(Operator, Box<Expression>, Box<Expression>),
}
