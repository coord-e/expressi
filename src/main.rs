extern crate expressi;

use expressi::jit;

use std::mem;
use std::process;

fn main() {
    let mut jit = jit::JIT::new();
    let foo = jit.compile("1+2").unwrap_or_else(|msg| {
        eprintln!("error: {}", msg);
        process::exit(1);
    });
    let foo = unsafe { mem::transmute::<_, fn() -> isize>(foo) };

    // And now we can call it!
    println!("the answer is: {}", foo());
}
