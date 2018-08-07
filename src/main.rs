extern crate expressi;

fn main() {
    match expressi::parser::parse("1+1") {
        Ok(r) => println!("Parsed as: {:?}", r),
        Err(e) => println!("Parse error: {}", e),
    }
}
