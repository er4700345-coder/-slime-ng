// src/native.rs
// SLIME Native Backend — compiles SLIME AST to native machine code via Cranelift.
// Produces a standalone executable for the current platform.

use crate::ast::{BinOp, Decl, Expr, Stmt, Type};
use crate::stdlib::{is_stdlib_fn, lookup_stdlib, StdlibKind};

use cranelift::prelude::*;
use cranelift_module::{DataDescription, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use std::collections::HashMap;

pub struct NativeBackend {
    module: ObjectModule,
    ctx: codegen::Context,
    func_ids: HashMap<String, cranelift_module::FuncId>,
}

impl NativeBackend {
    pub fn new() -> Self {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();
        let isa_builder = cranelift_native::builder().unwrap_or_else(|e| {
            panic!("Native ISA not supported: {}", e);
        });
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .unwrap();

        let obj_builder = ObjectBuilder::new(
            isa,
            "slime_output",
            cranelift_module::default_libcall_names(),
        )
        .unwrap();

        let module = ObjectModule::new(obj_builder);
        let ctx = module.make_context();

        NativeBackend {
            module,
            ctx,
            func_ids: HashMap::new(),
        }
    }

    pub fn compile(&mut self, decls: &[Decl]) -> Vec<u8> {
        self.declare_functions(decls);

        for decl in decls {
            if let Decl::Function(f) = decl {
                self.compile_function(f);
            }
        }

        let product = std::mem::replace(
            &mut self.module,
            ObjectModule::new(
                ObjectBuilder::new(
                    cranelift_native::builder()
                        .unwrap()
                        .finish(settings::Flags::new(settings::builder()))
                        .unwrap(),
                    "slime_tmp",
                    cranelift_module::default_libcall_names(),
                )
                .unwrap(),
            ),
        );
        product.finish().emit().unwrap()
    }

    fn slime_type_to_cranelift(ty: &Type) -> types::Type {
        match ty {
            Type::I32 | Type::Bool => types::I32,
            Type::I64 => types::I64,
            Type::F32 => types::F32,
            Type::F64 => types::F64,
            _ => types::I32,
        }
    }

    fn declare_functions(&mut self, decls: &[Decl]) {
        for decl in decls {
            if let Decl::Function(f) = decl {
                let mut sig = self.module.make_signature();

                for (_, ty) in &f.params {
                    sig.params.push(AbiParam::new(Self::slime_type_to_cranelift(ty)));
                }

                if f.ret_type != Type::Void {
                    sig.returns.push(AbiParam::new(
                        Self::slime_type_to_cranelift(&f.ret_type),
                    ));
                }

                let linkage = if f.name == "main" {
                    Linkage::Export
                } else {
                    Linkage::Local
                };

                let func_id = self
                    .module
                    .declare_function(&f.name, linkage, &sig)
                    .unwrap();

                self.func_ids.insert(f.name.clone(), func_id);
            }
        }
    }

    fn compile_function(&mut self, func: &crate::ast::Function) {
        let func_id = *self.func_ids.get(&func.name).unwrap();

        let mut sig = self.module.make_signature();
        for (_, ty) in &func.params {
            sig.params.push(AbiParam::new(Self::slime_type_to_cranelift(ty)));
        }
        if func.ret_type != Type::Void {
            sig.returns
                .push(AbiParam::new(Self::slime_type_to_cranelift(&func.ret_type)));
        }

        self.ctx.func.signature = sig;
        self.ctx.func.name = cranelift::codegen::ir::UserFuncName::user(0, func_id.as_u32());

        {
            let mut builder_ctx = FunctionBuilderContext::new();
            let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut builder_ctx);

            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block);

            let mut vars: HashMap<String, Variable> = HashMap::new();
            let mut var_idx = 0usize;

            for (i, (name, ty)) in func.params.iter().enumerate() {
                let var = Variable::new(var_idx);
                var_idx += 1;
                let cl_ty = Self::slime_type_to_cranelift(ty);
                builder.declare_var(var, cl_ty);
                let param_val = builder.block_params(entry_block)[i];
                builder.def_var(var, param_val);
                vars.insert(name.clone(), var);
            }

            let mut compiler = FunctionCompiler {
                builder: &mut builder,
                module: &mut self.module,
                func_ids: &self.func_ids,
                vars,
                var_idx,
            };

            let mut returned = false;
            for stmt in &func.body {
                if compiler.compile_stmt(stmt) {
                    returned = true;
                    break;
                }
            }

            if !returned && func.ret_type == Type::Void {
                compiler.builder.ins().return_(&[]);
            }

            compiler.builder.finalize();
        }

        self.module.define_function(func_id, &mut self.ctx).unwrap();
        self.module.clear_context(&mut self.ctx);
    }
}

struct FunctionCompiler<'a> {
    builder: &'a mut FunctionBuilder<'a>,
    module: &'a mut ObjectModule,
    func_ids: &'a HashMap<String, cranelift_module::FuncId>,
    vars: HashMap<String, Variable>,
    var_idx: usize,
}

impl<'a> FunctionCompiler<'a> {
    fn compile_stmt(&mut self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Let(name, ty, expr) => {
                let val = self.compile_expr(expr);
                let cl_ty = ty.as_ref()
                    .map(|t| NativeBackend::slime_type_to_cranelift(t))
                    .unwrap_or(types::I32);
                let var = Variable::new(self.var_idx);
                self.var_idx += 1;
                self.builder.declare_var(var, cl_ty);
                self.builder.def_var(var, val);
                self.vars.insert(name.clone(), var);
                false
            }

            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    let val = self.compile_expr(e);
                    self.builder.ins().return_(&[val]);
                } else {
                    self.builder.ins().return_(&[]);
                }
                true
            }

            Stmt::Expr(expr) => {
                self.compile_expr(expr);
                false
            }

            Stmt::If(cond, then_body, else_body) => {
                let cond_val = self.compile_expr(cond);
                let then_block = self.builder.create_block();
                let else_block = self.builder.create_block();
                let merge_block = self.builder.create_block();

                self.builder.ins().brif(cond_val, then_block, &[], else_block, &[]);

                self.builder.switch_to_block(then_block);
                self.builder.seal_block(then_block);
                let mut then_returned = false;
                for s in then_body {
                    if self.compile_stmt(s) {
                        then_returned = true;
                        break;
                    }
                }
                if !then_returned {
                    self.builder.ins().jump(merge_block, &[]);
                }

                self.builder.switch_to_block(else_block);
                self.builder.seal_block(else_block);
                let mut else_returned = false;
                if let Some(else_stmts) = else_body {
                    for s in else_stmts {
                        if self.compile_stmt(s) {
                            else_returned = true;
                            break;
                        }
                    }
                }
                if !else_returned {
                    self.builder.ins().jump(merge_block, &[]);
                }

                self.builder.switch_to_block(merge_block);
                self.builder.seal_block(merge_block);
                false
            }

            Stmt::While(cond, body) => {
                let header_block = self.builder.create_block();
                let body_block = self.builder.create_block();
                let exit_block = self.builder.create_block();

                self.builder.ins().jump(header_block, &[]);

                self.builder.switch_to_block(header_block);
                let cond_val = self.compile_expr(cond);
                self.builder.ins().brif(cond_val, body_block, &[], exit_block, &[]);

                self.builder.switch_to_block(body_block);
                self.builder.seal_block(body_block);
                for s in body {
                    self.compile_stmt(s);
                }
                self.builder.ins().jump(header_block, &[]);
                self.builder.seal_block(header_block);

                self.builder.switch_to_block(exit_block);
                self.builder.seal_block(exit_block);
                false
            }

            _ => false,
        }
    }

    fn compile_expr(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Integer(n) => self.builder.ins().iconst(types::I32, *n as i64),

            Expr::Float(f) => self.builder.ins().f64const(*f),

            Expr::Bool(b) => self.builder.ins().iconst(types::I32, if *b { 1 } else { 0 }),

            Expr::Identifier(name) => {
                if let Some(&var) = self.vars.get(name) {
                    self.builder.use_var(var)
                } else {
                    self.builder.ins().iconst(types::I32, 0)
                }
            }

            Expr::Binary(left, op, right) => {
                let lv = self.compile_expr(left);
                let rv = self.compile_expr(right);
                match op {
                    BinOp::Add => self.builder.ins().iadd(lv, rv),
                    BinOp::Sub => self.builder.ins().isub(lv, rv),
                    BinOp::Mul => self.builder.ins().imul(lv, rv),
                    BinOp::Div => self.builder.ins().sdiv(lv, rv),
                    BinOp::Eq  => {
                        let r = self.builder.ins().icmp(IntCC::Equal, lv, rv);
                        self.builder.ins().uextend(types::I32, r)
                    }
                    BinOp::Neq => {
                        let r = self.builder.ins().icmp(IntCC::NotEqual, lv, rv);
                        self.builder.ins().uextend(types::I32, r)
                    }
                    BinOp::Lt  => {
                        let r = self.builder.ins().icmp(IntCC::SignedLessThan, lv, rv);
                        self.builder.ins().uextend(types::I32, r)
                    }
                    BinOp::Gt  => {
                        let r = self.builder.ins().icmp(IntCC::SignedGreaterThan, lv, rv);
                        self.builder.ins().uextend(types::I32, r)
                    }
                    BinOp::Le  => {
                        let r = self.builder.ins().icmp(IntCC::SignedLessThanOrEqual, lv, rv);
                        self.builder.ins().uextend(types::I32, r)
                    }
                    BinOp::Ge  => {
                        let r = self.builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, lv, rv);
                        self.builder.ins().uextend(types::I32, r)
                    }
                    _ => self.builder.ins().iadd(lv, rv),
                }
            }

            Expr::Unary(op, operand) => {
                let v = self.compile_expr(operand);
                match op {
                    UnOp::Neg => self.builder.ins().ineg(v),
                    UnOp::Not => {
                        let one = self.builder.ins().iconst(types::I32, 1);
                        self.builder.ins().bxor(v, one)
                    }
                }
            }

            Expr::Call(name, args) => {
                let arg_vals: Vec<Value> = args.iter()
                    .map(|a| self.compile_expr(a))
                    .collect();

                if let Some(func_id) = self.func_ids.get(name) {
                    let func_ref = self.module
                        .declare_func_in_func(*func_id, self.builder.func);
                    let call = self.builder.ins().call(func_ref, &arg_vals);
                    let results = self.builder.inst_results(call);
                    if results.is_empty() {
                        self.builder.ins().iconst(types::I32, 0)
                    } else {
                        results[0]
                    }
                } else {
                    self.builder.ins().iconst(types::I32, 0)
                }
            }

            _ => self.builder.ins().iconst(types::I32, 0),
        }
    }
}
```
