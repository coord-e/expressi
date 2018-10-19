use builder::Builder;
use error::{FinalizationError, ParseError, FailedToCreateJITError};
use expression::Expression;
use parser;
use translator::FunctionTranslator;
use value::{Type, ValueStore};
use scope::ScopeStack;

use std::collections::HashMap;
use std::rc::Rc;

use failure::Error;

use scopeguard;

use inkwell::{module,builder,context,execution_engine};
use inkwell::OptimizationLevel;

type CompiledFunc = unsafe extern "C" fn() -> u64;

pub struct JIT {
    context: context::Context,
    module: Rc<module::Module>,
    builder: builder::Builder,
    execution_engine: execution_engine::ExecutionEngine,
}

impl JIT {
    pub fn new() -> Result<Self, Error> {

        let context = context::Context::create();
        let module = Rc::new(context.create_module("expressi"));
        let builder = context.create_builder();
        let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None).map_err(|_| FailedToCreateJITError.into())?;

        Ok(Self {
            context,
            module,
            builder,
            execution_engine
        })
    }

    /// Compile a string in the toy language into machine code.
    pub fn compile(&mut self, name: &str, input: &str) -> Result<execution_engine::Symbol<CompiledFunc>, Error> {
        // Parse the string, producing AST nodes.
        let ast = parser::parse(&input).map_err(|e| ParseError {
            message: e.to_string(),
        })?;

        // Translate the AST nodes into Cranelift IR.
        self.translate(name, ast)?;

        unsafe { self.execution_engine.get_function(name) }.map_err(|e| e.into())
    }

    // Translate from toy-language AST nodes into Cranelift IR.
    fn translate(&mut self, name: &str, expr: Expression) -> Result<(), Error> {
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(&[], false);

        let function = self.module.add_function(name, fn_type, None);
        let basic_block = self.context.append_basic_block(&function, "entry");

        self.builder.position_at_end(&basic_block);

        let builder = Builder::new(&mut self.builder, self.module);

        let trans = FunctionTranslator {
            builder
        };

        let evaluated_value = trans.translate_expr(expr)?;
        let return_value = if evaluated_value.get_type() != Type::Number {
            trans.builder.cast_to(evaluated_value, Type::Number)?
        } else {
            evaluated_value
        };
        // Emit the return instruction.
        let cl = trans.builder.to_cl(return_value)?.into_int_value();
        trans
            .builder
            .inst_builder()
            .build_return(Some(&cl));

        Ok(())
    }
}
