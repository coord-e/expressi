use expression::Expression;

use builder::Builder;
use error::TranslationError;
use ir;
use scope::Scope;
use value::type_::{EnumTypeData, TypeID};
use value::{Atom, ValueData, ValueID};

use failure::Error;

pub struct EIRTranslator<'a> {
    pub builder: Builder<'a>,
}

impl<'a> EIRTranslator<'a> {
    pub fn translate_expr(&mut self, expr: ir::Value) -> Result<Atom, Error> {
        Ok(match expr {
            ir::Value::Constant(c) => match c {
                ir::Constant::Number(number) => {
                    self.builder.number_constant(i64::from(number))?.into()
                }
                ir::Constant::Boolean(tf) => self.builder.boolean_constant(tf)?.into(),
                ir::Constant::Empty => self.builder.empty_constant()?.into(),
            },
            ir::Value::Function(_, _) => unimplemented!(),
            ir::Value::Apply(_, _) => unimplemented!(),
            ir::Value::BinOp(op, lhs, rhs) => {
                let lhs = self.translate_expr(*lhs)?.expect_value()?;
                let rhs = self.translate_expr(*rhs)?.expect_value()?;
                self.builder.apply_op(op, lhs, rhs)?.into()
            }

            ir::Value::Follow(lhs, rhs) => {
                self.translate_expr(*lhs)?;
                self.translate_expr(*rhs)?
            }

            ir::Value::Bind(kind, name, rhs) => {
                let new_value = self.translate_expr(*rhs)?.expect_value()?;
                self.builder.bind_var(&name, new_value, kind)?;
                new_value.into()
            }

            ir::Value::Assign(lhs, rhs) => {
                let new_value = self.translate_expr(*rhs)?.expect_value()?;
                let name = match *lhs {
                    ir::Value::Typed(_, box ir::Value::Variable(name)) => name,
                    _ => panic!("Non-variable identifier"),
                };
                self.builder.assign_var(&name, new_value)?;
                new_value.into()
            }

            ir::Value::Variable(name) => self
                .builder
                .get_var(&name)
                .and_then(|v| v.ok_or(TranslationError::UndeclaredVariable.into()))?
                .into(),

            ir::Value::Scope(expr) => {
                self.builder.enter_new_scope();
                let content = self.translate_expr(*expr)?.expect_value()?;
                self.builder.exit_scope()?;
                content.into()
            }

            ir::Value::IfElse(cond, then_expr, else_expr) => {
                let condition_value = self.translate_expr(*cond)?.expect_value()?;

                let then_block = self.builder.create_block()?;
                let else_block = self.builder.create_block()?;
                let merge_block = self.builder.create_block()?;

                let initial_block = self.builder.current_block()?;

                self.builder.switch_to_block(&then_block);
                let then_return = self.translate_expr(*then_expr)?.expect_value()?;

                self.builder.switch_to_block(&initial_block);
                let then_type = self.builder.type_of(then_return)?;
                let var_name = self.builder.declare_mut_var("__cond", then_type, true)?;
                self.builder
                    .brz(condition_value, &then_block, &else_block)?;

                self.builder.switch_to_block(&then_block);
                self.builder.assign_var(&var_name, then_return)?;
                self.builder.jump(&merge_block);

                // Start writing 'else' block
                self.builder.switch_to_block(&else_block);
                let else_return = self.translate_expr(*else_expr)?.expect_value()?;
                let else_type = self.builder.type_of(else_return)?;
                if then_type != else_type {
                    panic!("Using different type value in if-else")
                }
                self.builder.assign_var(&var_name, else_return)?;

                // Jump to merge block after translation of the 'then' block
                self.builder.jump(&merge_block);

                self.builder.switch_to_block(&merge_block);
                self.builder.get_var(&var_name)?.unwrap().into()
            }
            ir::Value::Typed(_, value) => self.translate_expr(*value)?,
        })
    }
}
