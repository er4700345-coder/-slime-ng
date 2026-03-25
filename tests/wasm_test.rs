use slime::lexer::Lexer;
use slime::parser::Parser;
use slime::wasm::WasmBackend;

#[test]
fn test_wasm_output() {
    let input = "fn main() { let x = 42; }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let ast = parser.parse();
    
    let mut backend = WasmBackend::new();
    let wasm = backend.compile(&ast);
    
    assert!(wasm.contains("(module"));
    assert!(wasm.contains("(func $main"));
}

