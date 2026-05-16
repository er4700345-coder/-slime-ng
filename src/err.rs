use std::fmt;

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub enum SlimeError {
    LexError { msg: String, loc: SourceLocation },
    ParseError { msg: String, loc: SourceLocation },
    TypeError { msg: String, loc: Option<SourceLocation> },
    CodegenError(String),
}

impl fmt::Display for SlimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SlimeError::LexError { msg, loc } => write!(f, "Lex error at {}:{}: {}", loc.line, loc.col, msg),
            SlimeError::ParseError { msg, loc } => write!(f, "Parse error at {}:{}: {}", loc.line, loc.col, msg),
            SlimeError::TypeError { msg, loc } => {
                if let Some(l) = loc {
                    write!(f, "Type error at {}:{}: {}", l.line, l.col, msg)
                } else {
                    write!(f, "Type error: {}", msg)
                }
            }
            SlimeError::CodegenError(msg) => write!(f, "Codegen error: {}", msg),
        }
    }
}

pub type Result<T> = std::result::Result<T, SlimeError>;
pub type Error = SlimeError;
pub type ErrorKind = SlimeError;
