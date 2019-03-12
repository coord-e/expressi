extern crate ansi_term;
extern crate clap;
extern crate expressi;
extern crate failure;

use ansi_term::Colour::{Blue, Red};
use clap::{App, Arg};
use failure::Error;
use rustyline::Editor;

use expressi::error::CLIError;
use expressi::jit;

use std::fs::File;
use std::io::prelude::*;

#[cfg_attr(tarpaulin, skip)]
fn compile_from_file(jit: &mut jit::JIT, path: &str) -> Result<(), Error> {
    let mut f = File::open(path).map_err(|_| CLIError::NotFound {
        path: path.to_owned(),
    })?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .map_err(|error| CLIError::IOError { error })?;

    let func = jit.compile("file_input", &contents.trim())?;
    println!("{}", unsafe { func.call() });
    Ok(())
}

#[cfg_attr(tarpaulin, skip)]
fn repl(jit: &mut jit::JIT, buffer: &str, line_count: u32) -> Result<(), Error> {
    let func = jit.compile(&format!("repl_{}", line_count), buffer.trim())?;
    println!(
        "{}{}",
        Blue.paint("-> "),
        Blue.paint(unsafe { func.call() }.to_string())
    );
    Ok(())
}

#[cfg_attr(tarpaulin, skip)]
fn main() {
    let matches = App::new("expressi")
        .version("0.1")
        .author("coord.e <me@coord-e.com>")
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .index(1),
        )
        .arg(
            Arg::with_name("print-ast")
                .long("print-ast")
                .help("Print ast"),
        )
        .arg(
            Arg::with_name("print-eir")
                .long("print-eir")
                .help("Print eir"),
        )
        .arg(
            Arg::with_name("print-ir")
                .long("print-ir")
                .help("Print llvm ir"),
        )
        .get_matches();

    let mut jit = jit::JIT::new(
        matches.is_present("print-ast"),
        matches.is_present("print-eir"),
        matches.is_present("print-ir"),
    )
    .unwrap();

    if matches.is_present("INPUT") {
        if let Err(e) = compile_from_file(&mut jit, matches.value_of("INPUT").unwrap()) {
            eprintln!("{}: {}", Red.paint("Error"), e);
        }
    } else {
        let mut line_count = 0;
        let mut rl = Editor::<()>::new();
        if rl.load_history("history.txt").is_err() {
            eprintln!("No previous history.");
        }
        loop {
            match rl.readline(&format!("{}: > ", line_count)) {
                Ok(line) => {
                    rl.add_history_entry(line.as_ref());
                    if let Err(e) = repl(&mut jit, &line, line_count) {
                        eprintln!("{}: {}", Red.paint("Error"), e);
                    }
                }
                Err(err) => {
                    eprintln!("{}: {}", Red.paint("Input Error"), err);
                    break;
                }
            }
            line_count += 1;
        }
        rl.save_history("history.txt").unwrap();
    }
}
