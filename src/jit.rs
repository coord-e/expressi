use builder::Builder;
use expression::Expression;
use parser;
use translator::FunctionTranslator;

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
        self.ctx
            .func
            .signature
            .returns
            .push(AbiParam::new(types::I64));

        let mut function_builder =
            FunctionBuilder::<Variable>::new(&mut self.ctx.func, &mut self.builder_context);

        // TODO: Replace FunctionBuilder with Builder
        let entry_ebb = function_builder.create_ebb();

        function_builder.append_ebb_params_for_function_params(entry_ebb);

        function_builder.switch_to_block(entry_ebb);
        function_builder.seal_block(entry_ebb);

        let builder = Builder {
            inst_builder: &mut function_builder,
            variable_map: HashMap::new(),
            variable_value_map: HashMap::new(),
            block_table: HashMap::new(),
        };

        let mut trans = FunctionTranslator {
            builder,
            module: &mut self.module,
        };
        let evaluated_value = trans.translate_expr(expr);
        let return_value = if evaluated_value.get_type().cl_type().unwrap() != types::I64 {
            trans
                .builder
                .inst_builder()
                .ins()
                .bint(types::I64, evaluated_value.cl_value())
        } else {
            evaluated_value.cl_value()
        };
        // Emit the return instruction.
        trans.builder.inst_builder().ins().return_(&[return_value]);

        // Tell the builder we're done with this function.
        trans.builder.finalize();
        Ok(())
    }
}
