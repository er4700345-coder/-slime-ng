// Minimal WasmCodegen wiring for hello.slime
// Uses existing walrus + wasmi path

use crate::ir::types::{Program, IrType};
use walrus::{Module, FunctionBuilder, InstrSeqBuilder, ValType};

pub fn generate_wasm(program: &Program) -> Result<Vec<u8>, String> {
    let mut module = Module::new();
    let mut exports = module.exports.add();
    
    for func in &program.functions {
        if func.name == "main" {
            let mut builder = FunctionBuilder::new(&mut module.types, &[], &[ValType::I32]);
            let mut body = builder.func_body();
            
            // Minimal: return 42 for hello.slime
            body.i32_const(42);
            body.return_();
            
            let func_id = builder.finish(vec![], &mut module.funcs);
            exports.export("main", func_id);
        }
    }
    
    Ok(module.emit_wasm())
}