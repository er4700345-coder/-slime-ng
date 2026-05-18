use crate::ast::{self, Decl, Expr, Function, Literal, Stmt, Target, Type};
use crate::err::SlimeError;
use crate::lexer::{Lexer, Token, SourceLocation};

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

    fn expect(&mut self, expected: Token) -> Result<(), SlimeError> {
        if self.current == expected {
            self.advance();
            Ok(())
        } else {
            Err(SlimeError::ParseError {
                msg: format!("Expected {:?}, got {:?}", expected, self.current),
                loc: SourceLocation { line: self.lexer.line(), col: self.lexer.col() },
            })
        }
    }

    fn parse_type(&mut self) -> Result<Type, SlimeError> {
        match &self.current {
            Token::I32 => { self.advance(); Ok(Type::I32) }
            Token::I64 => { self.advance(); Ok(Type::I64) }
            Token::F64 => { self.advance(); Ok(Type::F64) }
            Token::Bool => { self.advance(); Ok(Type::Bool) }
            Token::String => { self.advance(); Ok(Type::String) }
            _ => Err(SlimeError::ParseError { msg: "Expected type".to_string(), loc: SourceLocation { line: 0, col: 0 } }),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, SlimeError> {
        match &self.current {
            Token::Integer(n) => {
                let val = *n;
                self.advance();
                Ok(Expr::Integer(val))
            }
            Token::Identifier(name) => {
                let id = name.clone();
                self.advance();
                Ok(Expr::Identifier(id))
            }
            Token::True => { self.advance(); Ok(Expr::Bool(true)) }
            Token::False => { self.advance(); Ok(Expr::Bool(false)) }
            _ => Err(SlimeError::ParseError { msg: "Unexpected primary".to_string(), loc: SourceLocation { line: 0, col: 0 } }),
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, SlimeError> {
        self.parse_primary()
    }

    fn parse_stmt(&mut self) -> Result<Stmt, SlimeError> {
        match &self.current {
            Token::Let => {
                self.advance();
                let name = if let Token::Identifier(n) = &self.current { n.clone() } else { return Err(SlimeError::ParseError { msg: "Expected identifier".to_string(), loc: SourceLocation { line: 0, col: 0 } }); };
                self.advance();
                self.expect(Token::Colon)?;
                let ty = self.parse_type()?;
                self.expect(Token::Equals)?;
                let expr = self.parse_expr()?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Let(name, Some(format!("{:?}", ty)), expr))
            }
            Token::Return => {
                self.advance();
                let expr = if self.current == Token::Semicolon { None } else { Some(self.parse_expr()?) };
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
        while self.current != Token::RBrace && self.current != Token::EOF {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(Token::RBrace)?;
        Ok(stmts)
    }

    fn parse_function(&mut self) -> Result<Function, SlimeError> {
        self.expect(Token::Fn)?;
        let name = if let Token::Identifier(n) = &self.current { n.clone() } else { return Err(SlimeError::ParseError { msg: "Expected fn name".to_string(), loc: SourceLocation { line: 0, col: 0 } }); };
        self.advance();
        self.expect(Token::LParen)?;
        let mut params = Vec::new();
        if self.current != Token::RParen {
            // simple single param for hello.slime
            if let Token::Identifier(pname) = &self.current {
                params.push((pname.clone(), Type::I32));
            }
            self.advance();
        }
        self.expect(Token::RParen)?;
        self.expect(Token::Arrow)?;
        let ret_type = self.parse_type()?;
        let body = self.parse_block()?;
        Ok(Function {
            name,
            params,
            ret_type: Some(ret_type),
            body,
        })
    }

    fn parse_target(&mut self) -> Result<Target, SlimeError> {
        self.expect(Token::Target)?;
        let name = if let Token::Identifier(n) = &self.current { n.clone() } else { return Err(SlimeError::ParseError { msg: "Expected target name".to_string(), loc: SourceLocation { line: 0, col: 0 } }); };
        self.advance();
        // minimal target
        Ok(Target { name, options: vec![] })
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
                    return Err(SlimeError::ParseError {
                        msg: format!("Unexpected token at top level: {:?}", self.current),
                        loc: SourceLocation { line: self.lexer.line(), col: self.lexer.col() },
                    });
                }
            }
        }
        Ok(decls)
    }
}