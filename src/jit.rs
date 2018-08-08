use parser;
use expression::{Expression, Operator};

use std::collections::HashMap;

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
        let ast = parser::parse(&input)
            .map_err(|e| e.to_string())?;

        // Translate the AST nodes into Cranelift IR.
        self.translate(ast)
            .map_err(|e| e.to_string())?;

        let id = self.module
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
    fn translate(
        &mut self,
        expr: Expression,
    ) -> Result<(), String> {
        let int = self.module.pointer_type();
        self.ctx.func.signature.returns.push(AbiParam::new(int));

        let mut builder =
            FunctionBuilder::<Variable>::new(&mut self.ctx.func, &mut self.builder_context);

        let entry_ebb = builder.create_ebb();

        builder.append_ebb_params_for_function_params(entry_ebb);

        builder.switch_to_block(entry_ebb);
        builder.seal_block(entry_ebb);

        let mut trans = FunctionTranslator {
            int,
            builder,
            variables: HashMap::new(),
            module: &mut self.module,
        };
        let return_value = trans.translate_expr(expr);
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
    int: types::Type,
    builder: FunctionBuilder<'a, Variable>,
    variables: HashMap<String, Variable>,
    module: &'a mut Module<SimpleJITBackend>,
}

impl<'a> FunctionTranslator<'a> {
    /// When you write out instructions in Cranelift, you get back `Value`s. You
    /// can then use these references in other instructions.
    fn translate_expr(&mut self, expr: Expression) -> Value {
        match expr {
            Expression::Number(number) => {
                self.builder.ins().iconst(self.int, i64::from(number))
            }

            Expression::BinOp(op, lhs, rhs) => {
                let lhs = self.translate_expr(*lhs);
                let rhs = self.translate_expr(*rhs);
                match op {
                    Operator::Add => self.builder.ins().iadd(lhs, rhs),
                    Operator::Sub => self.builder.ins().isub(lhs, rhs),
                    Operator::Mul => self.builder.ins().imul(lhs, rhs),
                    Operator::Div => self.builder.ins().udiv(lhs, rhs),
                    Operator::Unknown => lhs,
                }
            }

            Expression::Follow(lhs, rhs) => {
                self.translate_expr(*lhs);
                let rhs = self.translate_expr(*rhs);
                rhs
            }

            Expression::Assign(lhs, rhs) => {
                let new_value = self.translate_expr(*rhs);
                let name = match *lhs {
                    Expression::Identifier(name) => name,
                    _ => {
                        panic!("Non-identifier identifier")
                    }
                };
                let variable = if self.variables.contains_key(&name) {
                    *self.variables.get(&name).unwrap()
                } else {
                    let variable = Variable::new(self.variables.len());
                    self.variables.insert(name.into(), variable);
                    self.builder.declare_var(variable, self.module.pointer_type());
                    variable
                };
                self.builder.def_var(variable, new_value);
                new_value
            }

            Expression::Identifier(name) => {
                let variable = self.variables.get(&name).expect("variable not defined");
                self.builder.use_var(*variable)
            }
        }
    }
}

