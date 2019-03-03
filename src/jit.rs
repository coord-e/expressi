use crate::error::{LLVMError, ParseError};
use crate::expression::Expression;
use crate::ir::Printer;
use crate::parser;
use crate::transform::{CheckCapture, TypeInfer};
use crate::translator::eir_translator::Builder;
use crate::translator::{translate_ast, translate_eir};

use failure::Error;
use std::io;
use std::rc::Rc;

use inkwell::targets::{InitializationConfig, Target};
use inkwell::OptimizationLevel;
use inkwell::{builder, context, execution_engine, module};

type CompiledFunc = unsafe extern "C" fn() -> u64;

pub struct JIT {
    context: context::ContextRef,
    builder: builder::Builder,
    print_ast: bool,
    print_eir: bool,
    print_ir: bool,
}

impl JIT {
    pub fn new(print_ast: bool, print_eir: bool, print_ir: bool) -> Result<Self, Error> {
        Target::initialize_native(&InitializationConfig::default())
            .map_err(|message| LLVMError::TargetInitializationFailed { message })?;

        let context = context::Context::get_global();
        let builder = context.create_builder();

        Ok(Self {
            context,
            builder,
            print_ast,
            print_eir,
            print_ir,
        })
    }

    /// Compile a string in the toy language into machine code.
    pub fn compile(
        &mut self,
        name: &str,
        input: &str,
    ) -> Result<execution_engine::JitFunction<CompiledFunc>, Error> {
        let module = Rc::new(self.context.create_module(name));
        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .map_err(|_| LLVMError::FailedToCreateJIT)?;

        // Parse the string, producing AST nodes.
        let ast = parser::parse(&input).map_err(|e| ParseError {
            message: e.to_string(),
        })?;
        if self.print_ast {
            eprintln!("AST:\n{:#?}", ast);
        }

        // Translate the AST nodes into Cranelift IR.
        self.translate(module.clone(), ast)?;

        unsafe { execution_engine.get_function(name) }.map_err(Into::into)
    }

    // Translate from toy-language AST nodes into LLVM IR.
    fn translate(&mut self, module: Rc<module::Module>, expr: Expression) -> Result<(), Error> {
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(&[], false);

        let function = module.add_function(module.get_name().to_str()?, fn_type, None);
        let basic_block = self.context.append_basic_block(&function, "entry");

        self.builder.position_at_end(&basic_block);

        let eir = translate_ast(expr)?;

        if self.print_eir {
            eprintln!("EIR:");
            let printer = Printer::new();
            printer.print(&eir, &mut io::stderr())?;
            eprintln!();
        }

        let ti = TypeInfer::new();
        let cc = CheckCapture::new();
        let transformed = eir.apply(ti)?.apply(cc)?;

        if self.print_eir {
            eprintln!("Transformed EIR:");
            let printer = Printer::new();
            printer.print(&transformed, &mut io::stderr())?;
            eprintln!();
        }

        let mut builder = Builder::new(&mut self.builder, module.clone());

        let evaluated_value = translate_eir(&mut builder, transformed)?.expect_value()?;
        builder.ret_int(evaluated_value)?;

        if self.print_ir {
            eprintln!("LLVM IR:");
            module.print_to_stderr();
        }

        if !function.verify(true) {
            eprintln!(""); // function.verify print results to stderr directory but it doesn't contain \n on the end
            return Err(LLVMError::FunctionVerificationError {
                name: function.get_name().to_str()?.to_string(),
            }
            .into());
        }

        if let Err(message) = module.verify() {
            return Err(LLVMError::ModuleVerificationError {
                message: message.to_string(),
            }
            .into());
        }

        Ok(())
    }
}
