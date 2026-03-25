use crate::ast::{Decl, Expr, Function, Stmt, Type, BinOp, UnOp};

pub struct WasmBackend {
    output: String,
    locals: Vec<(String, i32)>,
    local_count: i32,
    func_types: Vec<(String, Vec<walrus::ValType>, Vec<walrus::ValType>)>,
}

impl WasmBackend {
    pub fn new() -> Self {
        WasmBackend {
            output: String::new(),
            locals: Vec::new(),
            local_count: 0,
            func_types: Vec::new(),
        }
    }
    
    pub fn compile(&mut self, decls: &[Decl]) -> String {
        self.output.push_str("(module\n");
        
        for decl in decls {
            match decl {
                Decl::Function(f) => self.compile_function(f),
                _ => {}
            }
        }
        
        self.output.push_str(")\n");
        self.output.clone()
    }
    
    fn type_to_wasm(&self, ty: &Type) -> &'static str {
        match ty {
            Type::I32 => "i32",
            Type::I64 => "i64",
            Type::F32 => "f32",
            Type::F64 => "f64",
            _ => "i32",
        }
    }
    
    fn compile_function(&mut self, func: &Function) {
        self.locals.clear();
        self.local_count = 0;
        
        for (name, ty) in &func.params {
            self.locals.push((name.clone(), self.local_count));
            self.local_count += 1;
        }
        
        let params: Vec<String> = func.params.iter()
            .map(|(_, ty)| format!("(param {})", self.type_to_wasm(ty)))
            .collect();
        
        let result = if func.ret_type != Type::Void {
            format!("(result {})", self.type_to_wasm(&func.ret_type))
        } else {
            String::new()
        };
        
        self.output.push_str(&format!(
            "  (func ${} {} {}\n",
            func.name,
            params.join(" "),
            result
        ));
        
        for stmt in &func.body {
            self.compile_stmt(stmt);
        }
        
        if func.ret_type == Type::Void {
            self.output.push_str("  )\n");
        } else {
            self.output.push_str("  )\n");
        }
    }
    
    fn compile_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(name, ty, expr) => {
                let local_idx = self.local_count;
                self.local_count += 1;
                self.locals.push((name.clone(), local_idx));
                
                self.output.push_str(&format!("    (local ${} {})\n", 
                    name, 
                    self.type_to_wasm(&ty.clone().unwrap_or(Type::I32))
                ));
                
                self.compile_expr(expr);
                self.output.push_str(&format!("    local.set ${}\n", name));
            }
            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    self.compile_expr(e);
                }
                self.output.push_str("    return\n");
            }
            Stmt::Expr(expr) => {
                self.compile_expr(expr);
                self.output.push_str("    drop\n");
            }
            _ => {}
        }
    }
    
    fn compile_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Integer(n) => {
                self.output.push_str(&format!("    i32.const {}\n", n));
            }
            Expr::Float(f) => {
                self.output.push_str(&format!("    f64.const {}\n", f));
            }
            Expr::Identifier(name) => {
                self.output.push_str(&format!("    local.get ${}\n", name));
            }
            Expr::Binary(left, op, right) => {
                self.compile_expr(left);
                self.compile_expr(right);
                
                let op_str = match op {
                    BinOp::Add => "i32.add",
                    BinOp::Sub => "i32.sub",
                    BinOp::Mul => "i32.mul",
                    BinOp::Div => "i32.div_s",
                    BinOp::Eq => "i32.eq",
                    BinOp::Neq => "i32.ne",
                    BinOp::Lt => "i32.lt_s",
                    BinOp::Gt => "i32.gt_s",
                    BinOp::Lte => "i32.le_s",
                    BinOp::Gte => "i32.ge_s",
                    _ => "i32.add",
                };
                
                self.output.push_str(&format!("    {}\n", op_str));
            }
            Expr::Call(name, args) => {
                for arg in args {
                    self.compile_expr(arg);
                }
                self.output.push_str(&format!("    call ${}\n", name));
            }
            _ => {}
        }
    }
    
    pub fn get_output(&self) -> &str {
        &self.output
    }
}
