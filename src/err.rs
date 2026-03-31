use std::fmt;

#[derive(Debug, Clone)]
pub enum SlimeError {
    LexError(String),
    ParseError(String),
    TypeError(String),
    CodegenError(String),
}

impl fmt::Display for SlimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SlimeError::LexError(msg) => write!(f, "Lex error: {}", msg),
            SlimeError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            SlimeError::TypeError(msg) => write!(f, "Type error: {}", msg),
            SlimeError::CodegenError(msg) => write!(f, "Codegen error: {}", msg),
        }
    }
}

pub type Result<T> = std::result::Result<T, SlimeError>;
pub type Error = SlimeError;
pub type ErrorKind = SlimeError;
