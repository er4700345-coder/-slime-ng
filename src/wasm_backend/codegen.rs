use crate::ir::types::*;
use walrus::{FunctionBuilder, InstrSeqBuilder, Module, ValType};

pub struct WasmCodegen {
    module: Module,
}

impl WasmCodegen {
    pub fn new() -> Self {
        WasmCodegen {
            module: Module::new(),
        }
    }

    pub fn lower_program(&mut self, program: &Program) -> Vec<u8> {
        for func in &program.functions {
            self.lower_function(func);
        }
        self.module.emit_wasm()
    }

    fn lower_function(&mut self, func: &Function) {
        let mut builder = FunctionBuilder::new(&mut self.module.types, &[], &[]);
        let mut body = builder.func_body();

        for instr in &func.blocks[0].instructions {
            match instr {
                Instruction::Literal(val) => {
                    if let IrType::I32 = val.ty {
                        body.i32_const(42); // placeholder
                    }
                }
                Instruction::Return(val) => {
                    body.return_();
                }
                Instruction::Binary { .. } => {
                    body.i32_add();
                }
                Instruction::Call { name, .. } => {
                    // placeholder for builtin
                    if name == "print" {
                        body.call(0); // assume print index
                    }
                }
                _ => {}
            }
        }

        let func_id = builder.finish(vec![], &mut self.module.funcs);
        self.module.exports.add(&func.name, func_id);
    }
}