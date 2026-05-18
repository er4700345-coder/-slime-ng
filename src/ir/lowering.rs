use crate::ast::{self, Decl, Expr, Function, Stmt, Type};
use crate::ir::IR;

pub struct LoweringContext;

impl LoweringContext {
    pub fn new() -> Self { LoweringContext }

    pub fn lower_program(&mut self, decls: &[Decl]) -> Result<Vec<IR>, String> {
        let mut ir = Vec::new();
        for decl in decls {
            if let Decl::Function(f) = decl {
                ir.push(self.lower_function(f)?);
            }
        }
        Ok(ir)
    }

    fn lower_function(&self, f: &Function) -> Result<IR, String> {
        // Minimal lowering for hello.slime
        let ret = match &f.ret_type {
            Type::I32 => "i32",
            Type::I64 => "i64",
            _ => "void",
        };
        Ok(IR::Function { name: f.name.clone(), params: f.params.clone(), ret_type: ret.to_string(), body: vec![] })
    }
}