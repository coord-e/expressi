use expression::{Expression, Operator};

use std::collections::HashMap;

use cranelift::codegen::ir::InstBuilderBase;
use cranelift::prelude::*;
use cranelift_module::Module;
use cranelift_simplejit::SimpleJITBackend;

/// A collection of state used for translating from toy-language AST nodes
/// into Cranelift IR.
pub struct FunctionTranslator<'a> {
    pub builder: FunctionBuilder<'a, Variable>,
    pub variables: HashMap<String, Variable>,
    pub module: &'a mut Module<SimpleJITBackend>,
}

impl<'a> FunctionTranslator<'a> {
    /// When you write out instructions in Cranelift, you get back `Value`s. You
    /// can then use these references in other instructions.
    pub fn translate_expr(&mut self, expr: Expression) -> Value {
        match expr {
            Expression::Number(number) => self.builder.ins().iconst(types::I64, i64::from(number)),

            Expression::Boolean(tf) => self.builder.ins().bconst(types::B1, tf),

            Expression::BinOp(op, lhs, rhs) => {
                let lhs = self.translate_expr(*lhs);
                let rhs = self.translate_expr(*rhs);
                match op {
                    Operator::Add => self.builder.ins().iadd(lhs, rhs),
                    Operator::Sub => self.builder.ins().isub(lhs, rhs),
                    Operator::Mul => self.builder.ins().imul(lhs, rhs),
                    Operator::Div => self.builder.ins().udiv(lhs, rhs),
                    Operator::BitAnd => self.builder.ins().iadd(lhs, rhs),
                    Operator::BitXor => self.builder.ins().bxor(lhs, rhs),
                    Operator::BitOr => self.builder.ins().bor(lhs, rhs),
                    Operator::Lt => self.builder.ins().icmp(IntCC::SignedLessThan, lhs, rhs),
                    Operator::Gt => self.builder.ins().icmp(IntCC::SignedGreaterThan, lhs, rhs),
                    Operator::Le => self
                        .builder
                        .ins()
                        .icmp(IntCC::SignedLessThanOrEqual, lhs, rhs),
                    Operator::Ge => {
                        self.builder
                            .ins()
                            .icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs)
                    }
                    Operator::Eq => self.builder.ins().icmp(IntCC::Equal, lhs, rhs),
                    Operator::Ne => self.builder.ins().icmp(IntCC::NotEqual, lhs, rhs),
                }
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
                let variable = if self.variables.contains_key(&name) {
                    *self.variables.get(&name).unwrap()
                } else {
                    let variable = Variable::new(self.variables.len());
                    self.variables.insert(name.into(), variable);
                    let new_type = self.builder.ins().data_flow_graph().value_type(new_value);
                    self.builder.declare_var(variable, new_type);
                    variable
                };
                self.builder.def_var(variable, new_value);
                new_value
            }

            Expression::Identifier(name) => {
                let variable = self.variables.get(&name).expect("variable not defined");
                self.builder.use_var(*variable)
            }

            Expression::IfElse(cond, then_expr, else_expr) => {
                let condition_value = self.translate_expr(*cond);

                let else_block = self.builder.create_ebb();
                let merge_block = self.builder.create_ebb();

                // Test the confition
                self.builder.ins().brz(condition_value, else_block, &[]);

                let then_return = self.translate_expr(*then_expr);

                let then_return_type = self.builder.ins().data_flow_graph().value_type(then_return);
                self.builder.append_ebb_param(merge_block, then_return_type);

                // Jump to merge block after translation of the 'then' block
                self.builder.ins().jump(merge_block, &[then_return]);

                // Start writing 'else' block
                self.builder.switch_to_block(else_block);
                self.builder.seal_block(else_block);

                let else_return = self.translate_expr(*else_expr);
                let else_return_type = self.builder.ins().data_flow_graph().value_type(else_return);
                if then_return_type != else_return_type {
                    panic!("Using different type value in if-else")
                }

                // Jump to merge block after translation of the 'then' block
                self.builder.ins().jump(merge_block, &[else_return]);

                self.builder.switch_to_block(merge_block);
                self.builder.seal_block(merge_block);

                // Get returned value and return it
                self.builder.ebb_params(merge_block)[0]
            }
        }
    }
}
