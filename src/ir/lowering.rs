use crate::ast::{Decl, Expr, Function as AstFunction, Stmt, Type as AstType};
use crate::ir::types::*;

pub struct LoweringContext {
    next_value_id: usize,
    variables: std::collections::HashMap<String, Value>,
}

impl LoweringContext {
    pub fn new() -> Self {
        LoweringContext {
            next_value_id: 0,
            variables: std::collections::HashMap::new(),
        }
    }

    fn fresh_value(&mut self, ty: IrType) -> Value {
        let id = self.next_value_id;
        self.next_value_id += 1;
        Value::new(id, ty)
    }

    pub fn lower_program(&mut self, decls: &[Decl]) -> Program {
        let mut program = Program::new();
        for decl in decls {
            if let Decl::Function(f) = decl {
                let ir_func = self.lower_function(f);
                program.functions.push(ir_func);
            }
        }
        if !program.functions.is_empty() {
            program.entry = Some(program.functions[0].name.clone());
        }
        program
    }

    fn lower_function(&mut self, f: &AstFunction) -> Function {
        let mut ctx = LoweringContext::new();
        let mut block = BasicBlock { id: 0, instructions: vec![] };

        for (name, ty_str) in &f.params {
            let ir_ty = self.ast_type_to_ir(ty_str);
            let val = ctx.fresh_value(ir_ty.clone());
            ctx.variables.insert(name.clone(), val);
        }

        for stmt in &f.body {
            self.lower_stmt(stmt, &mut block, &mut ctx);
        }

        Function {
            name: f.name.clone(),
            params: f.params.iter().map(|(n, t)| (n.clone(), self.ast_type_to_ir(t))).collect(),
            ret_type: self.ast_type_to_ir(&f.ret_type.clone().unwrap_or("void".to_string())),
            blocks: vec![block],
        }
    }

    fn lower_stmt(&mut self, stmt: &Stmt, block: &mut BasicBlock, ctx: &mut LoweringContext) {
        match stmt {
            Stmt::Let(name, ty_hint, expr) => {
                let val = self.lower_expr(expr, block, ctx);
                ctx.variables.insert(name.clone(), val);
            }
            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    let val = self.lower_expr(e, block, ctx);
                    block.instructions.push(Instruction::Return(val));
                }
            }
            Stmt::Expr(expr) => {
                let _ = self.lower_expr(expr, block, ctx);
            }
            _ => {}
        }
    }

    fn lower_expr(&mut self, expr: &Expr, block: &mut BasicBlock, ctx: &mut LoweringContext) -> Value {
        match expr {
            Expr::Integer(n) => {
                let val = ctx.fresh_value(IrType::I32);
                block.instructions.push(Instruction::Literal(val.clone()));
                val
            }
            Expr::Float(f) => {
                let val = ctx.fresh_value(IrType::F64);
                block.instructions.push(Instruction::Literal(val.clone()));
                val
            }
            Expr::String(s) => {
                let val = ctx.fresh_value(IrType::String);
                block.instructions.push(Instruction::Literal(val.clone()));
                val
            }
            Expr::Bool(b) => {
                let val = ctx.fresh_value(IrType::Bool);
                block.instructions.push(Instruction::Literal(val.clone()));
                val
            }
            Expr::Identifier(name) => {
                ctx.variables.get(name).cloned().unwrap_or_else(|| ctx.fresh_value(IrType::Unknown))
            }
            Expr::Binary(lhs, op, rhs) => {
                let l = self.lower_expr(lhs, block, ctx);
                let r = self.lower_expr(rhs, block, ctx);
                let result = ctx.fresh_value(IrType::I32); // simplified
                block.instructions.push(Instruction::Binary {
                    op: op.clone(),
                    lhs: l,
                    rhs: r,
                    result: result.clone(),
                });
                result
            }
            Expr::Call(name, args) => {
                let arg_vals: Vec<Value> = args.iter().map(|a| self.lower_expr(a, block, ctx)).collect();
                let result = ctx.fresh_value(IrType::Unknown);
                block.instructions.push(Instruction::Call {
                    name: name.clone(),
                    args: arg_vals,
                    result: result.clone(),
                });
                result
            }
            _ => ctx.fresh_value(IrType::Unknown),
        }
    }

    fn ast_type_to_ir(&self, ty: &str) -> IrType {
        match ty {
            "i32" => IrType::I32,
            "i64" => IrType::I64,
            "f32" => IrType::F32,
            "f64" => IrType::F64,
            "bool" => IrType::Bool,
            "string" => IrType::String,
            "void" => IrType::Void,
            _ => IrType::Unknown,
        }
    }
}