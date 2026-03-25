use crate::parser::{Decl, Expr, Function, Stmt, Target};

#[derive(Debug, Clone)]
pub enum Type {
    I32,
    I64,
    F32,
    F64,
    Bool,
    String,
    Void,
    Unknown,
}

pub struct TypeChecker {
    scopes: Vec<std::collections::HashMap<String, Type>>,
    errors: Vec<String>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            scopes: vec![std::collections::HashMap::new()],
            errors: Vec::new(),
        }
    }
    
    fn enter_scope(&mut self) {
        self.scopes.push(std::collections::HashMap::new());
    }
    
    fn exit_scope(&mut self) {
        self.scopes.pop();
    }
    
    fn define(&mut self, name: String, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, ty);
        }
    }
    
    fn lookup(&self, name: &str) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty.clone());
            }
        }
        None
    }
    
    fn string_to_type(&self, s: &str) -> Type {
        match s {
            "i32" => Type::I32,
            "i64" => Type::I64,
            "f32" => Type::F32,
            "f64" => Type::F64,
            "bool" => Type::Bool,
            "string" => Type::String,
            "void" => Type::Void,
            _ => Type::Unknown,
        }
    }
    
    pub fn check_program(&mut self, decls: &[Decl]) {
        for decl in decls {
            match decl {
                Decl::Function(f) => self.check_function(f),
                Decl::Target(_) => {}
            }
        }
    }
    
    fn check_function(&mut self, func: &Function) {
        self.enter_scope();
        
        for (name, ty_str) in &func.params {
            let ty = self.string_to_type(ty_str);
            self.define(name.clone(), ty);
        }
        
        for stmt in &func.body {
            self.check_stmt(stmt);
        }
        
        self.exit_scope();
    }
    
    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(name, ty_hint, expr) => {
                let expr_ty = self.check_expr(expr);
                let final_ty = match ty_hint {
                    Some(hint) => {
                        let hint_ty = self.string_to_type(hint);
                        if !self.types_match(&hint_ty, &expr_ty) {
                            self.errors.push(format!(
                                "Type mismatch: expected {:?}, got {:?}",
                                hint_ty, expr_ty
                            ));
                        }
                        hint_ty
                    }
                    None => expr_ty,
                };
                self.define(name.clone(), final_ty);
            }
            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    self.check_expr(e);
                }
            }
            Stmt::Expr(expr) => {
                self.check_expr(expr);
            }
        }
    }
    
    fn check_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Integer(_) => Type::I32,
            Expr::Float(_) => Type::F64,
            Expr::String(_) => Type::String,
            Expr::Bool(_) => Type::Bool,
            Expr::Identifier(name) => {
                match self.lookup(name) {
                    Some(ty) => ty,
                    None => {
                        self.errors.push(format!("Undefined variable: {}", name));
                        Type::Unknown
                    }
                }
            }
            Expr::Call(name, args) => {
                for arg in args {
                    self.check_expr(arg);
                }
                Type::Unknown
            }
            Expr::Binary(left, op, right) => {
                let left_ty = self.check_expr(left);
                let right_ty = self.check_expr(right);
                
                match op.as_str() {
                    "+" | "-" | "*" | "/" => {
                        if self.is_numeric(&left_ty) && self.is_numeric(&right_ty) {
                            left_ty
                        } else {
                            self.errors.push(format!(
                                "Cannot apply '{}' to {:?} and {:?}",
                                op, left_ty, right_ty
                            ));
                            Type::Unknown
                        }
                    }
                    "==" | "!=" | "<" | ">" | "<=" | ">=" => {
                        Type::Bool
                    }
                    "&&" | "||" => {
                        if left_ty == Type::Bool && right_ty == Type::Bool {
                            Type::Bool
                        } else {
                            self.errors.push(format!(
                                "Logical operators require bool, got {:?} and {:?}",
                                left_ty, right_ty
                            ));
                            Type::Unknown
                        }
                    }
                    _ => Type::Unknown,
                }
            }
        }
    }
    
    fn types_match(&self, a: &Type, b: &Type) -> bool {
        matches!((a, b), 
            (Type::I32, Type::I32) |
            (Type::I64, Type::I64) |
            (Type::F32, Type::F32) |
            (Type::F64, Type::F64) |
            (Type::Bool, Type::Bool) |
            (Type::String, Type::String) |
            (Type::Void, Type::Void) |
            (_, Type::Unknown) |
            (Type::Unknown, _)
        )
    }
    
    fn is_numeric(&self, ty: &Type) -> bool {
        matches!(ty, Type::I32 | Type::I64 | Type::F32 | Type::F64)
    }
    
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    pub fn report_errors(&self) {
        for err in &self.errors {
            eprintln!("Type error: {}", err);
        }
    }
}
