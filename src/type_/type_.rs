use type_::TypeID;

use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct VariableID(usize);

impl fmt::Display for VariableID {
    pub(crate) fn with_usize(id: usize) -> Self {
        Self(id)
    }

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a{}", self.0);
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    Variable(VariableID),
    Number,
    Boolean,
    Function(Box<Type>, Box<Type>)
}

impl fmt::Display for TypeData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rep: String = match self {
            TypeData::Operator(kind, tys) => match kind {
                OperatorKind::Number => "Number".to_string(),
                OperatorKind::Boolean => "Boolean".to_string(),
                OperatorKind::Empty => "Empty".to_string(),
                OperatorKind::Function => format!("{:?} -> {:?}", tys[0], tys[1]),
            },
            TypeData::Enum(data) => format!("{:?}", data),
            TypeData::Variable(instance) => format!("var({:?})", instance),
            TypeData::PolyVariable(types) => format!("pvar({:?})", types),
        };

        write!(f, "{}", rep)
    }
}
