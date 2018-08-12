use expression::Expression;

use builder::Builder;
use error::UndeclaredVariableError;
use value::Value;

use failure::Error;

use cranelift_module::Module;
use cranelift_simplejit::SimpleJITBackend;

/// A collection of state used for translating from toy-language AST nodes
/// into Cranelift IR.
pub struct FunctionTranslator<'a> {
    pub builder: Builder<'a>,
    pub module: &'a mut Module<SimpleJITBackend>,
}

impl<'a> FunctionTranslator<'a> {
    /// When you write out instructions in Cranelift, you get back `Value`s. You
    /// can then use these references in other instructions.
    pub fn translate_expr(&mut self, expr: Expression) -> Result<Value, Error> {
        Ok(match expr {
            Expression::Number(number) => self.builder.number_constant(i64::from(number))?,

            Expression::Boolean(tf) => self.builder.boolean_constant(tf)?,

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

            Expression::IfElse(cond, then_expr, else_expr) => {
                let condition_value = self.translate_expr(*cond)?;

                let else_block = self.builder.create_block();
                let merge_block = self.builder.create_block();

                // Test the confition
                self.builder.brz(condition_value, else_block);

                let then_return = self.translate_expr(*then_expr)?;

                self.builder
                    .set_block_signature(merge_block, &[then_return.get_type()])?;

                // Jump to merge block after translation of the 'then' block
                self.builder.jump(merge_block, &[then_return]);

                // Start writing 'else' block
                self.builder.switch_to_block(else_block);

                let else_return = self.translate_expr(*else_expr)?;
                if then_return.get_type() != else_return.get_type() {
                    panic!("Using different type value in if-else")
                }

                // Jump to merge block after translation of the 'then' block
                self.builder.jump(merge_block, &[else_return]);

                self.builder.switch_to_block(merge_block);

                // Get returned value and return it
                self.builder.block_params(merge_block)[0]
            }
        })
    }
}
