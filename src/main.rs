use std::env;
use std::fs;

use slime::lexer::Lexer;
use slime::parser::Parser;
use slime::typechecker::TypeChecker;
use slime::wasm::WasmBackend;
use slime::wasm_binary::WasmBinaryBackend;
use slime::native::NativeBackend;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: slimec <command> <file>");
        eprintln!("Commands:");
        eprintln!("  lex          - Tokenize and print tokens");
        eprintln!("  parse        - Parse and print AST");
        eprintln!("  check        - Type check the program");
        eprintln!("  compile      - Compile to WASM text format (WAT)");
        eprintln!("  build        - Full pipeline to .wat file");
        eprintln!("  build-bin    - Full pipeline to .wasm binary");
        eprintln!("  build-native - Compile to native object file (.o)");
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
        "lex"          => cmd_lex(&source),
        "parse"        => cmd_parse(&source),
        "check"        => cmd_check(&source),
        "compile"      => cmd_compile(&source),
        "build"        => cmd_build(&source, filepath),
        "build-bin"    => cmd_build_binary(&source, filepath),
        "build-native" => cmd_build_native(&source, filepath),
        _              => eprintln!("Unknown command: {}", command),
    }
}

fn cmd_lex(source: &str) {
    let lexer = Lexer::new(source).expect("Lex failed");
    for token in lexer {
        println!("{:?}", token);
    }
}

fn cmd_parse(source: &str) {
    let lexer = Lexer::new(source).expect("Lex failed");
    let mut parser = Parser::new(lexer);
    let ast = parser.parse();
    println!("{:#?}", ast);
}

fn cmd_check(source: &str) {
    let lexer = Lexer::new(source).expect("Lex failed");
    let mut parser = Parser::new(lexer);
    let ast = parser.parse();
    let mut checker = TypeChecker::new();
    checker.check_program(&ast);
    if checker.has_errors() {
        checker.report_errors();
        std::process::exit(1);
    } else {
        println!("Type check passed.");
    }
}

fn cmd_compile(source: &str) {
    let lexer = Lexer::new(source).expect("Lex failed");
    let mut parser = Parser::new(lexer);
    let ast = parser.parse();
    let mut backend = WasmBackend::new();
    let wat = backend.compile(&ast);
    println!("{}", wat);
}

fn cmd_build(source: &str, filepath: &str) {
    let lexer = Lexer::new(source).expect("Lex failed");
    let mut parser = Parser::new(lexer);
    let ast = parser.parse();
    let mut backend = WasmBackend::new();
    let wat = backend.compile(&ast);
    let output_path = filepath.replace(".slime", ".wat");
    fs::write(&output_path, wat).expect("Write failed");
    println!("Output: {}", output_path);
}

fn cmd_build_binary(source: &str, filepath: &str) {
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
    fs::write(&output_path, wasm).expect("Write failed");
    println!("Output: {}", output_path);
}

fn cmd_build_native(source: &str, filepath: &str) {
    println!("Compiling {} to native...", filepath);

    let lexer = Lexer::new(source).expect("Lex failed");
    let mut parser = Parser::new(lexer);
    let ast = parser.parse();

    let mut checker = TypeChecker::new();
    checker.check_program(&ast);
    if checker.has_errors() {
        checker.report_errors();
        std::process::exit(1);
    }

    let mut backend = NativeBackend::new();
    let obj_bytes = backend.compile(&ast);

    let output_path = filepath.replace(".slime", ".o");
    fs::write(&output_path, obj_bytes).expect("Write failed");

    println!("Object file: {}", output_path);
    println!("Link with:   cc {} -o {}", output_path, output_path.replace(".o", ""));
}
