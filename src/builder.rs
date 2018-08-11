use cranelift::codegen::ir::{InstBuilder, types};

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
}
