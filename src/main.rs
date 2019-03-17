use ansi_term::Colour::{Blue, Red};
use failure::Error;
use structopt::StructOpt;

use expressi::build::{self, BuildOpt};
use expressi::error::CLIError;
use expressi::jit;
use expressi::shell::Shell;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

#[derive(StructOpt)]
struct RunOpt {
    #[structopt(name = "FILE", parse(from_os_str))]
    input: Option<PathBuf>,

    #[structopt(long = "print-ast")]
    print_ast: bool,

    #[structopt(long = "print-eir")]
    print_eir: bool,

    #[structopt(long = "print-ir")]
    print_ir: bool,
}

#[derive(StructOpt)]
enum Opt {
    #[structopt(name = "run")]
    Run {
        #[structopt(flatten)]
        opt: RunOpt,
    },
    #[structopt(name = "build")]
    Build {
        #[structopt(flatten)]
        opt: BuildOpt,
    },
}

#[cfg_attr(tarpaulin, skip)]
fn compile_from_file(jit: &mut jit::JIT, path: &PathBuf) -> Result<(), Error> {
    let mut f = File::open(path).map_err(|_| CLIError::NotFound { path: path.clone() })?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .map_err(|error| CLIError::IOError { error })?;

    let func = jit.compile("file_input", &contents.trim())?;
    println!("{}", unsafe { func.call() });
    Ok(())
}

#[cfg_attr(tarpaulin, skip)]
fn repl(jit: &mut jit::JIT, buffer: &str) -> Result<(), Error> {
    let func = jit.compile("repl", buffer.trim())?;
    println!(
        "{}{}",
        Blue.paint("-> "),
        Blue.paint(unsafe { func.call() }.to_string())
    );
    Ok(())
}

#[cfg_attr(tarpaulin, skip)]
fn main() {
    match Opt::from_args() {
        Opt::Run { opt } => {
            let mut jit = jit::JIT::new(opt.print_ast, opt.print_eir, opt.print_ir).unwrap();

            if let Some(file) = opt.input {
                if let Err(e) = compile_from_file(&mut jit, &file) {
                    eprintln!("{}: {}", Red.paint("Error"), e);
                }
            } else {
                let home = dirs::home_dir().unwrap_or_else(|| env::current_dir().unwrap());
                let mut shell = Shell::new(home.join(".expressi_history"));
                loop {
                    match shell.get_next_line() {
                        Ok(line) => {
                            if let Err(e) = repl(&mut jit, &line) {
                                eprintln!("{}: {}", Red.paint("Error"), e);
                            }
                        }
                        Err(err) => {
                            eprintln!("{}: {}", Red.paint("Input Error"), err);
                            break;
                        }
                    }
                }
            }
        }
        Opt::Build { opt } => {
            if let Err(e) = build::build(opt) {
                eprintln!("{}: {}", Red.paint("Error"), e);
            }
        }
    }
}
