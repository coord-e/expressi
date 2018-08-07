#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Expression {
    Number(i64),
    BinOp(String, Box<Expression>, Box<Expression>),
}
