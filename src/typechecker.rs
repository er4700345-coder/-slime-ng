use crate::ast::{Decl, Expr, Function, Stmt, Type, BinOp};
use std::collections::HashMap;

pub struct TypeChecker {
    scopes: Vec<HashMap<String, Type>>,
    functions: HashMap<String, (Vec<Type>, Type)>,
    errors: Vec<String>,
    current_ret: Option<Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
            errors: Vec::new(),
            current_ret: None,
        }
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
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
        // First pass: collect function signatures
        for decl in decls {
            if let Decl::Function(f) = decl {
                let param_types: Vec<Type> = f.params.iter().map(|(_, t)| t.clone()).collect();
                self.functions.insert(f.name.clone(), (param_types, f.ret_type.clone()));
            }
        }
        // Second pass: check
        for decl in decls {
            if let Decl::Function(f) = decl {
                self.check_function(f);
            }
        }
    }

    fn check_function(&mut self, func: &Function) {
        self.enter_scope();
        for (name, ty) in &func.params {
            self.define(name.clone(), ty.clone());
        }
        self.current_ret = Some(func.ret_type.clone());
        let mut has_return = false;
        for stmt in &func.body {
            if let Stmt::Return(_) = stmt {
                has_return = true;
            }
            self.check_stmt(stmt);
        }
        if func.ret_type != Type::Void && !has_return {
            self.errors.push(format!("Function {} missing return of type {:?}", func.name, func.ret_type));
        }
        self.current_ret = None;
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
                if let Some(expected) = &self.current_ret {
                    if let Some(e) = expr {
                        let ret_ty = self.check_expr(e);
                        if !self.types_match(expected, &ret_ty) {
                            self.errors.push(format!("Return type mismatch: expected {:?}, got {:?}", expected, ret_ty));
                        }
                    } else if *expected != Type::Void {
                        self.errors.push(format!("Missing return expression for non-void function"));
                    }
                }
            }
            Stmt::Expr(expr) => { self.check_expr(expr); }
            Stmt::If(cond, then_block, else_block) => {
                let cond_ty = self.check_expr(cond);
                if cond_ty != Type::Bool { self.errors.push(format!("If condition must be bool")); }
                self.enter_scope(); for s in then_block { self.check_stmt(s); } self.exit_scope();
                if let Some(eb) = else_block { self.enter_scope(); for s in eb { self.check_stmt(s); } self.exit_scope(); }
            }
            Stmt::While(cond, body) => {
                let cond_ty = self.check_expr(cond);
                if cond_ty != Type::Bool { self.errors.push(format!("While condition must be bool")); }
                self.enter_scope(); for s in body { self.check_stmt(s); } self.exit_scope();
            }
            Stmt::Block(stmts) => { self.enter_scope(); for s in stmts { self.check_stmt(s); } self.exit_scope(); }
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Integer(_) => Type::I32,
            Expr::Float(_) => Type::F64,
            Expr::String(_) => Type::String,
            Expr::Bool(_) => Type::Bool,
            Expr::Identifier(name) => self.lookup(name).unwrap_or_else(|| { self.errors.push(format!("Undefined: {}", name)); Type::Unknown }),
            Expr::Binary(left, op, right) => {
                let l = self.check_expr(left); let r = self.check_expr(right);
                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => if self.is_numeric(&l) && self.is_numeric(&r) { l } else { self.errors.push("Arithmetic type error".to_string()); Type::Unknown },
                    _ => Type::Bool,
                }
            }
            Expr::Unary(_, e) => self.check_expr(e),
            Expr::Call(name, args) => {
                if let Some((param_tys, ret_ty)) = self.functions.get(name) {
                    if args.len() != param_tys.len() {
                        self.errors.push(format!("{} expects {} args, got {}", name, param_tys.len(), args.len()));
                    }
                    for (arg, pty) in args.iter().zip(param_tys) {
                        let aty = self.check_expr(arg);
                        if !self.types_match(pty, &aty) { self.errors.push(format!("Arg type mismatch in call to {}", name)); }
                    }
                    ret_ty.clone()
                } else {
                    self.errors.push(format!("Undefined function: {}", name));
                    Type::Unknown
                }
            }
            Expr::Assign(name, expr) => {
                let ty = self.check_expr(expr);
                if let Some(existing) = self.lookup(name) {
                    if !self.types_match(&existing, &ty) { self.errors.push(format!("Assign mismatch for {}", name)); }
                } else { self.define(name.clone(), ty.clone()); }
                ty
            }
        }
    }

    fn types_match(&self, a: &Type, b: &Type) -> bool {
        matches!((a, b), (Type::I32, Type::I32) | (Type::I64, Type::I64) | (Type::F32, Type::F32) | (Type::F64, Type::F64) | (Type::Bool, Type::Bool) | (Type::String, Type::String) | (Type::Void, Type::Void) | (_, Type::Unknown) | (Type::Unknown, _))
    }

    fn is_numeric(&self, ty: &Type) -> bool { matches!(ty, Type::I32 | Type::I64 | Type::F32 | Type::F64) }

    pub fn has_errors(&self) -> bool { !self.errors.is_empty() }
    pub fn get_errors(&self) -> &[String] { &self.errors }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Stmt, Function, Type};

    fn make_let(name: &str, expr: Expr) -> Stmt { Stmt::Let(name.to_string(), None, Box::new(expr)) }
    fn make_return(expr: Expr) -> Stmt { Stmt::Return(Some(Box::new(expr))) }

    #[test] fn test_valid_return() {
        let func = Function { name: "main".into(), params: vec![], ret_type: Type::I32, body: vec![make_return(Expr::Integer(42))] };
        let mut tc = TypeChecker::new(); tc.check_function(&func); assert!(!tc.has_errors());
    }
    #[test] fn test_invalid_return() {
        let func = Function { name: "main".into(), params: vec![], ret_type: Type::I32, body: vec![make_return(Expr::Bool(true))] };
        let mut tc = TypeChecker::new(); tc.check_function(&func); assert!(tc.has_errors());
    }
    #[test] fn test_missing_return() {
        let func = Function { name: "main".into(), params: vec![], ret_type: Type::I32, body: vec![make_let("x", Expr::Integer(1))] };
        let mut tc = TypeChecker::new(); tc.check_function(&func); assert!(tc.has_errors());
    }
    #[test] fn test_valid_call() {
        let f = Function { name: "add".into(), params: vec![("a".into(), Type::I32), ("b".into(), Type::I32)], ret_type: Type::I32, body: vec![] };
        let mut tc = TypeChecker::new(); tc.functions.insert("add".into(), (vec![Type::I32, Type::I32], Type::I32));
        let call = Expr::Call("add".into(), vec![Expr::Integer(1), Expr::Integer(2)]);
        assert_eq!(tc.check_expr(&call), Type::I32); assert!(!tc.has_errors());
    }
    #[test] fn test_undefined_call() {
        let mut tc = TypeChecker::new(); let call = Expr::Call("foo".into(), vec![]); tc.check_expr(&call); assert!(tc.has_errors());
    }
    #[test] fn test_wrong_arg_count() {
        let mut tc = TypeChecker::new(); tc.functions.insert("add".into(), (vec![Type::I32, Type::I32], Type::I32));
        let call = Expr::Call("add".into(), vec![Expr::Integer(1)]); tc.check_expr(&call); assert!(tc.has_errors());
    }
    #[test] fn test_wrong_arg_type() {
        let mut tc = TypeChecker::new(); tc.functions.insert("add".into(), (vec![Type::I32, Type::I32], Type::I32));
        let call = Expr::Call("add".into(), vec![Expr::Integer(1), Expr::Bool(true)]); tc.check_expr(&call); assert!(tc.has_errors());
    }
}