extern crate ansi_term;
extern crate clap;
extern crate expressi;
extern crate failure;

use ansi_term::Colour::{Blue, Red};
use clap::{App, Arg};
use failure::Error;

use expressi::error::{IOError, NotFoundError};
use expressi::jit;

use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::mem;

fn compile_from_file(jit: &mut jit::JIT, path: &str) -> Result<(), Error> {
    let mut f = File::open(path).map_err(|_| NotFoundError {
        path: path.to_owned(),
    })?;
    let mut contents = String::new();
    f.read_to_string(&mut contents).map_err(|_| IOError {
        message: "Failed to read file".to_owned(),
    })?;
    let func_obj = jit.compile("file_input", &contents.trim())?;
    let func = unsafe { mem::transmute::<_, fn() -> isize>(func_obj) };
    println!("{}", func());
    Ok(())
}

fn repl(jit: &mut jit::JIT, line_count: u32) -> Result<(), Error> {
    print!("{}: > ", line_count);
    io::stdout().flush().map_err(|_| IOError {
        message: "Failed to flush stdin".to_owned(),
    })?;

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).map_err(|_| IOError {
        message: "Failed to read stdin".to_owned(),
    })?;

    let func_obj = jit.compile(&format!("repl_{}", line_count), &buffer.trim())?;
    let func = unsafe { mem::transmute::<_, fn() -> isize>(func_obj) };
    println!("{}{}", Blue.paint("-> "), Blue.paint(func().to_string()));
    Ok(())
}

fn main() {
    let matches = App::new("expressi")
        .version("0.1")
        .author("coord.e <me@coord-e.com>")
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .index(1),
        )
        .get_matches();

    let mut jit = jit::JIT::new();

    if matches.is_present("INPUT") {
        if let Err(e) = compile_from_file(&mut jit, matches.value_of("INPUT").unwrap()) {
            eprintln!("{}: {}", Red.paint("Error"), e);
        }
    } else {
        let mut line_count = 0;
        loop {
            if let Err(e) = repl(&mut jit, line_count) {
                eprintln!("{}: {}", Red.paint("Error"), e);
            }
            line_count += 1;
        }
    }
}
