use super::llvm;
use super::opts::RunOpt;
use crate::error::{CLIError, LLVMError};
use crate::parser;
use crate::shell::Shell;
use crate::transform::TransformManager;
use crate::translator::translate_ast;

use failure::Error;

use ansi_term::Colour::{Blue, Red};
use inkwell::execution_engine;
use inkwell::targets::{InitializationConfig, Target};
use inkwell::OptimizationLevel;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::process;

type CompiledFunc = unsafe extern "C" fn() -> u64;

pub fn run(opt: &RunOpt) -> Result<!, Error> {
    let mut jit = JIT::new(opt.print_ast, opt.print_eir, opt.print_ir)?;

    if let Some(path) = &opt.input {
        let mut f = File::open(path).map_err(|_| CLIError::NotFound { path: path.clone() })?;
        let mut contents = String::new();
        f.read_to_string(&mut contents)
            .map_err(|error| CLIError::IOError { error })?;

        let func = jit.compile("file_input", &contents.trim())?;
        process::exit(unsafe { func.call() } as i32)
    } else {
        let home = dirs::home_dir().unwrap_or_else(|| env::current_dir().unwrap());
        let mut shell = Shell::new(home.join(".expressi_history"));
        loop {
            let line = shell.get_next_line()?;
            match jit.compile("repl", line.trim()) {
                Ok(func) => {
                    println!(
                        "{}{}",
                        Blue.paint("-> "),
                        Blue.paint(unsafe { func.call() }.to_string())
                    );
                }
                Err(e) => {
                    eprintln!("{}: {}", Red.paint("Error"), e);
                }
            }
        }
    }
}

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
