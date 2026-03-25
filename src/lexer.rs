// SLIME Lexer v0.1
// Run at: play.rust-lang.org

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Fn, Let, Return, Target, True, False, If, Else, While,
    
    // Types
    I32, I64, F32, F64, Bool, String, Void,
    
    // Literals
    Identifier(String),
    Integer(i64),
    Float(f64),
    StringLit(String),
    
    // Operators
    Arrow, Eq, Neq, Lt, Gt, Lte, Gte, And, Or,
    Assign, Plus, Minus, Star, Slash, Bang,
    
    // Delimiters
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
    
    fn read_number(&mut self) -> Token {
        let mut result = String::new();
        while self.peek().is_ascii_digit() {
            result.push(self.advance());
        }
        
        if self.peek() == '.' {
            result.push(self.advance());
            while self.peek().is_ascii_digit() {
                result.push(self.advance());
            }
            Token::Float(result.parse().unwrap())
        } else {
            Token::Integer(result.parse().unwrap())
        }
    }
    
    fn read_string(&mut self) -> Token {
        self.advance(); // consume opening "
        let mut result = String::new();
        while self.peek() != '"' && self.peek() != '\0' {
            result.push(self.advance());
        }
        self.advance(); // consume closing "
        Token::StringLit(result)
    }
    
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        
        match self.peek() {
            '\0' => Token::EOF,
            '(' => { self.advance(); Token::LParen }
            ')' => { self.advance(); Token::RParen }
            '{' => { self.advance(); Token::LBrace }
            '}' => { self.advance(); Token::RBrace }
            '[' => { self.advance(); Token::LBracket }
            ']' => { self.advance(); Token::RBracket }
            ';' => { self.advance(); Token::Semicolon }
            ':' => { self.advance(); Token::Colon }
            ',' => { self.advance(); Token::Comma }
            '.' => { self.advance(); Token::Dot }
            '+' => { self.advance(); Token::Plus }
            '-' => {
                self.advance();
                if self.peek() == '>' {
                    self.advance();
                    Token::Arrow
                } else {
                    Token::Minus
                }
            }
            '*' => { self.advance(); Token::Star }
            '/' => { self.advance(); Token::Slash }
            '!' => {
                self.advance();
                if self.peek() == '=' {
                    self.advance();
                    Token::Neq
                } else {
                    Token::Bang
                }
            }
            '=' => {
                self.advance();
                if self.peek() == '=' {
                    self.advance();
                    Token::Eq
                } else {
                    Token::Assign
                }
            }
            '<' => {
                self.advance();
                if self.peek() == '=' {
                    self.advance();
                    Token::Lte
                } else {
                    Token::Lt
                }
            }
            '>' => {
                self.advance();
                if self.peek() == '=' {
                    self.advance();
                    Token::Gte
                } else {
                    Token::Gt
                }
            }
            '&' => {
                self.advance();
                if self.peek() == '&' {
                    self.advance();
                    Token::And
                } else {
                    panic!("Unexpected character: &")
                }
            }
            '|' => {
                self.advance();
                if self.peek() == '|' {
                    self.advance();
                    Token::Or
                } else {
                    panic!("Unexpected character: |")
                }
            }
            '"' => self.read_string(),
            ch if ch.is_ascii_digit() => self.read_number(),
            ch if ch.is_alphabetic() || ch == '_' => {
                let ident = self.read_identifier();
                match ident.as_str() {
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
                }
            }
            ch => panic!("Unexpected character: {} at line {}, col {}", ch, self.line, self.col),
        }
    }
}

fn main() {
    let input = r#"
        fn main() {
            let x = 42;
            let y = "hello";
        }
    "#;
    
    let mut lexer = Lexer::new(input);
    loop {
        let tok = lexer.next_token();
        println!("{:?}", tok);
        if tok == Token::EOF { break; }
    }
}
