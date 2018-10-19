use expression::Expression;

use builder::Builder;
use error::{UndeclaredVariableError, TypeError};
use value::{Value, ValueData};
use scope::Scope;

use failure::Error;

pub struct FunctionTranslator<'a> {
    pub builder: Builder<'a>,
}

impl<'a> FunctionTranslator<'a> {
    pub fn translate_expr(&mut self, expr: Expression) -> Result<Value, Error> {
        Ok(match expr {
            Expression::Number(number) => self.builder.number_constant(i64::from(number))?,

            Expression::Boolean(tf) => self.builder.boolean_constant(tf)?,

            Expression::Empty => self.builder.value_store().new_value(ValueData::Empty),

            Expression::Array(expr) => {
                let elements = expr.into_iter().map(|expr| self.translate_expr(*expr)).collect::<Result<Vec<_>, _>>()?;
                let item_type = elements.last().unwrap().get_type();
                let addr = self.builder.array_alloc(item_type, elements.len() as u32)?;
                if elements.iter().any(|v| v.get_type() != item_type) {
                    return Err(TypeError.into());
                }
                for (idx, val) in elements.iter().enumerate() {
                    self.builder.store(*val, addr, (item_type.size() * idx) as i32)?;
                }
                self.builder.value_store().new_value(ValueData::array(addr, elements, item_type))
            }

            Expression::BinOp(op, lhs, rhs) => {
                let lhs = self.translate_expr(*lhs)?;
                let rhs = self.translate_expr(*rhs)?;
                self.builder.apply_op(op, lhs, rhs)?
            }

            Expression::Follow(lhs, rhs) => {
                self.translate_expr(*lhs)?;
                self.translate_expr(*rhs)?
            }

            Expression::Assign(lhs, rhs) => {
                let new_value = self.translate_expr(*rhs)?;
                let name = match *lhs {
                    Expression::Identifier(name) => name,
                    _ => panic!("Non-identifier identifier"),
                };
                self.builder.set_var(&name, new_value)?;
                new_value
            }

            Expression::Identifier(name) => {
                self.builder.get_var(&name).ok_or(UndeclaredVariableError)?
            }

            Expression::Cast(lhs, ty) => {
                let lhs = self.translate_expr(*lhs)?;
                self.builder.cast_to(lhs, ty)?
            }

            Expression::Scope(expr) => {
                self.builder.enter_scope(Scope::new());
                let content = self.translate_expr(*expr)?;
                self.builder.exit_scope()?;
                content
            }

            Expression::IfElse(cond, then_expr, else_expr) => {
                let condition_value = self.translate_expr(*cond)?;

                let then_block = self.builder.create_block()?;
                let else_block = self.builder.create_block()?;
                let merge_block = self.builder.create_block()?;
                self.builder.brz(condition_value, &then_block, &else_block)?;

                let initial_block = self.builder.current_block();

                self.builder.switch_to_block(&then_block);
                let then_return = self.translate_expr(*then_expr)?;
                self.builder.jump(&merge_block);

                self.builder.switch_to_block(&initial_block);
                let var_name = self.builder.declare_var("__cond", then_return.get_type(), true)?;

                self.builder.switch_to_block(&then_block);
                self.builder.set_var(&var_name, then_return);

                // Start writing 'else' block
                self.builder.switch_to_block(&else_block);
                let else_return = self.translate_expr(*else_expr)?;
                if then_return.get_type() != else_return.get_type() {
                    panic!("Using different type value in if-else")
                }
                self.builder.set_var(&var_name, else_return);

                // Jump to merge block after translation of the 'then' block
                self.builder.jump(&merge_block);

                self.builder.switch_to_block(&merge_block);
                self.builder.get_var(&var_name).unwrap()
            }
        })
    }
}
