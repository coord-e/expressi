mod syntax {
    include!(concat!(env!("OUT_DIR"), "/syntax.rs"));
}

fn main() {
    match syntax::word("internal") {
        Ok(r) => println!("Parsed as: {:?}", r),
        Err(e) => println!("Parse error: {}", e),
    }
}
