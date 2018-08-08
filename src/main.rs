extern crate expressi;

use expressi::jit;

use std::mem;
use std::io;
use std::io::prelude::*;

fn main() {
    let mut jit = jit::JIT::new();
    let mut line_count = 0;
    loop {
        print!("{}: > ", line_count);
        io::stdout().flush().expect("Failed to flush stdout");

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).expect("Failed to read line");

        match jit.compile(&format!("repl_{}", line_count), &buffer.trim()) {
            Ok(func) =>  {
                let func = unsafe { mem::transmute::<_, fn() -> isize>(func) };
                println!("-> {}", func());
            },
            Err(msg) => {
                eprintln!("error: {}", msg);
            }
        }
        line_count += 1;
    }
}
