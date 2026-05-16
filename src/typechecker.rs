use crate::ast::{Decl, Expr, Function, Stmt, Type, BinOp, UnOp};

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

    pub fn check_program(&mut self, decls: &[Decl]) {
        for decl in decls {
            match decl {
                Decl::Function(f) => self.check_function(f),
                Decl::Target(_) => {},
                Decl::Import(_) => {},
            }
        }
    }

    fn check_function(&mut self, func: &Function) {
        self.enter_scope();
        for (name, ty) in &func.params {
            self.define(name.clone(), ty.clone());
        }
        for stmt in &func.body {
            self.check_stmt(stmt);
        }
        // TODO: check return type matches func.ret_type
        self.exit_scope();
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(name, ty_hint, expr) => {
                let expr_ty = self.check_expr(expr);
                let final_ty = if let Some(hint) = ty_hint {
                    if !self.types_match(hint, &expr_ty) {
                        self.errors.push(format!("Type mismatch in let {}: expected {:?}, got {:?}", name, hint, expr_ty));
                    }
                    hint.clone()
                } else {
                    expr_ty
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
                let cond_ty = self.check_expr(cond);
                if cond_ty != Type::Bool {
                    self.errors.push(format!("If condition must be bool, got {:?}", cond_ty));
                }
                self.enter_scope();
                for s in then_block {
                    self.check_stmt(s);
                }
                self.exit_scope();
                if let Some(eb) = else_block {
                    self.enter_scope();
                    for s in eb {
                        self.check_stmt(s);
                    }
                    self.exit_scope();
                }
            }
            Stmt::While(cond, body) => {
                let cond_ty = self.check_expr(cond);
                if cond_ty != Type::Bool {
                    self.errors.push(format!("While condition must be bool, got {:?}", cond_ty));
                }
                self.enter_scope();
                for s in body {
                    self.check_stmt(s);
                }
                self.exit_scope();
            }
            Stmt::Block(stmts) => {
                self.enter_scope();
                for s in stmts {
                    self.check_stmt(s);
                }
                self.exit_scope();
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
            Expr::Binary(left, op, right) => {
                let left_ty = self.check_expr(left);
                let right_ty = self.check_expr(right);
                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                        if self.is_numeric(&left_ty) && self.is_numeric(&right_ty) {
                            left_ty
                        } else {
                            self.errors.push(format!("Arithmetic on non-numeric: {:?} {:?}", left_ty, right_ty));
                            Type::Unknown
                        }
                    }
                    BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Gt | BinOp::Lte | BinOp::Gte => Type::Bool,
                    BinOp::And | BinOp::Or => {
                        if left_ty == Type::Bool && right_ty == Type::Bool {
                            Type::Bool
                        } else {
                            self.errors.push(format!("Logical op on non-bool: {:?} {:?}", left_ty, right_ty));
                            Type::Unknown
                        }
                    }
                }
            }
            Expr::Unary(_, e) => self.check_expr(e),
            Expr::Call(_, args) => {
                for arg in args {
                    self.check_expr(arg);
                }
                Type::Unknown // TODO: lookup function signature
            }
            Expr::Assign(name, expr) => {
                let ty = self.check_expr(expr);
                if let Some(existing) = self.lookup(name) {
                    if !self.types_match(&existing, &ty) {
                        self.errors.push(format!("Assign type mismatch for {}: {:?} vs {:?}", name, existing, ty));
                    }
                } else {
                    self.define(name.clone(), ty.clone());
                }
                ty
            }
        }
    }

    fn types_match(&self, a: &Type, b: &Type) -> bool {
        matches!((a, b),
            (Type::I32, Type::I32) | (Type::I64, Type::I64) |
            (Type::F32, Type::F32) | (Type::F64, Type::F64) |
            (Type::Bool, Type::Bool) | (Type::String, Type::String) |
            (Type::Void, Type::Void) |
            (_, Type::Unknown) | (Type::Unknown, _)
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

    pub fn get_errors(&self) -> &[String] {
        &self.errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Stmt, Function, Decl, Type, BinOp};

    fn make_let(name: &str, expr: Expr) -> Stmt {
        Stmt::Let(name.to_string(), None, Box::new(expr))
    }

    #[test]
    fn test_valid_function() {
        let func = Function {
            name: "main".to_string(),
            params: vec![],
            ret_type: Type::Void,
            body: vec![make_let("x", Expr::Integer(42))],
        };
        let mut tc = TypeChecker::new();
        tc.check_function(&func);
        assert!(!tc.has_errors());
    }

    #[test]
    fn test_undefined_variable() {
        let func = Function {
            name: "main".to_string(),
            params: vec![],
            ret_type: Type::Void,
            body: vec![Stmt::Expr(Box::new(Expr::Identifier("y".to_string())))],
        };
        let mut tc = TypeChecker::new();
        tc.check_function(&func);
        assert!(tc.has_errors());
    }

    #[test]
    fn test_type_mismatch() {
        let func = Function {
            name: "main".to_string(),
            params: vec![],
            ret_type: Type::Void,
            body: vec![Stmt::Let("x".to_string(), Some(Type::I32), Box::new(Expr::Bool(true)))],
        };
        let mut tc = TypeChecker::new();
        tc.check_function(&func);
        assert!(tc.has_errors());
    }

    #[test]
    fn test_nested_scope() {
        let func = Function {
            name: "main".to_string(),
            params: vec![],
            ret_type: Type::Void,
            body: vec![
                make_let("x", Expr::Integer(1)),
                Stmt::Block(vec![make_let("x", Expr::Integer(2))]),
            ],
        };
        let mut tc = TypeChecker::new();
        tc.check_function(&func);
        assert!(!tc.has_errors());
    }
}