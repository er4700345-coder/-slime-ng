use crate::ast::{Decl, Expr, Function, Stmt, Type, BinOp, UnOp};

pub struct WasmBinaryBackend {
    bytes: Vec<u8>,
    functions: Vec<(String, u32, u32)>,
    types: Vec<(Vec<u8>, Vec<u8>)>,
    locals: Vec<u8>,
}

impl WasmBinaryBackend {
    pub fn new() -> Self {
        WasmBinaryBackend {
            bytes: Vec::new(),
            functions: Vec::new(),
            types: Vec::new(),
            locals: Vec::new(),
        }
    }
    
    pub fn compile(&mut self, decls: &[Decl]) -> Vec<u8> {
        self.bytes.clear();
        self.functions.clear();
        self.types.clear();
        
        self.emit_magic();
        self.emit_version();
        
        self.collect_types(decls);
        self.emit_type_section();
        
        self.emit_function_section();
        self.emit_export_section();
        self.emit_code_section(decls);
        
        self.bytes.clone()
    }
    
    fn emit_magic(&mut self) {
        self.bytes.extend_from_slice(&[0x00, 0x61, 0x73, 0x6d]);
    }
    
    fn emit_version(&mut self) {
        self.bytes.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
    }
    
    fn val_type(ty: &Type) -> u8 {
        match ty {
            Type::I32 => 0x7f,
            Type::I64 => 0x7e,
            Type::F32 => 0x7d,
            Type::F64 => 0x7c,
            _ => 0x7f,
        }
    }
    
    fn collect_types(&mut self, decls: &[Decl]) {
        for decl in decls {
            if let Decl::Function(f) = decl {
                let params: Vec<u8> = f.params.iter()
                    .map(|(_, t)| Self::val_type(t))
                    .collect();
                let results: Vec<u8> = if f.ret_type != Type::Void {
                    vec![Self::val_type(&f.ret_type)]
                } else {
                    vec![]
                };
                self.types.push((params, results));
            }
        }
    }
    
    fn emit_type_section(&mut self) {
        if self.types.is_empty() {
            return;
        }
        
        let mut section = Vec::new();
        section.push(self.types.len() as u8);
        
        for (params, results) in &self.types {
            section.push(0x60);
            section.push(params.len() as u8);
            section.extend(params);
            section.push(results.len() as u8);
            section.extend(results);
        }
        
        self.bytes.push(0x01);
        self.emit_leb128(section.len() as u32);
        self.bytes.extend(section);
    }
    
    fn emit_function_section(&mut self) {
        let func_count = self.types.len();
        if func_count == 0 {
            return;
        }
        
        let mut section = Vec::new();
        section.push(func_count as u8);
        for i in 0..func_count {
            section.push(i as u8);
        }
        
        self.bytes.push(0x03);
        self.emit_leb128(section.len() as u32);
        self.bytes.extend(section);
    }
    
    fn emit_export_section(&mut self) {
        if self.functions.is_empty() {
            let mut section = Vec::new();
            section.push(0x01);
            section.push(0x04);
            section.extend_from_slice(b"main");
            section.push(0x00);
            section.push(0x00);
            
            self.bytes.push(0x07);
            self.emit_leb128(section.len() as u32);
            self.bytes.extend(section);
        }
    }
    
    fn emit_code_section(&mut self, decls: &[Decl]) {
        let func_decls: Vec<_> = decls.iter()
            .filter_map(|d| match d {
                Decl::Function(f) => Some(f),
                _ => None,
            })
            .collect();
        
        if func_decls.is_empty() {
            return;
        }
        
        let mut section = Vec::new();
        section.push(func_decls.len() as u8);
        
        for func in func_decls {
            let mut func_body = Vec::new();
            
            func_body.push(0x00);
            
            for stmt in &func.body {
                self.compile_stmt(&mut func_body, stmt);
            }
            
            func_body.push(0x0b);
            
            self.emit_leb128(func_body.len() as u32);
            section.extend(func_body);
        }
        
        self.bytes.push(0x0a);
        self.emit_leb128(section.len() as u32);
        self.bytes.extend(section);
    }
    
    fn compile_stmt(&mut self, out: &mut Vec<u8>, stmt: &Stmt) {
        match stmt {
            Stmt::Let(name, _ty, expr) => {
                self.compile_expr(out, expr);
                out.push(0x21);
                out.push(0x00);
            }
            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    self.compile_expr(out, e);
                }
                out.push(0x0f);
            }
            Stmt::Expr(expr) => {
                self.compile_expr(out, expr);
                out.push(0x1a);
            }
            _ => {}
        }
    }
    
    fn compile_expr(&mut self, out: &mut Vec<u8>, expr: &Expr) {
        match expr {
            Expr::Integer(n) => {
                out.push(0x41);
                self.emit_leb128_signed(out, *n as i64);
            }
            Expr::Identifier(_name) => {
                out.push(0x20);
                out.push(0x00);
            }
            Expr::Binary(left, op, right) => {
                self.compile_expr(out, left);
                self.compile_expr(out, right);
                
                let opcode = match op {
                    BinOp::Add => 0x6a,
                    BinOp::Sub => 0x6b,
                    BinOp::Mul => 0x6c,
                    BinOp::Div => 0x6d,
                    BinOp::Eq => 0x46,
                    BinOp::Neq => 0x47,
                    BinOp::Lt => 0x48,
                    BinOp::Gt => 0x4a,
                    _ => 0x6a,
                };
                out.push(opcode);
            }
            Expr::Call(_name, args) => {
                for arg in args {
                    self.compile_expr(out, arg);
                }
                out.push(0x10);
                out.push(0x00);
            }
            _ => {}
        }
    }
    
    fn emit_leb128(&mut self, mut value: u32) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            self.bytes.push(byte);
            if value == 0 {
                break;
            }
        }
    }
    
    fn emit_leb128_signed(&self, out: &mut Vec<u8>, mut value: i64) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            let done = value == 0 && (byte & 0x40) == 0 
                || value == -1 && (byte & 0x40) != 0;
            if !done {
                byte |= 0x80;
            }
            out.push(byte);
            if done {
                break;
            }
        }
    }
}
