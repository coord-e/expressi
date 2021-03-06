use super::atom::Atom;
use super::BoundPointer;
use crate::error::TranslationError;
use crate::expression::Operator;
use crate::ir::{BindingKind, Type};
use crate::scope::{Env, Scope, ScopedEnv};

use failure::Error;

use inkwell::types::BasicType;
use inkwell::{basic_block, builder, module, types, values, AddressSpace, IntPredicate};

use std::collections::{BTreeMap, HashMap};
use std::mem;

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

pub struct Builder {
    inst_builder: builder::Builder,
    module: module::Module,
    env: ScopedEnv<BoundPointer>,
}

impl Builder {
    pub fn new(inst_builder: builder::Builder, module: module::Module) -> Self {
        Builder {
            inst_builder,
            module,
            env: ScopedEnv::new(),
        }
    }

    pub fn inst_builder(&mut self) -> &mut builder::Builder {
        &mut self.inst_builder
    }

    pub fn env(&mut self) -> &mut ScopedEnv<BoundPointer> {
        &mut self.env
    }

    pub fn take_module(self) -> module::Module {
        self.module
    }

    pub fn type_of(&self, v: values::BasicValueEnum) -> types::BasicTypeEnum {
        match v {
            values::BasicValueEnum::ArrayValue(v) => v.get_type().into(),
            values::BasicValueEnum::IntValue(v) => v.get_type().into(),
            values::BasicValueEnum::FloatValue(v) => v.get_type().into(),
            values::BasicValueEnum::PointerValue(v) => v.get_type().into(),
            values::BasicValueEnum::StructValue(v) => v.get_type().into(),
            values::BasicValueEnum::VectorValue(v) => v.get_type().into(),
        }
    }

    pub fn llvm_type(&self, ty: &Type) -> Result<types::BasicTypeEnum, Error> {
        Ok(match ty {
            Type::Number => types::IntType::i64_type().into(),
            Type::Boolean => types::IntType::bool_type().into(),
            Type::Empty => types::VoidType::void_type()
                .ptr_type(AddressSpace::Generic)
                .into(),
            Type::Variable(_) => return Err(TranslationError::UnresolvedType.into()),
            Type::Function(box param, box body) => {
                let void_ptr = types::VoidType::void_type().ptr_type(AddressSpace::Generic);
                let param = self.llvm_type(param)?;
                let ret = self.llvm_type(body)?;

                let fn_type = ret
                    .fn_type(&[void_ptr.into(), param], false)
                    .ptr_type(AddressSpace::Generic);
                types::StructType::struct_type(&[void_ptr.into(), fn_type.into()], false).into()
            }
        })
    }

    pub fn number_constant(&mut self, v: i64) -> Result<values::BasicValueEnum, Error> {
        let t = types::IntType::i64_type();
        Ok(values::BasicValueEnum::IntValue(
            t.const_int(v.abs() as u64, v < 0),
        ))
    }

    pub fn boolean_constant(&mut self, v: bool) -> Result<values::BasicValueEnum, Error> {
        let t = types::IntType::bool_type();
        Ok(values::BasicValueEnum::IntValue(
            t.const_int(v as u64, false),
        ))
    }

    pub fn empty_constant(&self) -> Result<values::BasicValueEnum, Error> {
        let t = types::VoidType::void_type().ptr_type(AddressSpace::Generic);
        Ok(values::BasicValueEnum::PointerValue(t.const_null()))
    }

    pub fn function_constant<F>(
        &mut self,
        ty: &Type,
        param_name: String,
        capture_list: &HashMap<String, Type>,
        eval: F,
    ) -> Result<values::BasicValueEnum, Error>
    where
        F: FnOnce(&mut Self) -> Result<values::BasicValueEnum, Error>,
    {
        let previous_block = self.inst_builder().get_insert_block().unwrap();
        self.enter_new_scope();

        // TODO: Remove this insufficient copy
        let capture_list: BTreeMap<_, _> = capture_list.iter().collect();

        let capture_types: Vec<types::BasicTypeEnum> = capture_list
            .iter()
            .map(|(_, ty)| self.llvm_type(ty))
            .collect::<Result<_, _>>()?;
        let capture_type = types::StructType::struct_type(&capture_types, false);

        let fn_concrete_type = self.llvm_type(ty)?.into_struct_type();

        let fn_type = fn_concrete_type
            .get_field_type_at_index(1)
            .unwrap()
            .into_pointer_type()
            .get_element_type()
            .into_function_type();

        let function = self.module.add_function("", fn_type, None);
        let basic_block = self
            .module
            .get_context()
            .append_basic_block(&function, "entry");
        self.inst_builder.position_at_end(&basic_block);
        let arg_ptr = self
            .inst_builder
            .build_alloca(fn_type.get_param_types()[1], "");
        self.inst_builder
            .build_store(arg_ptr, function.get_nth_param(1).unwrap());
        self.env.insert(
            &param_name,
            BoundPointer::new(BindingKind::Immutable, arg_ptr.into()),
        );

        let capture_arg = self.inst_builder.build_pointer_cast(
            function.get_nth_param(0).unwrap().into_pointer_value(),
            capture_type.ptr_type(AddressSpace::Generic),
            "",
        );
        for (i, (name, _)) in capture_list.iter().enumerate() {
            let ptr = unsafe {
                self.inst_builder
                    .build_struct_gep(capture_arg, i as u32, "")
            };
            self.env
                .insert(&name, BoundPointer::new(BindingKind::Immutable, ptr.into()));
        }

        let ret = eval(self)?;

        self.exit_scope()?;
        self.inst_builder().build_return(Some(&ret));
        self.inst_builder().position_at_end(&previous_block);

        // TODO: Fix memory leak
        let capture_ptr = self
            .inst_builder
            .build_malloc(capture_type, "eval_capture_ptr");
        for (i, (name, _)) in capture_list.iter().enumerate() {
            let ptr = unsafe {
                self.inst_builder
                    .build_struct_gep(capture_ptr, i as u32, "")
            };
            let var_ptr = self
                .env
                .get(&name)
                .unwrap()
                .ptr_value()
                .clone()
                .expect_value()?;
            self.inst_builder
                .build_store(ptr, self.inst_builder.build_load(var_ptr, ""));
        }

        let void_ptr_ty = types::VoidType::void_type().ptr_type(AddressSpace::Generic);
        let ret_type = types::StructType::struct_type(
            &[
                void_ptr_ty.into(),
                fn_type.ptr_type(AddressSpace::Generic).into(),
            ],
            false,
        );

        let capture_ptr_erased =
            self.inst_builder
                .build_pointer_cast(capture_ptr, void_ptr_ty, "capture_ptr_erase");
        let ptr: values::PointerValue = unsafe { mem::transmute(function) };

        // TODO: Fix this dangerous implementation
        let real_ret = self
            .inst_builder
            .build_insert_value(ret_type.get_undef(), capture_ptr_erased, 0, "")
            .unwrap();
        let real_ret: values::StructValue = unsafe { mem::transmute(real_ret) };
        let real_ret = self
            .inst_builder
            .build_insert_value(real_ret, ptr, 1, "")
            .unwrap();
        let real_ret: values::StructValue = unsafe { mem::transmute(real_ret) };

        Ok(real_ret.into())
    }

    pub fn call(
        &self,
        func: values::BasicValueEnum,
        arg: values::BasicValueEnum,
    ) -> Result<values::BasicValueEnum, Error> {
        let func = func.into_struct_value();
        let capture_ptr = self
            .inst_builder
            .build_extract_value(func, 0, "capture_ptr")
            .unwrap()
            .into_pointer_value();
        let func_ptr = self
            .inst_builder
            .build_extract_value(func, 1, "func")
            .unwrap()
            .into_pointer_value();
        let func_v: values::FunctionValue = unsafe { mem::transmute(func_ptr) };
        let call_inst = self
            .inst_builder
            .build_call(func_v, &[capture_ptr.into(), arg], "");
        Ok(call_inst.try_as_basic_value().left().unwrap())
    }

    pub fn apply_op(
        &mut self,
        op: Operator,
        lhs: values::BasicValueEnum,
        rhs: values::BasicValueEnum,
    ) -> Result<values::BasicValueEnum, Error> {
        let lhs_int = lhs.into_int_value();
        let rhs_int = rhs.into_int_value();
        Ok(match op {
            Operator::Add => self.inst_builder.build_int_add(lhs_int, rhs_int, "add"),
            Operator::Sub => self.inst_builder.build_int_sub(lhs_int, rhs_int, "sub"),
            Operator::Mul => self.inst_builder.build_int_mul(lhs_int, rhs_int, "mul"),
            Operator::Div => self
                .inst_builder
                .build_int_unsigned_div(lhs_int, rhs_int, "div"),
            Operator::BitAnd => self.inst_builder.build_and(lhs_int, rhs_int, "add"),
            Operator::BitXor => self.inst_builder.build_xor(lhs_int, rhs_int, "xor"),
            Operator::BitOr => self.inst_builder.build_or(lhs_int, rhs_int, "or"),
            Operator::Lt => self.cmp(CondCode::LessThan, lhs_int, rhs_int),
            Operator::Gt => self.cmp(CondCode::GreaterThan, lhs_int, rhs_int),
            Operator::Le => self.cmp(CondCode::LessThanOrEqual, lhs_int, rhs_int),
            Operator::Ge => self.cmp(CondCode::GreaterThanOrEqual, lhs_int, rhs_int),
            Operator::Eq => self.cmp(CondCode::Equal, lhs_int, rhs_int),
            Operator::Ne => self.cmp(CondCode::NotEqual, lhs_int, rhs_int),
            Operator::Index => self.index(lhs, rhs),
        }
        .into())
    }

    pub fn cmp(
        &mut self,
        cmp_type: CondCode,
        lhs: values::IntValue,
        rhs: values::IntValue,
    ) -> values::IntValue {
        let cc = match cmp_type {
            CondCode::Equal => IntPredicate::EQ,
            CondCode::NotEqual => IntPredicate::NE,
            CondCode::LessThan => IntPredicate::SLT,
            CondCode::GreaterThanOrEqual => IntPredicate::SGE,
            CondCode::GreaterThan => IntPredicate::SGT,
            CondCode::LessThanOrEqual => IntPredicate::SLE,
        };

        self.inst_builder.build_int_compare(cc, lhs, rhs, "cmp")
    }

    pub fn index(
        &mut self,
        _lhs: values::BasicValueEnum,
        _rhs: values::BasicValueEnum,
    ) -> values::IntValue {
        unimplemented!()
    }

    pub(crate) fn declare_mut_var(
        &mut self,
        name: &str,
        base_value: &Atom<values::BasicValueEnum>,
        unique: bool,
    ) -> Result<String, Error> {
        let real_name = if unique {
            self.env.unique_name(name)
        } else {
            name.to_string()
        };
        let ptr = match base_value {
            Atom::LLVMValue(val) => {
                let t = self.type_of(*val);
                self.inst_builder.build_alloca(t, &real_name).into()
            }
            Atom::PolyValue(val_table) => val_table
                .iter()
                .map(|(k, v)| {
                    let t = self.type_of(*v);
                    (k.clone(), self.inst_builder.build_alloca(t, &real_name))
                })
                .collect::<HashMap<_, _>>()
                .into(),
        };
        self.env
            .insert(&real_name, BoundPointer::new(BindingKind::Mutable, ptr));
        Ok(real_name)
    }

    pub fn bind_var(
        &mut self,
        name: &str,
        val: &Atom<values::BasicValueEnum>,
        kind: BindingKind,
    ) -> Result<(), Error> {
        let ptr = match val {
            Atom::LLVMValue(val) => self.store_mono_var(name, *val).into(),
            Atom::PolyValue(val_table) => val_table
                .iter()
                .map(|(k, v)| (k.clone(), self.store_mono_var(name, *v)))
                .collect::<HashMap<_, _>>()
                .into(),
        };
        self.env.insert(name, BoundPointer::new(kind, ptr));
        Ok(())
    }

    fn store_mono_var(&mut self, name: &str, val: values::BasicValueEnum) -> values::PointerValue {
        let llvm_type = self.type_of(val);
        let variable = self.inst_builder.build_alloca(llvm_type, name);
        self.inst_builder.build_store(variable, val);
        variable
    }

    pub fn assign_var(
        &mut self,
        name: &str,
        val: &Atom<values::BasicValueEnum>,
    ) -> Result<(), Error> {
        let var = self
            .env
            .get(name)
            .ok_or(TranslationError::UndeclaredVariable)?;

        if var.kind() != BindingKind::Mutable {
            return Err(TranslationError::ImmutableAssign.into());
        }

        match var.ptr_value() {
            Atom::LLVMValue(var) => {
                self.inst_builder
                    .build_store(*var, val.clone().expect_value()?);
            }
            Atom::PolyValue(var_table) => {
                var_table
                    .iter()
                    .map(|(k, v)| {
                        self.inst_builder
                            .build_store(*v, val.clone().expect_poly_value()?[k]);
                        Ok(())
                    })
                    .collect::<Result<(), Error>>()?;
            }
        };
        Ok(())
    }

    pub fn get_var(&mut self, name: &str) -> Result<Option<Atom<values::BasicValueEnum>>, Error> {
        self.env.get(name).map_or(Ok(None), |var| {
            Ok(Some(match var.ptr_value() {
                Atom::LLVMValue(var) => self.inst_builder.build_load(var.clone(), name).into(),
                Atom::PolyValue(var_table) => var_table
                    .iter()
                    .map(|(k, v)| (k.clone(), self.inst_builder.build_load(v.clone(), name)))
                    .collect::<HashMap<_, _>>()
                    .into(),
            }))
        })
    }

    pub fn cast_to(
        &mut self,
        v: values::BasicValueEnum,
        to_type: types::BasicTypeEnum,
    ) -> Result<values::BasicValueEnum, Error> {
        let from_type = self.type_of(v);
        if from_type == to_type {
            return Err(TranslationError::InvalidCast {
                from: format!("{:?}", from_type),
                to: format!("{:?}", to_type),
            }
            .into());
        }

        let number_type: types::BasicTypeEnum = types::IntType::i64_type().into();
        let bool_type: types::BasicTypeEnum = types::IntType::bool_type().into();

        // TODO: more elegant way to match types
        if from_type == number_type {
            if to_type == bool_type {
                let zero = self.number_constant(0)?;
                return Ok(self
                    .cmp(
                        CondCode::NotEqual,
                        v.into_int_value(),
                        zero.into_int_value(),
                    )
                    .into());
            }
        } else if from_type == bool_type && to_type == number_type {
            return Ok(self
                .inst_builder
                .build_int_z_extend(v.into_int_value(), to_type.into_int_type(), "b2i")
                .into());
        }
        Err(TranslationError::InvalidCast {
            from: format!("{:?}", from_type),
            to: format!("{:?}", to_type),
        }
        .into())
    }

    pub fn enter_new_scope(&mut self) {
        let scope = self.env.new_scope();
        self.enter_scope(scope);
    }

    pub fn enter_scope(&mut self, sc: Env<BoundPointer>) {
        self.env.push(sc);
    }

    pub fn exit_scope(&mut self) -> Result<Env<BoundPointer>, Error> {
        self.env.pop()
    }

    pub fn array_alloc(
        &mut self,
        _t: types::BasicTypeEnum,
        _size: u32,
    ) -> Result<values::PointerValue, Error> {
        unimplemented!()
    }

    pub fn store(
        &mut self,
        _v: values::BasicValueEnum,
        _addr: values::PointerValue,
        _offset: u32,
    ) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn load(
        &mut self,
        _t: &Type,
        _addr: values::PointerValue,
        _offset: u32,
    ) -> Result<values::BasicValueEnum, Error> {
        unimplemented!()
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
        condition: values::BasicValueEnum,
        then_block: &Block,
        else_block: &Block,
    ) -> Result<(), Error> {
        let bool_type = types::IntType::bool_type();
        if self.type_of(condition) != bool_type.into() {
            return Err(TranslationError::InvalidType.into());
        }
        self.inst_builder.build_conditional_branch(
            condition.into_int_value(),
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
            .ok_or_else(|| TranslationError::InvalidContextBranch.into())
            .map(|ebb| Block { ebb })
    }

    pub fn ret_int(&mut self, v: values::BasicValueEnum) -> Result<(), Error> {
        // TODO: Generic return
        let number_type: types::BasicTypeEnum = types::IntType::i64_type().into();
        let return_value: values::BasicValueEnum = if self.type_of(v) != number_type {
            self.cast_to(v, number_type)?
        } else {
            v
        };
        // Emit the return instruction.
        self.inst_builder.build_return(Some(&return_value));
        Ok(())
    }
}
