use expression::Operator;
use scope::BindingKind;
use value::TypeID;

#[derive(Debug)]
pub enum Constant {
    Number(i64),
    Boolean(bool),
    Empty,
}

pub type Identifier = String;

#[derive(Debug)]
pub enum Value {
    Bind(BindingKind, Identifier, Box<Value>),
    Assign(Box<Value>, Box<Value>),
    Scope(Box<Value>),
    Follow(Box<Value>, Box<Value>),
    BinOp(Operator, Box<Value>, Box<Value>),
    IfElse(Box<Value>, Box<Value>, Box<Value>),
    Variable(Identifier),
    Constant(Constant),
    Typed(TypeID, Box<Value>),
}
