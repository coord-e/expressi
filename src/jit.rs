use expression::{Expression, Operator};
use parser;

use std::collections::HashMap;

use cranelift::codegen::ir::InstBuilderBase;
use cranelift::prelude::*;
use cranelift_module::{DataContext, Linkage, Module};
use cranelift_simplejit::{SimpleJITBackend, SimpleJITBuilder};

pub struct JIT {
    builder_context: FunctionBuilderContext<Variable>,
    ctx: codegen::Context,
    data_ctx: DataContext,
    module: Module<SimpleJITBackend>,
}

impl JIT {
    pub fn new() -> Self {
        // Windows calling conventions are not supported by cranelift yet.
        if cfg!(windows) {
            unimplemented!();
        }

        let builder = SimpleJITBuilder::new();
        let module = Module::new(builder);
        Self {
            builder_context: FunctionBuilderContext::<Variable>::new(),
            ctx: module.make_context(),
            data_ctx: DataContext::new(),
            module,
        }
    }

    /// Compile a string in the toy language into machine code.
    pub fn compile(&mut self, name: &str, input: &str) -> Result<*const u8, String> {
        // Parse the string, producing AST nodes.
        let ast = parser::parse(&input).map_err(|e| e.to_string())?;

        // Translate the AST nodes into Cranelift IR.
        self.translate(ast).map_err(|e| e.to_string())?;

        let id = self
            .module
            .declare_function(&name, Linkage::Export, &self.ctx.func.signature)
            .map_err(|e| e.to_string())?;

        self.module
            .define_function(id, &mut self.ctx)
            .map_err(|e| e.to_string())?;

        // Now that compilation is finished, we can clear out the context state.
        self.module.clear_context(&mut self.ctx);

        // Finalize the function, finishing any outstanding relocations.
        let code = self.module.finalize_function(id);

        Ok(code)
    }

    // Translate from toy-language AST nodes into Cranelift IR.
    fn translate(&mut self, expr: Expression) -> Result<(), String> {
        self.ctx.func.signature.returns.push(AbiParam::new(self.module.pointer_type()));

        let mut builder =
            FunctionBuilder::<Variable>::new(&mut self.ctx.func, &mut self.builder_context);

        let entry_ebb = builder.create_ebb();

        builder.append_ebb_params_for_function_params(entry_ebb);

        builder.switch_to_block(entry_ebb);
        builder.seal_block(entry_ebb);

        let mut trans = FunctionTranslator {
            builder,
            variables: HashMap::new(),
            module: &mut self.module,
        };
        let evaluated_value = trans.translate_expr(expr);
        let evaluated_type = trans
            .builder
            .ins()
            .data_flow_graph()
            .value_type(evaluated_value);
        let return_value = if evaluated_type != types::I64 {
            trans.builder.ins().bint(types::I64, evaluated_value)
        } else {
            evaluated_value
        };
        // Emit the return instruction.
        trans.builder.ins().return_(&[return_value]);

        // Tell the builder we're done with this function.
        trans.builder.finalize();
        Ok(())
    }
}

/// A collection of state used for translating from toy-language AST nodes
/// into Cranelift IR.
struct FunctionTranslator<'a> {
    builder: FunctionBuilder<'a, Variable>,
    variables: HashMap<String, Variable>,
    module: &'a mut Module<SimpleJITBackend>,
}

impl<'a> FunctionTranslator<'a> {
    /// When you write out instructions in Cranelift, you get back `Value`s. You
    /// can then use these references in other instructions.
    fn translate_expr(&mut self, expr: Expression) -> Value {
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
                    Operator::Unknown => lhs,
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
                if then_return_type != else_return_type { panic!("Using different type value in if-else") }

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
