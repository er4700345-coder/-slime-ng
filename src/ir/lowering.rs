use crate::ast::{self, Decl, Expr, Function, Stmt, Type};
use crate::ir::types::{self, BasicBlock, Function as IrFunction, Instruction, IrType, Program, Value};

pub struct LoweringContext {
    next_value_id: u32,
    var_map: std::collections::HashMap<String, Value>,
}

impl LoweringContext {
    pub fn new() -> Self {
        LoweringContext {
            next_value_id: 0,
            var_map: std::collections::HashMap::new(),
        }
    }

    fn fresh_value(&mut self, ty: IrType) -> Value {
        let id = self.next_value_id;
        self.next_value_id += 1;
        Value { id, ty }
    }

    pub fn lower_program(&mut self, decls: &[Decl]) -> Result<Program, String> {
        let mut functions = Vec::new();
        let mut entry = None;
        for decl in decls {
            if let Decl::Function(f) = decl {
                let ir_fn = self.lower_function(f)?;
                if entry.is_none() {
                    entry = Some(ir_fn.name.clone());
                }
                functions.push(ir_fn);
            }
        }
        Ok(Program { functions, entry })
    }

    fn lower_function(&mut self, f: &Function) -> Result<IrFunction, String> {
        self.var_map.clear();
        let mut params = Vec::new();
        for (name, ty) in &f.params {
            let ir_ty = self.ast_type_to_ir(ty);
            let val = self.fresh_value(ir_ty);
            self.var_map.insert(name.clone(), val.clone());
            params.push((name.clone(), ir_ty));
        }
        let ret_ty = self.ast_type_to_ir(&f.ret_type);
        let mut body = vec![BasicBlock { id: 0, instructions: vec![] }];
        for stmt in &f.body {
            self.lower_stmt(stmt, &mut body[0])?;
        }
        Ok(IrFunction {
            name: f.name.clone(),
            params,
            ret_type: ret_ty,
            body,
        })
    }

    fn lower_stmt(&mut self, stmt: &Stmt, block: &mut BasicBlock) -> Result<(), String> {
        match stmt {
            Stmt::Let(name, Some(ty), expr) => {
                let val = self.lower_expr(expr, block)?;
                self.var_map.insert(name.clone(), val);
            }
            Stmt::Return(Some(expr)) => {
                let val = self.lower_expr(expr, block)?;
                block.instructions.push(Instruction::Return(Some(val)));
            }
            Stmt::Return(None) => {
                block.instructions.push(Instruction::Return(None));
            }
            Stmt::Expr(expr) => {
                let _ = self.lower_expr(expr, block)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn lower_expr(&mut self, expr: &Expr, block: &mut BasicBlock) -> Result<Value, String> {
        match expr {
            Expr::Integer(n) => {
                let val = self.fresh_value(IrType::I32);
                block.instructions.push(Instruction::Literal(val.clone(), format!("{}", n)));
                Ok(val)
            }
            Expr::Identifier(name) => {
                if let Some(val) = self.var_map.get(name) {
                    Ok(val.clone())
                } else {
                    Err(format!("Unknown variable: {}", name))
                }
            }
            _ => Err("Unsupported expr".to_string()),
        }
    }

    fn ast_type_to_ir(&self, ty: &Type) -> IrType {
        match ty {
            Type::I32 => IrType::I32,
            Type::I64 => IrType::I64,
            Type::F64 => IrType::F64,
            Type::Bool => IrType::Bool,
            Type::String => IrType::String,
            Type::Void => IrType::Void,
            _ => IrType::Unknown,
        }
    }
}