extern crate expressi;

mod syntax {
    include!(concat!(env!("OUT_DIR"), "/syntax.rs"));
}

fn main() {
    match syntax::expression("1+1") {
        Ok(r) => println!("Parsed as: {:?}", r),
        Err(e) => println!("Parse error: {}", e),
    }
}
