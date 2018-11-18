use error::TranslationError;
use expression::Operator;
use scope::{BindingKind, Scope, ScopeStack};
use value::manager::PrimitiveKind;
use value::type_::EnumTypeData;
use value::{TypeID, TypeStore, ValueData, ValueID, ValueManager, ValueManagerRef, ValueStore};

use failure::Error;

use inkwell::{basic_block, builder, module, types, values, IntPredicate};

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
    manager: ValueManagerRef,
    scope_stack: ScopeStack,
}

impl<'a> Builder<'a> {
    pub fn new(
        manager: ValueManagerRef,
        inst_builder: &'a mut builder::Builder,
        module: Rc<module::Module>,
    ) -> Self {
        let mut scope_stack = ScopeStack::new(manager.clone());

        scope_stack.bind(
            "Number",
            manager
                .borrow()
                .primitive_type(PrimitiveKind::Number)
                .into(),
            BindingKind::Immutable,
        );
        scope_stack.bind(
            "Boolean",
            manager
                .borrow()
                .primitive_type(PrimitiveKind::Boolean)
                .into(),
            BindingKind::Immutable,
        );
        scope_stack.bind(
            "Empty",
            manager.borrow().primitive_type(PrimitiveKind::Empty).into(),
            BindingKind::Immutable,
        );

        Builder {
            inst_builder,
            module,
            scope_stack,
            manager,
        }
    }

    pub fn inst_builder<'short>(&'short mut self) -> &'short mut builder::Builder {
        self.inst_builder
    }

    pub fn scope_stack<'short>(&'short mut self) -> &'short mut ScopeStack {
        &mut self.scope_stack
    }

    pub fn type_of(&self, v: ValueID) -> Result<TypeID, Error> {
        self.manager.try_borrow()?.type_of(v)
    }

    pub fn number_constant(&mut self, v: i64) -> Result<ValueID, Error> {
        let t = types::IntType::i64_type();
        self.manager.try_borrow_mut()?.new_value_from_llvm(
            values::BasicValueEnum::IntValue(t.const_int(v.abs() as u64, v < 0)),
            t,
        )
    }

    pub fn boolean_constant(&mut self, v: bool) -> Result<ValueID, Error> {
        let t = types::IntType::bool_type();
        self.manager.try_borrow_mut()?.new_value_from_llvm(
            values::BasicValueEnum::IntValue(t.const_int(v as u64, false)),
            t,
        )
    }

    pub fn empty_constant(&self) -> Result<ValueID, Error> {
        self.manager
            .try_borrow()
            .map_err(Into::into)
            .map(|manager| manager.empty_value())
    }

    pub fn register_type(&mut self, data: EnumTypeData) -> Result<TypeID, Error> {
        self.manager
            .try_borrow_mut()
            .map_err(Into::into)
            .map(|mut manager| manager.new_user_type(data))
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
        let manager = self.manager.try_borrow()?;
        let number_type = manager.primitive_type(PrimitiveKind::Number);
        if manager.type_of(lhs)? != number_type || manager.type_of(rhs)? != number_type {
            return Err(TranslationError::InvalidType.into());
        }
        Ok(())
    }

    pub fn add(&mut self, lhs: ValueID, rhs: ValueID) -> Result<ValueID, Error> {
        self.check_numeric_args(lhs, rhs)?;
        let lhs_cl = self.manager.try_borrow()?.llvm_value(lhs)?;
        let rhs_cl = self.manager.try_borrow()?.llvm_value(rhs)?;
        let res = self.inst_builder.build_int_add(
            lhs_cl.into_int_value(),
            rhs_cl.into_int_value(),
            "add",
        );
        self.manager
            .try_borrow_mut()?
            .new_value_from_llvm(res, types::IntType::i64_type())
    }

    pub fn sub(&mut self, lhs: ValueID, rhs: ValueID) -> Result<ValueID, Error> {
        self.check_numeric_args(lhs, rhs)?;
        let lhs_cl = self.manager.try_borrow()?.llvm_value(lhs)?;
        let rhs_cl = self.manager.try_borrow()?.llvm_value(rhs)?;
        let res = self.inst_builder.build_int_sub(
            lhs_cl.into_int_value(),
            rhs_cl.into_int_value(),
            "sub",
        );
        self.manager
            .try_borrow_mut()?
            .new_value_from_llvm(res, types::IntType::i64_type())
    }

    pub fn mul(&mut self, lhs: ValueID, rhs: ValueID) -> Result<ValueID, Error> {
        self.check_numeric_args(lhs, rhs)?;
        let lhs_cl = self.manager.try_borrow()?.llvm_value(lhs)?;
        let rhs_cl = self.manager.try_borrow()?.llvm_value(rhs)?;
        let res = self.inst_builder.build_int_mul(
            lhs_cl.into_int_value(),
            rhs_cl.into_int_value(),
            "mul",
        );
        self.manager
            .try_borrow_mut()?
            .new_value_from_llvm(res, types::IntType::i64_type())
    }

    pub fn div(&mut self, lhs: ValueID, rhs: ValueID) -> Result<ValueID, Error> {
        self.check_numeric_args(lhs, rhs)?;
        let lhs_cl = self.manager.try_borrow()?.llvm_value(lhs)?;
        let rhs_cl = self.manager.try_borrow()?.llvm_value(rhs)?;
        let res = self.inst_builder.build_int_unsigned_div(
            lhs_cl.into_int_value(),
            rhs_cl.into_int_value(),
            "div",
        );
        self.manager
            .try_borrow_mut()?
            .new_value_from_llvm(res, types::IntType::i64_type())
    }

    pub fn bit_and(&mut self, lhs: ValueID, rhs: ValueID) -> Result<ValueID, Error> {
        self.check_numeric_args(lhs, rhs)?;
        let lhs_cl = self.manager.try_borrow()?.llvm_value(lhs)?;
        let rhs_cl = self.manager.try_borrow()?.llvm_value(rhs)?;
        let res =
            self.inst_builder
                .build_and(lhs_cl.into_int_value(), rhs_cl.into_int_value(), "and");
        self.manager
            .try_borrow_mut()?
            .new_value_from_llvm(res, types::IntType::i64_type())
    }

    pub fn bit_or(&mut self, lhs: ValueID, rhs: ValueID) -> Result<ValueID, Error> {
        self.check_numeric_args(lhs, rhs)?;
        let lhs_cl = self.manager.try_borrow()?.llvm_value(lhs)?;
        let rhs_cl = self.manager.try_borrow()?.llvm_value(rhs)?;
        let res =
            self.inst_builder
                .build_or(lhs_cl.into_int_value(), rhs_cl.into_int_value(), "or");
        self.manager
            .try_borrow_mut()?
            .new_value_from_llvm(res, types::IntType::i64_type())
    }

    pub fn bit_xor(&mut self, lhs: ValueID, rhs: ValueID) -> Result<ValueID, Error> {
        self.check_numeric_args(lhs, rhs)?;
        let lhs_cl = self.manager.try_borrow()?.llvm_value(lhs)?;
        let rhs_cl = self.manager.try_borrow()?.llvm_value(rhs)?;
        let res =
            self.inst_builder
                .build_xor(lhs_cl.into_int_value(), rhs_cl.into_int_value(), "xor");
        self.manager
            .try_borrow_mut()?
            .new_value_from_llvm(res, types::IntType::i64_type())
    }

    pub fn cmp(
        &mut self,
        cmp_type: CondCode,
        lhs: ValueID,
        rhs: ValueID,
    ) -> Result<ValueID, Error> {
        self.check_numeric_args(lhs, rhs)?;
        let cc = match cmp_type {
            CondCode::Equal => IntPredicate::EQ,
            CondCode::NotEqual => IntPredicate::NE,
            CondCode::LessThan => IntPredicate::SLT,
            CondCode::GreaterThanOrEqual => IntPredicate::SGE,
            CondCode::GreaterThan => IntPredicate::SGT,
            CondCode::LessThanOrEqual => IntPredicate::SLE,
        };

        let lhs_cl = self.manager.try_borrow()?.llvm_value(lhs)?;
        let rhs_cl = self.manager.try_borrow()?.llvm_value(rhs)?;
        let res = self.inst_builder.build_int_compare(
            cc,
            lhs_cl.into_int_value(),
            rhs_cl.into_int_value(),
            "cmp",
        );
        self.manager
            .try_borrow_mut()?
            .new_value_from_llvm(res, types::IntType::bool_type())
    }

    pub fn index(&mut self, lhs: ValueID, rhs: ValueID) -> Result<ValueID, Error> {
        unimplemented!()
    }

    pub fn declare_mut_var(
        &mut self,
        name: &str,
        t: TypeID,
        unique: bool,
    ) -> Result<String, Error> {
        let manager = self.manager.try_borrow()?;
        let real_name = if unique {
            self.scope_stack.unique_name(name)
        } else {
            name.to_string()
        };
        let llvm_type = manager.llvm_type(t)?;
        let variable = self.inst_builder.build_alloca(llvm_type, &real_name);
        let empty = manager.empty_value();
        self.scope_stack.add_var(&real_name, variable); // TODO: TypeValue
        self.scope_stack
            .bind(&real_name, empty.into(), BindingKind::Mutable);
        Ok(real_name)
    }

    pub fn bind_var(
        &mut self,
        name: &str,
        val: ValueID,
        kind: BindingKind,
    ) -> Result<ValueID, Error> {
        let manager = self.manager.try_borrow()?;
        let t = manager.type_of(val)?;
        let llvm_type = manager.llvm_type(t)?;
        let variable = self.inst_builder.build_alloca(llvm_type, name);
        self.scope_stack.add_var(name, variable);

        if let Ok(val) = manager.llvm_value(val) {
            self.inst_builder.build_store(variable, val);
        }
        self.scope_stack.bind(name, val.into(), kind);
        Ok(val)
    }

    pub fn assign_var(&mut self, name: &str, val: ValueID) -> Result<ValueID, Error> {
        let var = self
            .scope_stack
            .get_var(name)
            .ok_or(TranslationError::UndeclaredVariable)?;
        if let Ok(val) = self.manager.try_borrow()?.llvm_value(val) {
            self.inst_builder.build_store(var, val);
        }
        self.scope_stack.assign(name, val.into())?;
        Ok(val)
    }

    pub fn get_var(&mut self, name: &str) -> Result<Option<ValueID>, Error> {
        self.scope_stack.get_var(name).map_or(Ok(None), |var| {
            let value = self.scope_stack.get(name).unwrap().expect_value()?;
            let t = self.manager.try_borrow()?.type_of(value)?;
            let llvm_type = self.manager.try_borrow()?.llvm_type(t)?;
            let loaded = self.inst_builder.build_load(var, "load_var");
            self.manager
                .try_borrow_mut()?
                .new_value_from_llvm(loaded, llvm_type)
                .map(Some)
        })
    }

    pub fn cast_to(&mut self, v: ValueID, to_type: TypeID) -> Result<ValueID, Error> {
        let from_type = self.manager.try_borrow()?.type_of(v)?;
        if from_type == to_type {
            return Err(TranslationError::InvalidCast {
                from: from_type,
                to: to_type,
            }.into());
        }

        let bool_type = self
            .manager
            .try_borrow()?
            .primitive_type(PrimitiveKind::Boolean);
        let number_type = self
            .manager
            .try_borrow()?
            .primitive_type(PrimitiveKind::Number);

        // TODO: more elegant way to match types
        if from_type == number_type {
            if to_type == bool_type {
                let zero = self.number_constant(0)?;
                return self.cmp(CondCode::NotEqual, v, zero);
            }
        } else if from_type == bool_type {
            if to_type == number_type {
                let cl = self.manager.try_borrow()?.llvm_value(v)?;
                let to_llvm_type = self.manager.try_borrow()?.llvm_type(to_type)?;
                return self.manager.try_borrow_mut()?.new_value_from_llvm(
                    self.inst_builder.build_int_z_extend(
                        cl.into_int_value(),
                        to_llvm_type.into_int_type(),
                        "b2i",
                    ),
                    to_llvm_type,
                );
            }
        }
        Err(TranslationError::InvalidCast {
            from: from_type,
            to: to_type,
        }.into())
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

    pub fn store(
        &mut self,
        v: ValueID,
        addr: values::PointerValue,
        offset: u32,
    ) -> Result<(), Error> {
        // TODO: Safety check
        let ptr = unsafe {
            self.inst_builder.build_in_bounds_gep(
                addr,
                &[types::IntType::i32_type().const_int(offset as u64, false)],
                "store",
            )
        };
        let cl = self.manager.try_borrow()?.llvm_value(v)?;
        self.inst_builder.build_store(ptr, cl);
        Ok(())
    }

    pub fn load(
        &mut self,
        t: TypeID,
        addr: values::PointerValue,
        offset: u32,
    ) -> Result<ValueID, Error> {
        // TODO: Safety check
        let ptr = unsafe {
            self.inst_builder.build_in_bounds_gep(
                addr,
                &[types::IntType::i32_type().const_int(offset as u64, false)],
                "store",
            )
        };
        let llvm_type = self.manager.try_borrow()?.llvm_type(t)?;
        self.manager
            .try_borrow_mut()?
            .new_value_from_llvm(self.inst_builder.build_load(ptr, "load"), llvm_type)
    }

    pub fn create_block(&mut self) -> Result<Block, Error> {
        let parent = self
            .inst_builder
            .get_insert_block()
            .and_then(|b| b.get_parent())
            .ok_or(TranslationError::InvalidContextBranch)?;
        let block = self.module.get_context().append_basic_block(&parent, "");
        Ok(Block { ebb: block })
    }

    pub fn brz(
        &mut self,
        condition: ValueID,
        then_block: &Block,
        else_block: &Block,
    ) -> Result<(), Error> {
        let manager = self.manager.try_borrow()?;
        let bool_type = manager.primitive_type(PrimitiveKind::Boolean);
        if manager.type_of(condition)? != bool_type {
            return Err(TranslationError::InvalidType.into());
        }
        let cl = manager.llvm_value(condition)?;
        self.inst_builder.build_conditional_branch(
            cl.into_int_value(),
            then_block.cl_ebb(),
            else_block.cl_ebb(),
        );
        Ok(())
    }

    pub fn jump(&mut self, block: &Block) {
        self.inst_builder.build_unconditional_branch(block.cl_ebb());
    }

    pub fn switch_to_block(&mut self, block: &Block) {
        self.inst_builder.position_at_end(block.cl_ebb());
    }

    pub fn current_block(&self) -> Result<Block, Error> {
        self.inst_builder
            .get_insert_block()
            .ok_or(TranslationError::InvalidContextBranch.into())
            .map(|ebb| Block { ebb })
    }

    pub fn ret_int(&mut self, v: ValueID) -> Result<(), Error> {
        // TODO: Generic return
        let number_type = self
            .manager
            .try_borrow()?
            .primitive_type(PrimitiveKind::Number);
        let return_value = if self.manager.try_borrow()?.type_of(v)? != number_type {
            self.cast_to(v, number_type)?
        } else {
            v
        };
        // Emit the return instruction.
        let cl = self
            .manager
            .try_borrow()?
            .llvm_value(return_value)?
            .into_int_value();
        self.inst_builder.build_return(Some(&cl));
        Ok(())
    }
}
