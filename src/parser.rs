use crate::lexer::{Lexer, Token};

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
        let current = lexer.next_token();
        Parser { lexer, current }
    }
    
    fn advance(&mut self) {
        self.current = self.lexer.next_token();
    }
    
    fn expect(&mut self, token: Token) {
        if std::mem::discriminant(&self.current) == std::mem::discriminant(&token) {
            self.advance();
        } else {
            panic!("Expected {:?}, got {:?}", token, self.current);
        }
    }
    
    fn parse_identifier(&mut self) -> String {
        match &self.current {
            Token::Identifier(s) => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => panic!("Expected identifier, got {:?}", self.current),
        }
    }
    
    fn parse_type(&mut self) -> String {
        match &self.current {
            Token::I32 => { self.advance(); "i32".to_string() }
            Token::I64 => { self.advance(); "i64".to_string() }
            Token::F32 => { self.advance(); "f32".to_string() }
            Token::F64 => { self.advance(); "f64".to_string() }
            Token::Bool => { self.advance(); "bool".to_string() }
            Token::String => { self.advance(); "string".to_string() }
            Token::Void => { self.advance(); "void".to_string() }
            Token::Identifier(s) => {
                let s = s.clone();
                self.advance();
                s
            }
            _ => panic!("Expected type, got {:?}", self.current),
        }
    }
    
    fn parse_primary(&mut self) -> Expr {
        match &self.current {
            Token::Integer(n) => {
                let n = *n;
                self.advance();
                Expr::Integer(n)
            }
            Token::Float(f) => {
                let f = *f;
                self.advance();
                Expr::Float(f)
            }
            Token::StringLit(s) => {
                let s = s.clone();
                self.advance();
                Expr::String(s)
            }
            Token::True => {
                self.advance();
                Expr::Bool(true)
            }
            Token::False => {
                self.advance();
                Expr::Bool(false)
            }
            Token::Identifier(s) => {
                let s = s.clone();
                self.advance();
                if matches!(self.current, Token::LParen) {
                    self.advance();
                    let mut args = Vec::new();
                    while !matches!(self.current, Token::RParen) {
                        args.push(self.parse_expr());
                        if matches!(self.current, Token::Comma) {
                            self.advance();
                        }
                    }
                    self.advance();
                    Expr::Call(s, args)
                } else {
                    Expr::Identifier(s)
                }
            }
            _ => panic!("Unexpected token in expression: {:?}", self.current),
        }
    }
    
    fn parse_expr(&mut self) -> Expr {
        self.parse_primary()
    }
    
    fn parse_stmt(&mut self) -> Stmt {
        match &self.current {
            Token::Let => {
                self.advance();
                let name = self.parse_identifier();
                let ty = if matches!(self.current, Token::Colon) {
                    self.advance();
                    Some(self.parse_type())
                } else {
                    None
                };
                self.expect(Token::Assign);
                let expr = self.parse_expr();
                self.expect(Token::Semicolon);
                Stmt::Let(name, ty, expr)
            }
            Token::Return => {
                self.advance();
                let expr = if !matches!(self.current, Token::Semicolon) {
                    Some(self.parse_expr())
                } else {
                    None
                };
                self.expect(Token::Semicolon);
                Stmt::Return(expr)
            }
            _ => {
                let expr = self.parse_expr();
                self.expect(Token::Semicolon);
                Stmt::Expr(expr)
            }
        }
    }
    
    fn parse_block(&mut self) -> Vec<Stmt> {
        self.expect(Token::LBrace);
        let mut stmts = Vec::new();
        while !matches!(self.current, Token::RBrace) {
            stmts.push(self.parse_stmt());
        }
        self.advance();
        stmts
    }
    
    fn parse_function(&mut self) -> Function {
        self.expect(Token::Fn);
        let name = self.parse_identifier();
        self.expect(Token::LParen);
        let mut params = Vec::new();
        while !matches!(self.current, Token::RParen) {
            let pname = self.parse_identifier();
            self.expect(Token::Colon);
            let ptype = self.parse_type();
            params.push((pname, ptype));
            if matches!(self.current, Token::Comma) {
                self.advance();
            }
        }
        self.advance();
        let ret = if matches!(self.current, Token::Arrow) {
            self.advance();
            Some(self.parse_type())
        } else {
            None
        };
        let body = self.parse_block();
        Function { name, params, ret_type: ret, body }
    }
    
    fn parse_target(&mut self) -> Target {
        self.expect(Token::Target);
        let name = self.parse_identifier();
        self.expect(Token::LBrace);
        let mut options = Vec::new();
        while !matches!(self.current, Token::RBrace) {
            let key = self.parse_identifier();
            self.expect(Token::Assign);
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
                _ => panic!("Expected literal in target option"),
            };
            options.push((key, val));
        }
        self.advance();
        Target { name, options }
    }
    
    pub fn parse(&mut self) -> Vec<Decl> {
        let mut decls = Vec::new();
        while self.current != Token::EOF {
            match &self.current {
                Token::Fn => {
                    let f = self.parse_function();
                    decls.push(Decl::Function(f));
                }
                Token::Target => {
                    let t = self.parse_target();
                    decls.push(Decl::Target(t));
                }
                _ => panic!("Unexpected token at top level: {:?}", self.current),
            }
        }
        decls
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_function() {
        let input = "fn main() { let x = 42; }";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let decls = parser.parse();
        assert_eq!(decls.len(), 1);
        match &decls[0] {
            Decl::Function(f) => {
                assert_eq!(f.name, "main");
                assert_eq!(f.params.len(), 0);
                assert_eq!(f.body.len(), 1);
            }
            _ => panic!("Expected function"),
        }
    }
}
