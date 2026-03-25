use std::fmt;

#[derive(Debug, Clone)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    LexError,
    ParseError,
    TypeError,
    CompileError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} at {}:{}: {}",
            self.kind_str(),
            self.line,
            self.col,
            self.message
        )
    }
}

impl Error {
    pub fn new(kind: ErrorKind, message: String, line: usize, col: usize) -> Self {
        Error {
            kind,
            message,
            line,
            col,
        }
    }
    
    fn kind_str(&self) -> &'static str {
        match self.kind {
            ErrorKind::LexError => "Lex error",
            ErrorKind::ParseError => "Parse error",
            ErrorKind::TypeError => "Type error",
            ErrorKind::CompileError => "Compile error",
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
