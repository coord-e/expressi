use cranelift::codegen::ir::{InstBuilder, types};

pub enum CondCode {
    Equal,
    NotEqual,
    LessThan,
    GreaterThanOrEqual,
    GreaterThan,
    LessThanOrEqual
}

struct Builder<T: InstBuilder> {
    inst_builder: T,
    variable_map: HashMap<String, u32>
};

impl<T> Builder<T> {
    pub fn inst_builder(&self) -> T {
        self.inst_builder
    }

    pub fn constant<T>(&self, t: types::Type, v: T) -> Option<Value> {
        Some(Value::new(match t {
            types::I64 => self.inst_builder.ins().iconst(t, i64::from(v)),
            types::B1  => self.inst_builder.ins().bconst(t, v),
            _ => return None
        }, t))
    }

    pub fn add(&self, lhs: Value, rhs: Value) -> Value {
        let res = self.inst_builder.ins().iadd(lhs.cl_value(), rhs.cl_value());
        Value::new(res, types::I64)
    }

    pub fn sub(&self, lhs: Value, rhs: Value) -> Value {
        let res = self.inst_builder.ins().isub(lhs.cl_value(), rhs.cl_value());
        Value::new(res, types::I64)
    }

    pub fn mul(&self, lhs: Value, rhs: Value) -> Value {
        let res = self.inst_builder.ins().imul(lhs.cl_value(), rhs.cl_value());
        Value::new(res, types::I64)
    }

    pub fn div(&self, lhs: Value, rhs: Value) -> Value {
        let res = self.inst_builder.ins().udiv(lhs.cl_value(), rhs.cl_value());
        Value::new(res, types::I64)
    }

    pub fn bit_and(&self, lhs: Value, rhs: Value) -> Value {
        let res = self.inst_builder.ins().band(lhs.cl_value(), rhs.cl_value());
        Value::new(res, types::I64)
    }

    pub fn bit_or(&self, lhs: Value, rhs: Value) -> Value {
        let res = self.inst_builder.ins().bor(lhs.cl_value(), rhs.cl_value());
        Value::new(res, types::I64)
    }

    pub fn bit_xor(&self, lhs: Value, rhs: Value) -> Value {
        let res = self.inst_builder.ins().bxor(lhs.cl_value(), rhs.cl_value());
        Value::new(res, types::I64)
    }

    pub fn cmp(&self, cmp_type: CondCode, lhs: Value, rhs: Value) -> Value {
        let cc = match cmp_type {
            CondCode::Equal => IntCC::Equal,
            CondCode::NotEqual => IntCC::NotEqual,
            CondCode::LessThan => IntCC::SignedLessThan,
            CondCode::GreaterThanOrEqual => IntCC::SignedGreaterThanOrEqual,
            CondCode::GreaterThan => IntCC::SignedGreaterThan,
            CondCode::LessThanOrEqual => IntCC::SignedLessThanOrEqual
        };

        let res = self.inst_builder.ins().icmp(cc, lhs.cl_value(), rhs.cl_value());
        Value::new(res, types::I64)
    }
}
