use std::time::SystemTime;
use crate::parser::token::{TokenEnum};

mod parser;
mod ast;

fn main() {
    let now = SystemTime::now();
    TokenEnum::try_lexer("./lua/test.lua").unwrap();
    println!("{:?}", now.elapsed())
}
