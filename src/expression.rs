#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Unknown,
}

impl Operator {
    pub fn from_str(x: &str) -> Operator {
        match x {
            "+" => Operator::Add,
            "-" => Operator::Sub,
            "*" => Operator::Mul,
            "/" => Operator::Div,
            _ => Operator::Unknown,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Expression {
    Number(i64),
    Identifier(String),
    Assign(Box<Expression>, Box<Expression>),
    Follow(Box<Expression>, Box<Expression>),
    BinOp(Operator, Box<Expression>, Box<Expression>),
}
