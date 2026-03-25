use crate::error::{Error, ErrorKind, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Fn, Let, Return, Target, True, False, If, Else, While,
    I32, I64, F32, F64, Bool, String, Void,
    Identifier(String),
    Integer(i64),
    Float(f64),
    StringLit(String),
    Arrow, Eq, Neq, Lt, Gt, Lte, Gte, And, Or,
    Assign, Plus, Minus, Star, Slash, Bang,
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Semicolon, Colon, Comma, Dot,
    EOF,
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }
    
    pub fn line(&self) -> usize { self.line }
    pub fn col(&self) -> usize { self.col }
    
    fn peek(&self) -> char {
        self.input.get(self.pos).copied().unwrap_or('\0')
    }
    
    fn advance(&mut self) -> char {
        let ch = self.peek();
        self.pos += 1;
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        ch
    }
    
    fn skip_whitespace(&mut self) {
        while self.peek().is_whitespace() {
            self.advance();
        }
    }
    
    fn read_identifier(&mut self) -> String {
        let mut result = String::new();
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            result.push(self.advance());
        }
        result
    }
    
    fn read_number(&mut self) -> Result<Token> {
        let mut result = String::new();
        while self.peek().is_ascii_digit() {
            result.push(self.advance());
        }
        
        if self.peek() == '.' {
            result.push(self.advance());
            if !self.peek().is_ascii_digit() {
                return Err(Error::new(
                    ErrorKind::LexError,
                    "Expected digits after decimal point".to_string(),
                    self.line,
                    self.col,
                ));
            }
            while self.peek().is_ascii_digit() {
                result.push(self.advance());
            }
            Ok(Token::Float(result.parse().unwrap()))
        } else {
            Ok(Token::Integer(result.parse().unwrap()))
        }
    }
    
    fn read_string(&mut self) -> Result<Token> {
        self.advance();
        let mut result = String::new();
        while self.peek() != '"' && self.peek() != '\0' {
            if self.peek() == '\n' {
                return Err(Error::new(
                    ErrorKind::LexError,
                    "Unterminated string literal".to_string(),
                    self.line,
                    self.col,
                ));
            }
            result.push(self.advance());
        }
        if self.peek() != '"' {
            return Err(Error::new(
                ErrorKind::LexError,
                "Unterminated string literal".to_string(),
                self.line,
                self.col,
            ));
        }
        self.advance();
        Ok(Token::StringLit(result))
    }
    
    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();
        
        match self.peek() {
            '\0' => Ok(Token::EOF),
            '(' => { self.advance(); Ok(Token::LParen) }
            ')' => { self.advance(); Ok(Token::RParen) }
            '{' => { self.advance(); Ok(Token::LBrace) }
            '}' => { self.advance(); Ok(Token::RBrace) }
            '[' => { self.advance(); Ok(Token::LBracket) }
            ']' => { self.advance(); Ok(Token::RBracket) }
            ';' => { self.advance(); Ok(Token::Semicolon) }
            ':' => { self.advance(); Ok(Token::Colon) }
            ',' => { self.advance(); Ok(Token::Comma) }
            '.' => { self.advance(); Ok(Token::Dot) }
            '+' => { self.advance(); Ok(Token::Plus) }
            '-' => {
                self.advance();
                if self.peek() == '>' {
                    self.advance();
                    Ok(Token::Arrow)
                } else {
                    Ok(Token::Minus)
                }
            }
            '*' => { self.advance(); Ok(Token::Star) }
            '/' => { self.advance(); Ok(Token::Slash) }
            '!' => {
                self.advance();
                if self.peek() == '=' {
                    self.advance();
                    Ok(Token::Neq)
                } else {
                    Ok(Token::Bang)
                }
            }
            '=' => {
                self.advance();
                if self.peek() == '=' {
                    self.advance();
                    Ok(Token::Eq)
                } else {
                    Ok(Token::Assign)
                }
            }
            '<' => {
                self.advance();
                if self.peek() == '=' {
                    self.advance();
                    Ok(Token::Lte)
                } else {
                    Ok(Token::Lt)
                }
            }
            '>' => {
                self.advance();
                if self.peek() == '=' {
                    self.advance();
                    Ok(Token::Gte)
                } else {
                    Ok(Token::Gt)
                }
            }
            '&' => {
                self.advance();
                if self.peek() == '&' {
                    self.advance();
                    Ok(Token::And)
                } else {
                    Err(Error::new(
                        ErrorKind::LexError,
                        "Unexpected character: &".to_string(),
                        self.line,
                        self.col,
                    ))
                }
            }
            '|' => {
                self.advance();
                if self.peek() == '|' {
                    self.advance();
                    Ok(Token::Or)
                } else {
                    Err(Error::new(
                        ErrorKind::LexError,
                        "Unexpected character: |".to_string(),
                        self.line,
                        self.col,
                    ))
                }
            }
            '"' => self.read_string(),
            ch if ch.is_ascii_digit() => self.read_number(),
            ch if ch.is_alphabetic() || ch == '_' => {
                let ident = self.read_identifier();
                Ok(match ident.as_str() {
                    "fn" => Token::Fn,
                    "let" => Token::Let,
                    "return" => Token::Return,
                    "target" => Token::Target,
                    "true" => Token::True,
                    "false" => Token::False,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "while" => Token::While,
                    "i32" => Token::I32,
                    "i64" => Token::I64,
                    "f32" => Token::F32,
                    "f64" => Token::F64,
                    "bool" => Token::Bool,
                    "string" => Token::String,
                    "void" => Token::Void,
                    _ => Token::Identifier(ident),
                })
            }
            ch => Err(Error::new(
                ErrorKind::LexError,
                format!("Unexpected character: {}", ch),
                self.line,
                self.col,
            )),
        }
    }
}
