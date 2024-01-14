use std::error::Error;
use std::fmt::Debug;
use std::fs::File;
use std::io::Read;
use logos::{Lexer, Logos, Span};

pub struct TokenExtras {
    line_breaks: usize,
    line_start: usize,
    file_path: String,
}

impl Default for TokenExtras {
    fn default() -> Self {
        Self {
            line_breaks: 1,
            line_start: 0,
            file_path: "".to_string(),
        }
    }
}

#[derive(Logos, Debug)]
#[logos(extras = TokenExtras)]
#[logos(skip r"[ \t]")]
pub enum TokenEnum {
    #[regex(r"(\r\n)|[\n\f\r]", line)]
    Line,
    #[regex(r"--[^\n\f\r]*", str_lexer)]
    Comment(Token<String>),
    #[regex(r#""([^"\\]|\\.|"")*""#, string_lexer)]
    #[regex(r#"'([^'\\]|\\.|'')*'"#, string_lexer)]
    QuotedString(Token<String>),
    #[token("=")]
    Equal,
    #[token("+")]
    Plus,
    #[token(";")]
    Semicolon,
    #[token(",")]
    Comma,
    #[token("..")]
    DoubleDot,
    #[token("(")]
    ParenthesesLeft,
    #[token(")")]
    ParenthesesRight,
    #[regex(r"[_a-zA-Z][_0-9a-zA-Z]*", str_lexer)]
    Name(Token<String>),
    #[regex("(0x)?[0-9]+", u64_lexer)]
    Int(Token<u64>),
    #[regex(r"([0-9]*\.[0-9]+([eE][+-]?[0-9]+)?)|([0-9]+\.[0-9]*([eE][+-]?[0-9]+)?)|([0-9]+[eE][+-]?[0-9]+)", f64_lexer)]
    Float(Token<f64>),
    #[token("function")]
    Function,
    #[token("end")]
    End,
    #[token("local")]
    Local,
}

#[derive(Debug)]
pub struct Token<T> {
    pub line: usize,
    pub span: Span,
    pub v: T,
}


impl<T> Token<T> {
    #[inline]
    fn new(v: T, span: Span, line: usize) -> Self {
        Self {
            line,
            span,
            v,
        }
    }
}

#[inline]
fn str_lexer(lex: &mut Lexer<TokenEnum>) -> Option<Token<String>> {
    Some(Token::new(lex.slice().to_string(), lex.span(), lex.extras.line_breaks))
}

#[inline]
fn string_lexer(lex: &mut Lexer<TokenEnum>) -> Option<Token<String>> {
    let span = lex.span();
    let x = lex.slice();
    Some(Token::new(x[..x.len() - 1].replace("\\", ""), span, lex.extras.line_breaks))
}

fn u64_lexer(lex: &mut Lexer<TokenEnum>) -> Option<Token<u64>> {
    let mut x = lex.slice();
    let radix = if x.starts_with("0x") {
        x = &x[2..];
        16
    } else { 10 };
    match u64::from_str_radix(x, radix) {
        Ok(v) => Some(Token::new(v, lex.span(), lex.extras.line_breaks)),
        Err(err) => print_err(lex, Box::new(err))
    }
}

fn f64_lexer(lex: &mut Lexer<TokenEnum>) -> Option<Token<f64>> {
    match lex.slice().parse() {
        Ok(v) => Some(Token::new(v, lex.span(), lex.extras.line_breaks)),
        Err(err) => print_err(lex, Box::new(err))
    }
}

impl TokenEnum {
    pub fn try_lexer(lua_path: &str) -> anyhow::Result<usize> {
        let mut content = String::new();
        File::open(lua_path)?.read_to_string(&mut content)?;
        let mut lex = Self::lexer(content.as_str());
        lex.extras.file_path = lua_path.to_string();
        while let Some(token) = lex.next() {
            println!("{:?}", token)
        }
        Ok(lex.count())
    }
}

#[inline]
fn print_err<T>(lex: &Lexer<TokenEnum>, err: Box<dyn Error>) -> Option<Token<T>> {
    let source = &lex.source()[lex.extras.line_start..];
    println!("{:?}: {}, {} {}\nerr: {:?}",
             lex.extras.file_path,
             lex.extras.line_breaks,
             lex.slice(),
             &source[..source.as_bytes().iter().position(|&c| c == b'\n' || c == b'\r').unwrap_or(source.len())],
             err);
    None
}

#[inline]
fn line(lex: &mut Lexer<TokenEnum>) -> logos::Skip {
    lex.extras.line_breaks += 1;
    lex.extras.line_start = lex.span().start;
    logos::Skip
}

