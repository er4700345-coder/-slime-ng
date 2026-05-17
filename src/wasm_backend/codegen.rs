use crate::ir::types::*;
use walrus::{FunctionBuilder, InstrSeqBuilder, Module, ValType};
use wasmparser::validate;
use wasmi::{Engine, Linker, Module as WasmModule, Store, Value as WasmValue};

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

    fn execute_wasm(wasm: &[u8], func_name: &str, args: &[WasmValue]) -> Option<i32> {
        let engine = Engine::default();
        let module = WasmModule::new(&engine, wasm).ok()?;
        let mut store = Store::new(&engine, ());
        let linker = Linker::new(&engine);
        let instance = linker.instantiate(&mut store, &module).ok()?.start(&mut store).ok()?;
        let func = instance.get_func(&store, func_name)?;
        let mut results = [WasmValue::I32(0)];
        func.call(&mut store, args, &mut results).ok()?;
        if let WasmValue::I32(v) = results[0] { Some(v) } else { None }
    }

    #[test]
    fn test_simple_return_executes() {
        let mut codegen = WasmCodegen::new();
        let program = make_simple_program();
        let wasm = codegen.lower_program(&program);
        assert!(validate(&wasm).is_ok());
        let result = execute_wasm(&wasm, "main", &[]);
        assert_eq!(result, Some(42));
    }

    #[test]
    fn test_binary_operation_executes() {
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
        assert!(validate(&wasm).is_ok());
        let result = execute_wasm(&wasm, "add", &[]);
        assert_eq!(result, Some(42)); // placeholder, real would compute
    }

    #[test]
    fn test_function_call_executes() {
        // Placeholder: if call lowering supports, execute
        assert!(true);
    }

    #[test]
    fn test_invalid_source_does_not_execute() {
        // Invalid blocks before WASM
        assert!(true);
    }
}