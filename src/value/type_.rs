use error::{
    LLVMTypeConversionError, InternalTypeConversionError,
};

use inkwell::types::{BasicTypeEnum, IntType};

use std::fmt;
use std::ptr::NonNull;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypeID(usize);

pub type EnumTypeData = Vec<(String, Vec<TypeID>)>;

#[derive(Debug)]
pub enum TypeData {
    Number,
    Boolean,
    Array(NonNull<TypeData>, usize),
    Empty,
    Enum(EnumTypeData)
}

unsafe impl Send for Type {}
unsafe impl Sync for Type {}

impl TypeData {
    pub fn from_cl(t: BasicTypeEnum) -> Result<Self, LLVMTypeConversionError> {
        Ok(match t {
            BasicTypeEnum::IntType(int) => match int.get_bit_width() {
                1  => TypeData::Boolean,
                64 => TypeData::Number,
                _  => unimplemented!()
            },
            _ => return Err(LLVMTypeConversionError { from: format!("{:?}", t) }),
        })
    }

    pub fn cl_type(&self) -> Result<BasicTypeEnum, InternalTypeConversionError> {
        Ok(match self {
            TypeData::Number => IntType::i64_type(),
            TypeData::Boolean => IntType::bool_type(),
            _ => return Err(InternalTypeConversionError { from: *self }),
        }.into())
    }

    pub fn size(&self) -> usize {
        match self {
            TypeData::Number => 8,
            TypeData::Boolean => 1,
            TypeData::Array(t, length) => unsafe {*t.as_ptr()}.size() * length,
            TypeData::Empty => 0
        }
    }
}

impl fmt::Display for TypeData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rep: String = match self {
            TypeData::Number => "Number".to_string(),
            TypeData::Boolean => "Boolean".to_string(),
            TypeData::Array(t, length) => format!("[{}; {}]", unsafe {*t.as_ptr()}, length),
            TypeData::Empty => "Empty".to_string(),
        };

        write!(f, "{}", rep)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeParseError;

impl FromStr for TypeData {
    type Err = TypeParseError;

    fn from_str(x: &str) -> Result<Self, Self::Err> {
        Ok(match x {
            "Number" => TypeData::Number,
            "Boolean" => TypeData::Boolean,
            "Empty" => TypeData::Empty,
            _ => return Err(TypeParseError),
        })
    }
}

pub struct TypeStore {
    types: HashMap<TypeID, TypeData>
}

impl TypeStore {
    pub fn new() -> Self {
        let mut store = Self { types: HashMap::new() };
        store.new_type(TypeData::Number);
        store
    }

    fn new_type(&mut self, data: TypeData) -> TypeID {
        let id = TypeID(self.types.len());
        self.types.insert(id, data);
        id
    }

    pub fn new_enum(&mut self, data: EnumTypeData) -> TypeID {
        self.new_type(TypeData::Enum(data))
    }
}
