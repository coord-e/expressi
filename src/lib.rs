#![feature(box_syntax, box_patterns, never_type)]

#[cfg_attr(tarpaulin, skip)]
pub mod cli;

#[cfg_attr(tarpaulin, skip)]
pub mod test;

pub mod codegen;
pub mod expression;
pub mod ir;
pub mod parser;
pub mod scope;
pub mod transform;
pub mod translator;

pub mod error;
