use parser;
use expression::{Expression, Operator};

use std::collections::HashMap;

use cranelift::prelude::*;
use cranelift_module::{DataContext, Linkage, Module};
use cranelift_simplejit::{SimpleJITBackend, SimpleJITBuilder};

/// The basic JIT class.
pub struct JIT {
    /// The function builder context, which is reused across multiple
    /// FunctionBuilder instances.
    builder_context: FunctionBuilderContext<Variable>,

    /// The main Cranelift context, which holds the state for codegen. Cranelift
    /// separates this from `Module` to allow for parallel compilation, with a
    /// context per thread, though this isn't in the simple demo here.
    ctx: codegen::Context,

    /// The data context, which is to data objects what `ctx` is to functions.
    data_ctx: DataContext,

    /// The module, with the simplejit backend, which manages the JIT'd
    /// functions.
    module: Module<SimpleJITBackend>,
}

impl JIT {
    /// Create a new `JIT` instance.
    pub fn new() -> Self {
        // Windows calling conventions are not supported yet.
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
        // First, parse the string, producing AST nodes.
        let ast = parser::parse(&input)
            .map_err(|e| e.to_string())?;

        // Then, translate the AST nodes into Cranelift IR.
        self.translate(ast)
            .map_err(|e| e.to_string())?;

        // Next, declare the function to simplejit. Functions must be declared
        // before they can be called, or defined.
        //
        // TODO: This may be an area where the API should be streamlined; should
        // we have a version of `declare_function` that automatically declares
        // the function?
        let id = self.module
            .declare_function(&name, Linkage::Export, &self.ctx.func.signature)
            .map_err(|e| e.to_string())?;

        // Define the function to simplejit. This finishes compilation, although
        // there may be outstanding relocations to perform. Currently, simplejit
        // cannot finish relocations until all functions to be called are
        // defined. For this toy demo for now, we'll just finalize the function
        // below.
        self.module
            .define_function(id, &mut self.ctx)
            .map_err(|e| e.to_string())?;

        // Now that compilation is finished, we can clear out the context state.
        self.module.clear_context(&mut self.ctx);

        // Finalize the function, finishing any outstanding relocations. The
        // result is a pointer to the finished machine code.
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

            Expression::Identifier(name) => {
                // `use_var` is used to read the value of a variable.
                let variable = self.variables.get(&name).expect("variable not defined");
                self.builder.use_var(*variable)
            }
        }
    }
}

