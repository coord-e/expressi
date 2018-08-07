use expression::Expression;

pub mod syntax {
    include!(concat!(env!("OUT_DIR"), "/syntax.rs"));
}

pub fn parse(x: &str) -> Result<Expression, syntax::ParseError> {
    syntax::expression(x)
}
