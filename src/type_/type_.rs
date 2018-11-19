use type_::TypeID;

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorKind {
    Number,
    Boolean,
    Empty,
    Function,
}

pub type EnumTypeData = Vec<(String, Vec<TypeID>)>;

#[derive(Debug, Clone)]
pub enum TypeData {
    Operator(OperatorKind, Vec<TypeID>),
    Enum(EnumTypeData),
    Variable(Option<TypeID>),
    PolyVariable(Vec<TypeID>),
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
