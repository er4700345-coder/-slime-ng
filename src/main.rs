use std::env;
use std::fs;

use slime::lexer::Lexer;
use slime::parser::Parser;
use slime::typechecker::TypeChecker;
use slime::wasm::WasmBackend;
use slime::wasm_binary::WasmBinaryBackend;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Usage: slimec <command> <file>");
        eprintln!("Commands:");
        eprintln!("  lex       - Tokenize and print tokens");
        eprintln!("  parse     - Parse and print AST");
        eprintln!("  check     - Type check the program");
        eprintln!("  compile   - Compile to WASM text format");
        eprintln!("  build     - Full pipeline to WAT");
        eprintln!("  build-bin - Full pipeline to WASM binary");
        return;
    }
    
    let command = &args[1];
    let filepath = &args[2];
    
    let source = match fs::read_to_string(filepath) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return;
        }
    };
    
    match command.as_str() {
        "lex" => cmd_lex(&source),
        "parse" => cmd_parse(&source),
        "check" => cmd_check(&source),
        "compile" => cmd_compile(&source),
        "build" => cmd_build(&source, filepath),
        "build-bin" => cmd_build_binary(&source, filepath),
        _ => eprintln!("Unknown command: {}", command),
    }
}

fn cmd_build_binary(source: &str, filepath: &str) {
    println!("Building {} to binary...", filepath);
    
    let lexer = Lexer::new(source).expect("Lex failed");
    let mut parser = Parser::new(lexer);
    let ast = parser.parse();
    
    let mut checker = TypeChecker::new();
    checker.check_program(&ast);
    
    if checker.has_errors() {
        checker.report_errors();
        std::process::exit(1);
    }
    
    let mut backend = WasmBinaryBackend::new();
    let wasm = backend.compile(&ast);
    
    let output_path = filepath.replace(".slime", ".wasm");
    match fs::write(&output_path, wasm) {
        Ok(_) => println!("Output: {}", output_path),
        Err(e) => {
            eprintln!("Write failed: {}", e);
            std::process::exit(1);
        }
    }
        }
