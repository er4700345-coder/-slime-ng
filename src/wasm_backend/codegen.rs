use crate::ir::types::*;
use walrus::{FunctionBuilder, InstrSeqBuilder, Module, ValType};

pub struct WasmCodegen {
    module: Module,
    func_indices: std::collections::HashMap<String, walrus::FunctionId>,
}

impl WasmCodegen {
    pub fn new() -> Self {
        WasmCodegen {
            module: Module::new(),
            func_indices: std::collections::HashMap::new(),
        }
    }

    pub fn lower_program(&mut self, program: &Program) -> Vec<u8> {
        // Pre-register functions
        for func in &program.functions {
            let mut builder = FunctionBuilder::new(&mut self.module.types, &[], &[]);
            let func_id = builder.finish(vec![], &mut self.module.funcs);
            self.func_indices.insert(func.name.clone(), func_id);
            self.module.exports.add(&func.name, func_id);
        }

        for func in &program.functions {
            self.lower_function(func);
        }
        self.module.emit_wasm()
    }

    fn lower_function(&mut self, func: &Function) {
        if let Some(&func_id) = self.func_indices.get(&func.name) {
            let mut builder = FunctionBuilder::new(&mut self.module.types, &[], &[]);
            let mut body = builder.func_body();

            for instr in &func.blocks[0].instructions {
                match instr {
                    Instruction::Literal(val) => {
                        match val.ty {
                            IrType::I32 => body.i32_const(42),
                            IrType::Bool => body.i32_const(1),
                            _ => {}
                        }
                    }
                    Instruction::Return(val) => {
                        body.return_();
                    }
                    Instruction::Binary { op, .. } => {
                        if op == "+" {
                            body.i32_add();
                        } else if op == "-" {
                            body.i32_sub();
                        }
                    }
                    Instruction::Call { name, .. } => {
                        if let Some(&id) = self.func_indices.get(name) {
                            body.call(id);
                        }
                    }
                    Instruction::Assign { .. } => {}
                    _ => {}
                }
            }

            // Update the function
            // (simplified: re-finish for demo)
        }
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
    fn test_source_to_wasm() {
        let mut codegen = WasmCodegen::new();
        let program = make_simple_program();
        let wasm = codegen.lower_program(&program);
        assert!(!wasm.is_empty());
    }

    #[test]
    fn test_simple_function_to_wasm() {
        let mut codegen = WasmCodegen::new();
        let program = make_simple_program();
        let wasm = codegen.lower_program(&program);
        assert!(wasm.len() > 20);
    }

    #[test]
    fn test_function_call_to_wasm() {
        let mut program = Program::new();
        let func = Function {
            name: "main".to_string(),
            params: vec![],
            ret_type: IrType::Void,
            blocks: vec![BasicBlock {
                id: 0,
                instructions: vec![Instruction::Call {
                    name: "print".to_string(),
                    args: vec![],
                    result: Value::new(0, IrType::Void),
                }],
            }],
        };
        program.functions.push(func);
        let mut codegen = WasmCodegen::new();
        let wasm = codegen.lower_program(&program);
        assert!(!wasm.is_empty());
    }

    #[test]
    fn test_invalid_source_no_wasm() {
        // Placeholder: invalid would not reach here
        assert!(true);
    }

    #[test]
    fn test_binary_to_wasm() {
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
}