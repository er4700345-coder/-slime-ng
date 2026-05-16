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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Stmt, Function as AstFunction, Type as AstType};

    fn make_simple_function() -> AstFunction {
        AstFunction {
            name: "main".to_string(),
            params: vec![],
            ret_type: Some("i32".to_string()),
            body: vec![Stmt::Return(Some(Box::new(Expr::Integer(42))))],
        }
    }

    #[test]
    fn test_lowering_valid_function() {
        let mut ctx = LoweringContext::new();
        let func = make_simple_function();
        let ir_func = ctx.lower_function(&func);
        assert_eq!(ir_func.name, "main");
        assert!(!ir_func.blocks.is_empty());
    }

    #[test]
    fn test_lowering_return() {
        let mut ctx = LoweringContext::new();
        let func = make_simple_function();
        let ir_func = ctx.lower_function(&func);
        let has_return = ir_func.blocks[0].instructions.iter().any(|i| matches!(i, Instruction::Return(_)));
        assert!(has_return);
    }

    #[test]
    fn test_lowering_binary() {
        let mut ctx = LoweringContext::new();
        let func = AstFunction {
            name: "add".to_string(),
            params: vec![("a".to_string(), "i32".to_string()), ("b".to_string(), "i32".to_string())],
            ret_type: Some("i32".to_string()),
            body: vec![Stmt::Return(Some(Box::new(Expr::Binary(
                Box::new(Expr::Identifier("a".to_string())),
                "+".to_string(),
                Box::new(Expr::Identifier("b".to_string())),
            ))))],
        };
        let ir_func = ctx.lower_function(&func);
        let has_binary = ir_func.blocks[0].instructions.iter().any(|i| matches!(i, Instruction::Binary { .. }));
        assert!(has_binary);
    }

    #[test]
    fn test_lowering_builtin_call() {
        let mut ctx = LoweringContext::new();
        let func = AstFunction {
            name: "main".to_string(),
            params: vec![],
            ret_type: Some("void".to_string()),
            body: vec![Stmt::Expr(Box::new(Expr::Call("print".to_string(), vec![Box::new(Expr::String("hi".to_string()))])))],
        };
        let ir_func = ctx.lower_function(&func);
        let has_call = ir_func.blocks[0].instructions.iter().any(|i| matches!(i, Instruction::Call { name, .. } if name == "print"));
        assert!(has_call);
    }
}