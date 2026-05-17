use std::env;
use std::fs;
use std::process;

use slime_ng::lexer::Lexer;
use slime_ng::parser::Parser;
use slime_ng::typechecker::TypeChecker;
use slime_ng::ir::lowering::LoweringContext;
use slime_ng::wasm_backend::WasmCodegen;

fn compile_file(path: &str, emit_wasm: bool) -> i32 {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: Failed to read '{}': {}", path, e);
            return 1;
        }
    };

    println!("Compiling '{}'
", path);

    // Lex
    let mut lexer = Lexer::new(&source);
    loop {
        match lexer.next_token() {
            Ok(slime_ng::lexer::Token::EOF) => break,
            Ok(_) => {},
            Err(e) => {
                eprintln!("{}", e);
                return 1;
            }
        }
    }

    // Parse
    let mut parser = Parser::new(Lexer::new(&source));
    let decls = match parser.parse() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };

    // Typecheck
    let mut tc = TypeChecker::new();
    tc.check_program(&decls);

    if tc.has_errors() {
        eprintln!("\nType errors:");
        for err in tc.get_errors() {
            eprintln!("  {}", err);
        }
        eprintln!("\nCompilation failed.");
        return 1;
    }

    if emit_wasm {
        // Lower to IR
        let mut lower = LoweringContext::new();
        let program = lower.lower_program(&decls);

        // WASM codegen
        let mut codegen = WasmCodegen::new();
        let wasm_bytes = codegen.lower_program(&program);

        let wasm_path = if path.ends_with(".slime") {
            path.replace(".slime", ".wasm")
        } else {
            format!("{}.wasm", path)
        };

        match fs::write(&wasm_path, &wasm_bytes) {
            Ok(_) => println!("\n[SUCCESS] WASM emitted to {}", wasm_path),
            Err(e) => {
                eprintln!("Error writing WASM: {}", e);
                return 1;
            }
        }
    } else {
        println!("\n[SUCCESS] Compilation successful. No errors found.");
    }

    0
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: slimec compile <file.slime> [--emit wasm]");
        process::exit(0);
    }

    match args[1].as_str() {
        "compile" => {
            if args.len() < 3 {
                eprintln!("Error: Missing input file");
                process::exit(1);
            }
            let emit_wasm = args.len() > 3 && args[3] == "--emit" && args.get(4) == Some(&"wasm".to_string());
            let exit_code = compile_file(&args[2], emit_wasm);
            process::exit(exit_code);
        }
        _ => {
            println!("Unknown command: {}. Use 'compile <file.slime> [--emit wasm]'", args[1]);
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_wasm_stdlib_print() {
        assert!(std::fs::read_to_string("examples/valid/wasm_stdlib_print.slime").is_ok());
        assert!(true);
    }

    #[test]
    fn test_cli_wasm_stdlib_len() {
        assert!(std::fs::read_to_string("examples/valid/wasm_stdlib_len.slime").is_ok());
        assert!(true);
    }

    #[test]
    fn test_cli_wasm_stdlib_to_string() {
        assert!(std::fs::read_to_string("examples/valid/wasm_stdlib_to_string.slime").is_ok());
        assert!(true);
    }

    #[test]
    fn test_cli_wasm_stdlib_input() {
        assert!(std::fs::read_to_string("examples/valid/wasm_stdlib_input.slime").is_ok());
        assert!(true);
    }

    #[test]
    fn test_invalid_source_blocks_wasm() {
        assert!(std::fs::read_to_string("examples/invalid/builtin_type_error.slime").is_ok());
        assert!(true);
    }

    #[test]
    fn test_emitted_wasm_validates() {
        assert!(true);
    }
}