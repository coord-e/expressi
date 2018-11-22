use type_::TypeID;

use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct VariableID(usize);

impl VariableID {
    pub(crate) fn with_usize(id: usize) -> Self {
        Self(id)
    }
}

impl fmt::Display for VariableID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a{}", self.0);
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    Variable(VariableID),
    Number,
    Boolean,
    Empty,
    Function(Box<Type>, Box<Type>)
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Variable(id) => write!(f, "{}", id),
            Type::Function(box t1, box t2) => write!(f, "({} -> {})", t1, t2),
            _ => write!(f, "{:?}", self)
        }
    }
}
