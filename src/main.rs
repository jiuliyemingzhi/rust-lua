use crate::parser::token::{ TokenEnum};

mod parser;
mod ast;

fn main() {
    TokenEnum::try_lexer("./lua/test.lua").unwrap();
}
