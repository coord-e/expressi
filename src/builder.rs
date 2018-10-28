use error::{InvalidCastError, TypeError, ReleasedValueError, InvalidContextBranchError};
use expression::Operator;
use value::{ValueID, ValueStore, ValueData, TypeStore, TypeID, ValueManager};
use value::manager::PrimitiveKind;
use scope::{Scope, ScopeStack};

use failure::Error;

use inkwell::{basic_block,builder,module,types,values,IntPredicate};

use std::rc::Rc;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CondCode {
    Equal,
    NotEqual,
    LessThan,
    GreaterThanOrEqual,
    GreaterThan,
    LessThanOrEqual,
}

pub struct Block {
    ebb: basic_block::BasicBlock,
}

impl Block {
    pub fn cl_ebb(&self) -> &basic_block::BasicBlock {
        &self.ebb
    }
}

pub struct Builder<'a> {
    inst_builder: &'a mut builder::Builder,
    module: Rc<module::Module>,
    manager: Rc<ValueManager>,
    scope_stack: ScopeStack
}

impl<'a> Builder<'a> {
    pub fn new(inst_builder: &'a mut builder::Builder, module: Rc<module::Module>) -> Self {
        let manager = Rc::new(ValueManager::new());
        Builder {
            inst_builder,
            module,
            manager,
            scope_stack: ScopeStack::new(manager.clone())
        }
    }

    pub fn inst_builder<'short>(&'short mut self) -> &'short mut builder::Builder {
        self.inst_builder
    }

    pub fn scope_stack<'short>(&'short mut self) -> &'short mut ScopeStack {
        &mut self.scope_stack
    }

    pub fn number_constant(&mut self, v: i64) -> Result<ValueID, Error> {
        let t = types::IntType::i64_type();
        self.manager.new_value_from_llvm(values::BasicValueEnum::IntValue(t.const_int(v.abs() as u64, v < 0)), t)
    }

    pub fn boolean_constant(&mut self, v: bool) -> Result<ValueID, Error> {
        let t = types::IntType::bool_type();
        self.manager.new_value_from_llvm(values::BasicValueEnum::IntValue(t.const_int(v as u64, false)), t)
    }

    pub fn apply_op(&mut self, op: Operator, lhs: ValueID, rhs: ValueID) -> Result<ValueID, Error> {
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
            Operator::Index => self.index(lhs, rhs),
        }
    }

    fn check_numeric_args(&self, lhs: ValueID, rhs: ValueID) -> Result<(), Error> {
        let number_type = self.manager.primitive_type(PrimitiveKind::Number);
        if self.manager.type_of(lhs) != number_type || self.manager.type_of(rhs) != number_type {
            return Err(TypeError.into());
        }
        Ok(())
    }

    pub fn add(&mut self, lhs: ValueID, rhs: ValueID) -> Result<ValueID, Error> {
        self.check_numeric_args(lhs, rhs)?;
        let lhs_cl = self.manager.llvm_value(lhs)?;
        let rhs_cl = self.manager.llvm_value(lhs)?;
        let res = self
            .inst_builder
            .build_int_add(lhs_cl.into_int_value(), rhs_cl.into_int_value(), "add");
        self.manager.new_value_from_llvm(res, types::IntType::i64_type())
    }

    pub fn sub(&mut self, lhs: ValueID, rhs: ValueID) -> Result<ValueID, Error> {
        self.check_numeric_args(lhs, rhs)?;
        let lhs_cl = self.manager.llvm_value(lhs)?;
        let rhs_cl = self.manager.llvm_value(rhs)?;
        let res = self
            .inst_builder
            .build_int_sub(lhs_cl.into_int_value(), rhs_cl.into_int_value(), "sub");
        self.manager.new_value_from_llvm(res, types::IntType::i64_type())
    }

    pub fn mul(&mut self, lhs: ValueID, rhs: ValueID) -> Result<ValueID, Error> {
        self.check_numeric_args(lhs, rhs)?;
        let lhs_cl = self.manager.llvm_value(lhs)?;
        let rhs_cl = self.manager.llvm_value(rhs)?;
        let res = self
            .inst_builder
            .build_int_mul(lhs_cl.into_int_value(), rhs_cl.into_int_value(), "mul");
        self.manager.new_value_from_llvm(res, types::IntType::i64_type())
    }

    pub fn div(&mut self, lhs: ValueID, rhs: ValueID) -> Result<ValueID, Error> {
        self.check_numeric_args(lhs, rhs)?;
        let lhs_cl = self.manager.llvm_value(lhs)?;
        let rhs_cl = self.manager.llvm_value(rhs)?;
        let res = self
            .inst_builder
            .build_int_unsigned_div(lhs_cl.into_int_value(), rhs_cl.into_int_value(), "div");
        self.manager.new_value_from_llvm(res, types::IntType::i64_type())
    }

    pub fn bit_and(&mut self, lhs: ValueID, rhs: ValueID) -> Result<ValueID, Error> {
        self.check_numeric_args(lhs, rhs)?;
        let lhs_cl = self.manager.llvm_value(lhs)?;
        let rhs_cl = self.manager.llvm_value(rhs)?;
        let res = self
            .inst_builder
            .build_and(lhs_cl.into_int_value(), rhs_cl.into_int_value(), "and");
        self.manager.new_value_from_llvm(res, types::IntType::i64_type())
    }

    pub fn bit_or(&mut self, lhs: ValueID, rhs: ValueID) -> Result<ValueID, Error> {
        self.check_numeric_args(lhs, rhs)?;
        let lhs_cl = self.manager.llvm_value(lhs)?;
        let rhs_cl = self.manager.llvm_value(rhs)?;
        let res = self
            .inst_builder
            .build_or(lhs_cl.into_int_value(), rhs_cl.into_int_value(), "or");
        self.manager.new_value_from_llvm(res, types::IntType::i64_type())
    }

    pub fn bit_xor(&mut self, lhs: ValueID, rhs: ValueID) -> Result<ValueID, Error> {
        self.check_numeric_args(lhs, rhs)?;
        let lhs_cl = self.manager.llvm_value(lhs)?;
        let rhs_cl = self.manager.llvm_value(rhs)?;
        let res = self
            .inst_builder
            .build_xor(lhs_cl.into_int_value(), rhs_cl.into_int_value(), "xor");
        self.manager.new_value_from_llvm(res, types::IntType::i64_type())
    }

    pub fn cmp(&mut self, cmp_type: CondCode, lhs: ValueID, rhs: ValueID) -> Result<ValueID, Error> {
        self.check_numeric_args(lhs, rhs)?;
        let cc = match cmp_type {
            CondCode::Equal              => IntPredicate::EQ,
            CondCode::NotEqual           => IntPredicate::NE,
            CondCode::LessThan           => IntPredicate::SLT,
            CondCode::GreaterThanOrEqual => IntPredicate::SGE,
            CondCode::GreaterThan        => IntPredicate::SGT,
            CondCode::LessThanOrEqual    => IntPredicate::SLE,
        };

        let lhs_cl = self.manager.llvm_value(lhs)?;
        let rhs_cl = self.manager.llvm_value(rhs)?;
        let res = self
            .inst_builder
            .build_int_compare(cc, lhs_cl.into_int_value(), rhs_cl.into_int_value(), "cmp");
        self.manager.new_value_from_llvm(res, types::IntType::bool_type())
    }

    pub fn index(&mut self, lhs: ValueID, rhs: ValueID) -> Result<ValueID, Error> {
        unimplemented!()
    }

    pub fn declare_var(&mut self, name: &str, t: Type, unique: bool) -> Result<String, Error> {
        let real_name = if unique { self.scope_stack.unique_name(name) } else { name.to_string() };
        let variable = self.inst_builder.build_alloca(t.cl_type()?, &real_name);
        let empty = self.value_store.new_value(ValueData::Empty);
        self.scope_stack.add(&real_name, empty, variable); // TODO: TypeValue
        Ok(real_name)
    }

    pub fn set_var(&mut self, name: &str, val: ValueID) -> Result<ValueID, Error> {
        let variable = self.scope_stack.get_var(name).ok_or(()).or_else(|_| -> Result<values::PointerValue, Error> {
            let variable = self.inst_builder.build_alloca(val.get_type().cl_type()?, name);
            self.scope_stack.add(name, val, variable);
            Ok(variable)
        })?;
        if let Ok(val) = self.manager.llvm_value(val) {
            self.inst_builder.build_store(variable, val);
        }
        self.scope_stack.set(name, val);
        Ok(val)
    }

    pub fn get_var(&mut self, name: &str) -> Option<ValueID> {
        self.scope_stack.get_var(name).map(|var| {
            let value = self.scope_stack.get(name).unwrap();
            let data = ValueData::primitive(self.inst_builder.build_load(var, "load_var"), value.get_type());
            self.value_store.new_value(data)
        })
    }

    pub fn cast_to(&mut self, v: ValueID, to_type: TypeID) -> Result<ValueID, Error> {
        let from_type = self.manager.type_of(v);
        if from_type == to_type {
            return Err(InvalidCastError {
                from: from_type,
                to: to_type,
            }.into());
        }

        let bool_type = self.manager.primitive_type(PrimitiveKind::Boolean);
        let number_type = self.manager.primitive_type(PrimitiveKind::Number);

        Ok(match (from_type, to_type) {
            (number_type, bool_type) => {
                let zero = self.number_constant(0)?;
                self.cmp(CondCode::NotEqual, v, zero)?
            }
            (bool_type, number_type) => {
                let cl = self.manager.llvm_value(v)?;
                let to_llvm_type = self.manager.llvm_type(to_type)?;
                self.manager.new_value_from_llvm(self.inst_builder.build_int_cast(cl.into_int_value(), to_llvm_type.into_int_type(), "b2i"), t)
            },
            _ => {
                return Err(InvalidCastError {
                    from: from_type,
                    to: to_type,
                }.into())
            }
        })
    }

    pub fn enter_new_scope(&mut self) {
        let scope = self.scope_stack.new_scope();
        self.enter_scope(scope);
    }

    pub fn enter_scope(&mut self, sc: Scope) {
        self.scope_stack.push(sc);
    }

    pub fn exit_scope(&mut self) -> Result<Scope, Error> {
        self.scope_stack.pop()
    }

    pub fn array_alloc(&mut self, t: TypeID, size: u32) -> Result<values::PointerValue, Error> {
        unimplemented!()
    }

    pub fn store(&mut self, v: ValueID, addr: values::PointerValue, offset: u32) -> Result<(), Error> {
        // TODO: Safety check
        let ptr = unsafe { self.inst_builder.build_in_bounds_gep(addr, &[types::IntType::i32_type().const_int(offset as u64, false)], "store") };
        let cl = self.manager.llvm_value(v)?;
        self.inst_builder.build_store(ptr, cl);
        Ok(())
    }

    pub fn load(&mut self, t: TypeID, addr: values::PointerValue, offset: u32) -> Result<ValueID, Error> {
        // TODO: Safety check
        let ptr = unsafe { self.inst_builder.build_in_bounds_gep(addr, &[types::IntType::i32_type().const_int(offset as u64, false)], "store") };
        let llvm_type = self.manager.llvm_type(t)?;
        self.manager.new_value_from_llvm(self.inst_builder.build_load(ptr, "load"), llvm_type)
    }

    pub fn create_block(&mut self) -> Result<Block, Error> {
        let parent = self.inst_builder.get_insert_block().and_then(|b| b.get_parent()).ok_or(InvalidContextBranchError)?;
        let block = self.module.get_context().append_basic_block(&parent, "");
        Ok(Block { ebb: block })
    }

    pub fn brz(&mut self, condition: ValueID, then_block: &Block, else_block: &Block) -> Result<(), Error> {
        let bool_type = self.manager.primitive_type(PrimitiveKind::Boolean);
        if self.manager.type_of(condition) != bool_type {
            return Err(TypeError.into());
        }
        let cl = self.manager.llvm_value(condition)?;
        self.inst_builder
            .build_conditional_branch(cl.into_int_value(), then_block.cl_ebb(), else_block.cl_ebb());
        Ok(())
    }

    pub fn jump(&mut self, block: &Block) {
        self.inst_builder.build_unconditional_branch(block.cl_ebb());
    }

    pub fn switch_to_block(&mut self, block: &Block) {
        self.inst_builder.position_at_end(block.cl_ebb());
    }

    pub fn current_block(&self) -> Result<Block, Error> {
        self.inst_builder.get_insert_block()
            .ok_or(InvalidContextBranchError.into())
            .map(|ebb| Block { ebb })
    }
}
