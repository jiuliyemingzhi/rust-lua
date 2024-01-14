use std::fmt::Debug;
use std::fs::File;
use std::io::Read;
use std::ops::Range;
use logos::{Lexer, Logos, Span};

pub struct TokenExtras {
    line_breaks: usize,
    line_start: usize,
    file_path: String,
    before_token_is_separate: bool,
    before_token_start: usize,
}

impl TokenExtras {
    pub fn println_err(&self, lex: &Lexer<TokenEnum>, span: Range<usize>, reason: &str) {
        let x = &lex.source()[span];
        println!("{}:{}:  {} '{}'", self.file_path, self.line_breaks, reason, x)
    }
}

impl Default for TokenExtras {
    fn default() -> Self {
        Self {
            line_breaks: 1,
            line_start: 0,
            file_path: "".to_string(),
            before_token_is_separate: true,
            before_token_start: 0,
        }
    }
}

#[derive(Logos, Debug)]
#[logos(extras = TokenExtras)]
pub enum TokenEnum {
    #[regex(r"[ \t]+")]
    Skip,
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

impl TokenEnum {
    pub fn is_separate(&self) -> bool {
        match self {
            TokenEnum::Skip
            | TokenEnum::Line
            | TokenEnum::Comment(_)
            | TokenEnum::Equal
            | TokenEnum::Plus
            | TokenEnum::Semicolon
            | TokenEnum::Comma
            | TokenEnum::DoubleDot
            | TokenEnum::ParenthesesLeft
            | TokenEnum::ParenthesesRight => true,
            _ => false,
        }
    }
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
        Err(err) => print_err(lex, err.to_string().as_str())
    }
}

fn f64_lexer(lex: &mut Lexer<TokenEnum>) -> Option<Token<f64>> {
    match lex.slice().parse() {
        Ok(v) => Some(Token::new(v, lex.span(), lex.extras.line_breaks)),
        Err(err) => print_err(lex, err.to_string().as_str())
    }
}

impl TokenEnum {
    pub fn try_lexer(lua_path: &str) -> anyhow::Result<usize> {
        let mut content = String::new();
        File::open(lua_path)?.read_to_string(&mut content)?;
        let mut lex = Self::lexer(content.as_str());
        lex.extras.file_path = lua_path.to_string();
        let mut token_list: Vec<TokenEnum> = Vec::new();
        while let Some(token) = lex.next() {
            // println!("{:?} {}", token, lex.extras.line_start);
            if let Ok(ok) = token {
                on_token(&mut lex, &ok);
                token_list.push(ok);
            }
        }
        Ok(lex.count())
    }
}

#[inline]
fn print_err<T>(lex: &Lexer<TokenEnum>, err: &str) -> Option<Token<T>> {
    lex.extras.println_err(lex, lex.span(), err);
    None
}


#[inline]
fn line(lex: &mut Lexer<TokenEnum>) -> logos::Skip {
    lex.extras.line_breaks += 1;
    lex.extras.line_start = lex.span().start;
    logos::Skip
}

fn on_token(lex: &mut Lexer<TokenEnum>, token: &TokenEnum) {
    if !token.is_separate() && !lex.extras.before_token_is_separate {
        lex.extras.println_err(lex, lex.extras.before_token_start..lex.span().end, "unknown token")
    }
    lex.extras.before_token_is_separate = token.is_separate();
    lex.extras.before_token_start = lex.span().start;
}