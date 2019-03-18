use super::llvm;
use crate::error::LLVMError;
use crate::parser;
use crate::transform::TransformManager;
use crate::translator::translate_ast;

use failure::Error;

use inkwell::execution_engine;
use inkwell::targets::{InitializationConfig, Target};
use inkwell::OptimizationLevel;

type CompiledFunc = unsafe extern "C" fn() -> u64;

pub struct JIT {
    print_ast: bool,
    print_eir: bool,
    print_ir: bool,
}

impl JIT {
    pub fn new(print_ast: bool, print_eir: bool, print_ir: bool) -> Result<Self, Error> {
        Target::initialize_native(&InitializationConfig::default())
            .map_err(|message| LLVMError::TargetInitializationFailed { message })?;

        Ok(Self {
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
        // Parse the string, producing AST nodes.
        let ast = parser::parse(&input)?;

        if self.print_ast {
            eprintln!("AST:\n{:#?}", ast);
        }

        let eir = translate_ast(ast)?;

        if self.print_eir {
            eprintln!("EIR:\n{}\n", eir);
        }

        let transformed = TransformManager::default().apply(eir)?;

        if self.print_eir {
            eprintln!("Transformed EIR:\n{}\n", transformed);
        }

        let result = llvm::compile_eir(transformed, name)?;

        if self.print_ir {
            eprintln!("LLVM IR: \n{}", result.llvm_ir());
        }

        result.verify()?;

        let execution_engine = result
            .module()
            .create_jit_execution_engine(OptimizationLevel::None)
            .map_err(|_| LLVMError::FailedToCreateJIT)?;

        unsafe { execution_engine.get_function(name) }.map_err(Into::into)
    }
}