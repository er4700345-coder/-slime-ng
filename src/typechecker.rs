use crate::ast::{Decl, Expr, Function, Stmt};

#[derive(Debug, Clone, PartialEq)]
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
                Decl::Import(_) => {}
            }
        }
    }

    fn check_function(&mut self, func: &Function) {
        self.enter_scope();

        for (name, ty) in &func.params {
            let t = self.string_to_type(&format!("{:?}", ty).to_lowercase());
            self.define(name.clone(), t);
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
                        let hint_ty = self.string_to_type(&format!("{:?}", hint).to_lowercase());
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
            Stmt::If(cond, then_block, else_block) => {
                self.check_expr(cond);
                for s in then_block { self.check_stmt(s); }
                if let Some(eb) = else_block {
                    for s in eb { self.check_stmt(s); }
                }
            }
            Stmt::While(cond, body) => {
                self.check_expr(cond);
                for s in body { self.check_stmt(s); }
            }
            Stmt::Block(stmts) => {
                for s in stmts { self.check_stmt(s); }
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
            Expr::Call(_, args) => {
                for arg in args { self.check_expr(arg); }
                Type::Unknown
            }
            Expr::Binary(left, op, right) => {
                let left_ty = self.check_expr(left);
                let right_ty = self.check_expr(right);
                match op {
                    crate::ast::BinOp::Add | crate::ast::BinOp::Sub |
                    crate::ast::BinOp::Mul | crate::ast::BinOp::Div => {
                        if self.is_numeric(&left_ty) && self.is_numeric(&right_ty) {
                            left_ty
                        } else {
                            self.errors.push(format!(
                                "Cannot apply arithmetic to {:?} and {:?}", left_ty, right_ty
                            ));
                            Type::Unknown
                        }
                    }
                    crate::ast::BinOp::Eq | crate::ast::BinOp::Neq |
                    crate::ast::BinOp::Lt | crate::ast::BinOp::Gt |
                    crate::ast::BinOp::Lte | crate::ast::BinOp::Gte => Type::Bool,
                    crate::ast::BinOp::And | crate::ast::BinOp::Or => {
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
                }
            }
            Expr::Unary(_, expr) => self.check_expr(expr),
            Expr::Assign(name, expr) => {
                let ty = self.check_expr(expr);
                self.define(name.clone(), ty.clone());
                ty
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
```
