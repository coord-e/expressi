pub enum Constant {
    Number(i64),
    Boolean(bool),
    Empty
}

pub enum Value {
    Bind(Identifier, Box<Value>),
    Assign(Box<Value>, Box<Value>),
    Constant(Constant)
}


