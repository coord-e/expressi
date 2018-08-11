use cranelift::codegen::ir::{InstBuilder, types, condcodes};

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

    pub fn apply_op(&self, op: Operator, lhs: Value, rhs: Value) => Value {
        match op {
            Operator::Add => self.add(lhs, rhs),
            Operator::Sub => self.sub(lhs, rhs),
            Operator::Mul => self.mul(lhs, rhs),
            Operator::Div => self.div(lhs, rhs),
            Operator::BitAnd => self.bit_and(lhs, rhs),
            Operator::BitXor => self.bit_xor(lhs, rhs),
            Operator::BitOr => self.bit_or(lhs, rhs),
            Operator::Lt => self.cmp(CondCode::LessThan, lhs, rhs),
            Operator::Gt => self.cmp(CondCode::GreaterThan, lhs, rhs),
            Operator::Le => self.cmp(CondCode::LessThanOrEqual, lhs, rhs),
            Operator::Ge => self.cmp(CondCode::GreaterThanOrEqual, lhs, rhs),
            Operator::Eq => self.cmp(CondCode::Equal, lhs, rhs),
            Operator::Ne => self.cmp(CondCode::NotEqual, lhs, rhs),
        }
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
