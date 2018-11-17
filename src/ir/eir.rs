use expression::Operator;

#[derive(Serialize, Deserialize, Debug)]
pub enum Constant {
    Number(i64),
    Boolean(bool),
    Empty
}

pub type Identifier = String;

#[derive(Serialize, Deserialize, Debug)]
pub enum Value {
    Bind(Identifier, Box<Value>),
    Assign(Box<Value>, Box<Value>),
    Scope(Box<Value>),
    Follow(Box<Value>, Box<Value>),
    BinOp(Operator, Box<Value>, Box<Value>),
    Variable(Identifier),
    Constant(Constant)
}


