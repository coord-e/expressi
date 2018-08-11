use expression::{Expression, Operator};

use builder::Builder;
use value::Value;

use cranelift::prelude::{FunctionBuilder, types, Variable};
use cranelift_module::Module;
use cranelift_simplejit::SimpleJITBackend;

/// A collection of state used for translating from toy-language AST nodes
/// into Cranelift IR.
pub struct FunctionTranslator<'a> {
    pub builder: Builder<FunctionBuilder<'a, Variable>>,
    pub module: &'a mut Module<SimpleJITBackend>,
}

impl<'a> FunctionTranslator<'a> {
    /// When you write out instructions in Cranelift, you get back `Value`s. You
    /// can then use these references in other instructions.
    pub fn translate_expr(&mut self, expr: Expression) -> Value {
        match expr {
            Expression::Number(number) => self.builder.constant(types::I64, number),

            Expression::Boolean(tf) => self.builder.constant(types::B1, tf),

            Expression::BinOp(op, lhs, rhs) => {
                let lhs = self.translate_expr(*lhs);
                let rhs = self.translate_expr(*rhs);
                self.builder.apply_op(op, lhs, rhs)
            }

            Expression::Follow(lhs, rhs) => {
                self.translate_expr(*lhs);
                self.translate_expr(*rhs)
            }

            Expression::Assign(lhs, rhs) => {
                let new_value = self.translate_expr(*rhs);
                let name = match *lhs {
                    Expression::Identifier(name) => name,
                    _ => panic!("Non-identifier identifier"),
                };
                self.builder.set_var(name, new_value);
                new_value
            }

            Expression::Identifier(name) => {
                self.builder.get_var(&name).expect("variable not defined");
            }

            Expression::IfElse(cond, then_expr, else_expr) => {
                let condition_value = self.translate_expr(*cond);

                let else_block = self.builder.create_block();
                let merge_block = self.builder.create_block();

                // Test the confition
                self.builder.brz(condition_value, else_block, &[]);

                let then_return = self.translate_expr(*then_expr);

                self.builder.set_block_signature(merge_block, &[then_return.get_type()]);

                // Jump to merge block after translation of the 'then' block
                self.builder.jump(merge_block, &[then_return]);

                // Start writing 'else' block
                self.builder.switch_to_block(else_block);

                let else_return = self.translate_expr(*else_expr);
                if then_return.get_type() != else_return.get_type() {
                    panic!("Using different type value in if-else")
                }

                // Jump to merge block after translation of the 'then' block
                self.builder.jump(merge_block, &[else_return]);

                self.builder.switch_to_block(merge_block);

                // Get returned value and return it
                self.builder.block_params(merge_block)[0]
            }
        }
    }
}
