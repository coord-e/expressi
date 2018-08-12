extern crate clap;
extern crate expressi;
extern crate ansi_term;

use clap::{App, Arg};
use ansi_term::Colour::{Red, Blue};

use expressi::jit;

use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::mem;

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
        let mut f = File::open(matches.value_of("INPUT").unwrap()).expect("file not found");
        let mut contents = String::new();
        f.read_to_string(&mut contents)
            .expect("something went wrong reading the file");
        let func_obj = jit
            .compile("file_input", &contents.trim())
            .unwrap_or_else(|msg| panic!("{}: {}", Red.paint("Error"), msg));
        let func = unsafe { mem::transmute::<_, fn() -> isize>(func_obj) };
        println!("{}", func());
    } else {
        let mut line_count = 0;
        loop {
            print!("{}: > ", line_count);
            io::stdout().flush().expect("Failed to flush stdout");

            let mut buffer = String::new();
            io::stdin()
                .read_line(&mut buffer)
                .expect("Failed to read line");

            match jit.compile(&format!("repl_{}", line_count), &buffer.trim()) {
                Ok(func) => {
                    let func = unsafe { mem::transmute::<_, fn() -> isize>(func) };
                    println!("{}{}", Blue.paint("-> "), Blue.paint(func().to_string()));
                }
                Err(msg) => {
                    eprintln!("{}: {}", Red.paint("Error"), msg);
                }
            }
            line_count += 1;
        }
    }
}
