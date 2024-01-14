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
    #[regex(r"[\n\f]", line)]
    Line,
    #[regex(r"--[^\n\f]*", to_string_token)]
    Comment(Token<String>),
    #[token("=")]
    Equal,
    #[token("+")]
    Plus,
    #[token(";")]
    Semicolon,
    #[token(",")]
    Comma,
    #[token("(")]
    ParenthesesLeft,
    #[token(")")]
    ParenthesesRight,
    #[regex(r"[_a-zA-Z][_0-9a-zA-Z]*", to_string_token)]
    Name(Token<String>),
    #[regex("(0x)?[0-9]+", to_u64_token)]
    Int(Token<u64>),
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

impl Token<String> {
    #[inline]
    fn from_lexer(lex: &mut Lexer<TokenEnum>) -> Option<Token<String>> {
        Some(Self::new(lex.slice().to_string(), lex.span(), lex.extras.line_breaks))
    }
}

impl Token<u64> {
    #[inline]
    fn from_lexer(lex: &mut Lexer<TokenEnum>) -> Option<Token<u64>> {
        let mut x = lex.slice();
        let radix = if x.starts_with("0x") {
            x = &x[..2];
            16
        } else { 10 };
        match u64::from_str_radix(x, radix) {
            Ok(v) => Some(Token::new(v, lex.span(), lex.extras.line_breaks)),
            Err(err) => {
                let source = &lex.source()[..lex.extras.line_start];
                println!("{:?}: {}, {}\nerr: {:?}",
                         lex.extras.file_path,
                         lex.extras.line_breaks,
                         &source[..source.as_bytes().iter().position(|&c| c == b'\n').unwrap_or(source.len())],
                         err);
                None
            }
        }
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
fn line(lex: &mut Lexer<TokenEnum>) -> logos::Skip {
    lex.extras.line_breaks += 1;
    lex.extras.line_start = lex.span().end;
    logos::Skip
}

#[inline]
fn to_string_token(lex: &mut Lexer<TokenEnum>) -> Option<Token<String>> {
    Token::<String>::from_lexer(lex)
}

#[inline]
fn to_u64_token(lex: &mut Lexer<TokenEnum>) -> Option<Token<u64>> {
    Token::<u64>::from_lexer(lex)
}