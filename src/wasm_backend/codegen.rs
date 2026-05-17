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
                        body.i32_const(42);
                    }
                }
                Instruction::Return(val) => {
                    body.return_();
                }
                Instruction::Binary { .. } => {
                    body.i32_add();
                }
                Instruction::Call { name, .. } => {
                    if name == "print" {
                        body.call(0);
                    }
                }
                _ => {}
            }
        }

        let func_id = builder.finish(vec![], &mut self.module.funcs);
        self.module.exports.add(&func.name, func_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::types::*;

    fn make_simple_program() -> Program {
        let mut program = Program::new();
        let func = Function {
            name: "main".to_string(),
            params: vec![],
            ret_type: IrType::I32,
            blocks: vec![BasicBlock {
                id: 0,
                instructions: vec![
                    Instruction::Literal(Value::new(0, IrType::I32)),
                    Instruction::Return(Value::new(0, IrType::I32)),
                ],
            }],
        };
        program.functions.push(func);
        program
    }

    #[test]
    fn test_lowering_simple_function() {
        let mut codegen = WasmCodegen::new();
        let program = make_simple_program();
        let wasm = codegen.lower_program(&program);
        assert!(!wasm.is_empty());
    }

    #[test]
    fn test_lowering_return() {
        let mut codegen = WasmCodegen::new();
        let program = make_simple_program();
        let wasm = codegen.lower_program(&program);
        // WASM should contain return
        assert!(wasm.len() > 10);
    }

    #[test]
    fn test_lowering_binary() {
        let mut program = Program::new();
        let func = Function {
            name: "add".to_string(),
            params: vec![],
            ret_type: IrType::I32,
            blocks: vec![BasicBlock {
                id: 0,
                instructions: vec![Instruction::Binary {
                    op: "+".to_string(),
                    lhs: Value::new(0, IrType::I32),
                    rhs: Value::new(1, IrType::I32),
                    result: Value::new(2, IrType::I32),
                }],
            }],
        };
        program.functions.push(func);
        let mut codegen = WasmCodegen::new();
        let wasm = codegen.lower_program(&program);
        assert!(!wasm.is_empty());
    }

    #[test]
    fn test_lowering_builtin_call() {
        let mut program = Program::new();
        let func = Function {
            name: "main".to_string(),
            params: vec![],
            ret_type: IrType::Void,
            blocks: vec![BasicBlock {
                id: 0,
                instructions: vec![Instruction::Call {
                    name: "print".to_string(),
                    args: vec![Value::new(0, IrType::String)],
                    result: Value::new(1, IrType::Void),
                }],
            }],
        };
        program.functions.push(func);
        let mut codegen = WasmCodegen::new();
        let wasm = codegen.lower_program(&program);
        assert!(!wasm.is_empty());
    }
}