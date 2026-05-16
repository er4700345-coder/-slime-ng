use crate::lexer::{Lexer, Token};
use crate::err::{SlimeError, SourceLocation};

#[derive(Debug, Clone)]
pub enum Expr {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Identifier(String),
    Binary(Box<Expr>, String, Box<Expr>),
    Call(String, Vec<Expr>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(String, Option<String>, Expr),
    Return(Option<Expr>),
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<(String, String)>,
    pub ret_type: Option<String>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct Target {
    pub name: String,
    pub options: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub enum Decl {
    Function(Function),
    Target(Target),
}

pub struct Parser {
    lexer: Lexer,
    current: Token,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let current = lexer.next_token().unwrap_or(Token::EOF);
        Parser { lexer, current }
    }
    
    fn advance(&mut self) {
        self.current = self.lexer.next_token().unwrap_or(Token::EOF);
    }
    
    fn expect(&mut self, token: Token) -> Result<(), SlimeError> {
        if std::mem::discriminant(&self.current) == std::mem::discriminant(&token) {
            self.advance();
            Ok(())
        } else {
            let loc = SourceLocation { line: self.lexer.line(), col: self.lexer.col() };
            Err(SlimeError::ParseError { msg: format!("Expected {:?}, got {:?}", token, self.current), loc })
        }
    }
    
    fn parse_identifier(&mut self) -> Result<String, SlimeError> {
        match &self.current {
            Token::Identifier(s) => {
                let s = s.clone();
                self.advance();
                Ok(s)
            }
            _ => {
                let loc = SourceLocation { line: self.lexer.line(), col: self.lexer.col() };
                Err(SlimeError::ParseError { msg: format!("Expected identifier, got {:?}", self.current), loc })
            }
        }
    }
    
    fn parse_type(&mut self) -> Result<String, SlimeError> {
        match &self.current {
            Token::I32 => { self.advance(); Ok("i32".to_string()) }
            Token::I64 => { self.advance(); Ok("i64".to_string()) }
            Token::F32 => { self.advance(); Ok("f32".to_string()) }
            Token::F64 => { self.advance(); Ok("f64".to_string()) }
            Token::Bool => { self.advance(); Ok("bool".to_string()) }
            Token::String => { self.advance(); Ok("string".to_string()) }
            Token::Void => { self.advance(); Ok("void".to_string()) }
            Token::Identifier(s) => {
                let s = s.clone();
                self.advance();
                Ok(s)
            }
            _ => {
                let loc = SourceLocation { line: self.lexer.line(), col: self.lexer.col() };
                Err(SlimeError::ParseError { msg: format!("Expected type, got {:?}", self.current), loc })
            }
        }
    }
    
    fn parse_primary(&mut self) -> Result<Expr, SlimeError> {
        match &self.current {
            Token::Integer(n) => {
                let n = *n;
                self.advance();
                Ok(Expr::Integer(n))
            }
            Token::Float(f) => {
                let f = *f;
                self.advance();
                Ok(Expr::Float(f))
            }
            Token::StringLit(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::String(s))
            }
            Token::True => {
                self.advance();
                Ok(Expr::Bool(true))
            }
            Token::False => {
                self.advance();
                Ok(Expr::Bool(false))
            }
            Token::Identifier(s) => {
                let s = s.clone();
                self.advance();
                if matches!(self.current, Token::LParen) {
                    self.advance();
                    let mut args = Vec::new();
                    while !matches!(self.current, Token::RParen) {
                        args.push(self.parse_expr()?);
                        if matches!(self.current, Token::Comma) {
                            self.advance();
                        }
                    }
                    self.advance();
                    Ok(Expr::Call(s, args))
                } else {
                    Ok(Expr::Identifier(s))
                }
            }
            _ => {
                let loc = SourceLocation { line: self.lexer.line(), col: self.lexer.col() };
                Err(SlimeError::ParseError { msg: format!("Unexpected token in expression: {:?}", self.current), loc })
            }
        }
    }
    
    fn parse_expr(&mut self) -> Result<Expr, SlimeError> {
        self.parse_primary()
    }
    
    fn parse_stmt(&mut self) -> Result<Stmt, SlimeError> {
        match &self.current {
            Token::Let => {
                self.advance();
                let name = self.parse_identifier()?;
                let ty = if matches!(self.current, Token::Colon) {
                    self.advance();
                    Some(self.parse_type()?)
                } else {
                    None
                };
                self.expect(Token::Assign)?;
                let expr = self.parse_expr()?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Let(name, ty, expr))
            }
            Token::Return => {
                self.advance();
                let expr = if !matches!(self.current, Token::Semicolon) {
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Return(expr))
            }
            _ => {
                let expr = self.parse_expr()?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Expr(expr))
            }
        }
    }
    
    fn parse_block(&mut self) -> Result<Vec<Stmt>, SlimeError> {
        self.expect(Token::LBrace)?;
        let mut stmts = Vec::new();
        while !matches!(self.current, Token::RBrace) {
            stmts.push(self.parse_stmt()?);
        }
        self.advance();
        Ok(stmts)
    }
    
    fn parse_function(&mut self) -> Result<Function, SlimeError> {
        self.expect(Token::Fn)?;
        let name = self.parse_identifier()?;
        self.expect(Token::LParen)?;
        let mut params = Vec::new();
        while !matches!(self.current, Token::RParen) {
            let pname = self.parse_identifier()?;
            self.expect(Token::Colon)?;
            let ptype = self.parse_type()?;
            params.push((pname, ptype));
            if matches!(self.current, Token::Comma) {
                self.advance();
            }
        }
        self.advance();
        let ret = if matches!(self.current, Token::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        let body = self.parse_block()?;
        Ok(Function { name, params, ret_type: ret, body })
    }
    
    fn parse_target(&mut self) -> Result<Target, SlimeError> {
        self.expect(Token::Target)?;
        let name = self.parse_identifier()?;
        self.expect(Token::LBrace)?;
        let mut options = Vec::new();
        while !matches!(self.current, Token::RBrace) {
            let key = self.parse_identifier()?;
            self.expect(Token::Assign)?;
            let val = match &self.current {
                Token::Identifier(s) => {
                    let s = s.clone();
                    self.advance();
                    s
                }
                Token::StringLit(s) => {
                    let s = s.clone();
                    self.advance();
                    s
                }
                Token::True => {
                    self.advance();
                    "true".to_string()
                }
                Token::False => {
                    self.advance();
                    "false".to_string()
                }
                _ => {
                    let loc = SourceLocation { line: self.lexer.line(), col: self.lexer.col() };
                    return Err(SlimeError::ParseError { msg: "Expected literal in target option".to_string(), loc });
                }
            };
            options.push((key, val));
        }
        self.advance();
        Ok(Target { name, options })
    }
    
    pub fn parse(&mut self) -> Result<Vec<Decl>, SlimeError> {
        let mut decls = Vec::new();
        while self.current != Token::EOF {
            match &self.current {
                Token::Fn => {
                    let f = self.parse_function()?;
                    decls.push(Decl::Function(f));
                }
                Token::Target => {
                    let t = self.parse_target()?;
                    decls.push(Decl::Target(t));
                }
                _ => {
                    let loc = SourceLocation { line: self.lexer.line(), col: self.lexer.col() };
                    return Err(SlimeError::ParseError { msg: format!("Unexpected token at top level: {:?}", self.current), loc });
                }
            }
        }
        Ok(decls)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unexpected_token_location() {
        let input = "let x = 1;";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_delimiter_location() {
        let input = "fn main() { let x = 1 ";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_function_syntax_location() {
        let input = "fn 123() {} ";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let result = parser.parse();
        assert!(result.is_err());
    }
}