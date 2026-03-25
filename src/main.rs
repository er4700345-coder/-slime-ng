use std::env;
use std::fs;
use std::path::Path;

use slime::lexer::Lexer;
use slime::parser::Parser;
use slime::typechecker::TypeChecker;
use slime::wasm::WasmBackend;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Usage: slimec <command> <file>");
        eprintln!("Commands:");
        eprintln!("  lex     - Tokenize and print tokens");
        eprintln!("  parse   - Parse and print AST");
        eprintln!("  check   - Type check the program");
        eprintln!("  compile - Compile to WASM text format");
        eprintln!("  build   - Full pipeline: lex -> parse -> check -> compile");
        return;
    }
    
    let command = &args[1];
    let filepath = &args[2];
    
    if !Path::new(filepath).exists() {
        eprintln!("Error: File '{}' not found", filepath);
        return;
    }
    
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
        _ => {
            eprintln!("Unknown command: {}", command);
        }
    }
}

fn cmd_lex(source: &str) {
    let mut lexer = Lexer::new(source);
    let mut count = 0;
    
    println!("Tokens:");
    println!("{:-<40}", "");
    
    loop {
        let tok = lexer.next_token();
        println!("  {:?}", tok);
        count += 1;
        
        if matches!(tok, slime::lexer::Token::EOF) {
            break;
        }
    }
    
    println!("{:-<40}", "");
    println!("Total: {} tokens", count);
}

fn cmd_parse(source: &str) {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let ast = parser.parse();
    
    println!("AST:");
    println!("{:#?}", ast);
}

fn cmd_check(source: &str) {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let ast = parser.parse();
    
    let mut checker = TypeChecker::new();
    checker.check_program(&ast);
    
    if checker.has_errors() {
        checker.report_errors();
        std::process::exit(1);
    } else {
        println!("✓ Type check passed");
    }
}

fn cmd_compile(source: &str) {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let ast = parser.parse();
    
    let mut checker = TypeChecker::new();
    checker.check_program(&ast);
    
    if checker.has_errors() {
        checker.report_errors();
        std::process::exit(1);
    }
    
    let mut backend = WasmBackend::new();
    let wasm = backend.compile(&ast);
    
    println!("{}", wasm);
}

fn cmd_build(source: &str, filepath: &str) {
    println!("Building {}...", filepath);
    
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let ast = parser.parse();
    println!("  ✓ Parsed");
    
    let mut checker = TypeChecker::new();
    checker.check_program(&ast);
    
    if checker.has_errors() {
        checker.report_errors();
        std::process::exit(1);
    }
    println!("  ✓ Type checked");
    
    let mut backend = WasmBackend::new();
    let wasm = backend.compile(&ast);
    println!("  ✓ Compiled to WASM");
    
    let output_path = filepath.replace(".slime", ".wat");
    match fs::write(&output_path, wasm) {
        Ok(_) => println!("  ✓ Output: {}", output_path),
        Err(e) => {
            eprintln!("  ✗ Write failed: {}", e);
            std::process::exit(1);
        }
    }
    
    println!("Build complete!");
    }
        
