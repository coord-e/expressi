use expression::Expression;

use builder::Builder;
use error::{UndeclaredVariableError, UndeclaredTypeError, TypeError};
use value::{ValueID, ValueData, Atom};
use value::type_::{EnumTypeData, TypeID};
use scope::Scope;

use failure::Error;

pub struct FunctionTranslator<'a> {
    pub builder: Builder<'a>,
}

impl<'a> FunctionTranslator<'a> {
    pub fn translate_expr(&mut self, expr: Expression) -> Result<Atom, Error> {
        Ok(match expr {
            Expression::Number(number) => self.builder.number_constant(i64::from(number))?.into(),

            Expression::Boolean(tf) => self.builder.boolean_constant(tf)?.into(),

            Expression::Empty => self.builder.empty_constant().into(),

            Expression::Array(expr) => {
                unimplemented!()
            }

            Expression::Type(expr) => {
                let typedata = expr.into_iter().map(|(ident, params)| Ok((
                        match ident {
                            Expression::TypeIdentifier(id) => id,
                            _ => unreachable!()
                        },
                        params.into_iter().map(|t| self.translate_expr(t).and_then(|e| e.expect_type())).collect::<Result<Vec<TypeID>, _>>()?
                    ))).collect::<Result<EnumTypeData, Error>>()?;
                self.builder.register_type(typedata).into()
            }

            Expression::BinOp(op, lhs, rhs) => {
                let lhs = self.translate_expr(*lhs)?.expect_value()?;
                let rhs = self.translate_expr(*rhs)?.expect_value()?;
                self.builder.apply_op(op, lhs, rhs)?.into()
            }

            Expression::Follow(lhs, rhs) => {
                self.translate_expr(*lhs)?;
                self.translate_expr(*rhs)?
            }

            Expression::Assign(lhs, rhs) => {
                let new_value = self.translate_expr(*rhs)?.expect_value()?;
                let name = match *lhs {
                    Expression::Identifier(name) => name,
                    _ => panic!("Non-identifier identifier"),
                };
                self.builder.set_var(&name, new_value)?;
                new_value.into()
            }

            Expression::TypeIdentifier(id) => self.builder.scope_stack().resolve_type(&id).ok_or(UndeclaredTypeError)?.into(),

            Expression::Identifier(name) => {
                self.builder.get_var(&name).and_then(|v| v.ok_or(UndeclaredVariableError.into()))?.into()
            }

            Expression::Cast(lhs, rhs) => {
                let lhs = self.translate_expr(*lhs)?.expect_value()?;
                let rhs = self.translate_expr(*rhs)?.expect_type()?;
                self.builder.cast_to(lhs, rhs)?.into()
            }

            Expression::Scope(expr) => {
                self.builder.enter_new_scope();
                let content = self.translate_expr(*expr)?.expect_value()?;
                self.builder.exit_scope()?;
                content.into()
            }

            Expression::IfElse(cond, then_expr, else_expr) => {
                let condition_value = self.translate_expr(*cond)?.expect_value()?;

                let then_block = self.builder.create_block()?;
                let else_block = self.builder.create_block()?;
                let merge_block = self.builder.create_block()?;

                let initial_block = self.builder.current_block()?;

                self.builder.switch_to_block(&then_block);
                let then_return = self.translate_expr(*then_expr)?.expect_value()?;

                self.builder.switch_to_block(&initial_block);
                let then_type = self.builder.type_of(then_return)?;
                let var_name = self.builder.declare_var("__cond", then_type, true)?;
                self.builder.brz(condition_value, &then_block, &else_block)?;

                self.builder.switch_to_block(&then_block);
                self.builder.set_var(&var_name, then_return)?;
                self.builder.jump(&merge_block);

                // Start writing 'else' block
                self.builder.switch_to_block(&else_block);
                let else_return = self.translate_expr(*else_expr)?.expect_value()?;
                let else_type = self.builder.type_of(else_return)?;
                if then_type != else_type {
                    panic!("Using different type value in if-else")
                }
                self.builder.set_var(&var_name, else_return)?;

                // Jump to merge block after translation of the 'then' block
                self.builder.jump(&merge_block);

                self.builder.switch_to_block(&merge_block);
                self.builder.get_var(&var_name)?.unwrap().into()
            }
        })
    }
}
