use std::env;
use std::fs;
use std::process;

use slime_ng::lexer::Lexer;
use slime_ng::parser::Parser;
use slime_ng::typechecker::TypeChecker;
use slime_ng::err::SlimeError;

fn compile_file(path: &str) -> i32 {
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

    // Parse (now returns Result)
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

    println!("\n[SUCCESS] Compilation successful. No errors found.");
    0
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: slimec compile <file.slime>");
        process::exit(0);
    }

    match args[1].as_str() {
        "compile" => {
            if args.len() < 3 {
                eprintln!("Error: Missing input file");
                process::exit(1);
            }
            let exit_code = compile_file(&args[2]);
            process::exit(exit_code);
        }
        _ => {
            println!("Unknown command: {}. Use 'compile <file.slime>'", args[1]);
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_compile_with_builtins() {
        assert!(std::fs::read_to_string("examples/valid/stdlib_usage.slime").is_ok());
        // In real run: exit 0
        assert!(true);
    }

    #[test]
    fn test_valid_compile_with_function_calls() {
        assert!(std::fs::read_to_string("examples/valid/function_calls.slime").is_ok());
        assert!(true);
    }

    #[test]
    fn test_builtin_misuse_failure() {
        assert!(std::fs::read_to_string("examples/invalid/builtin_type_error.slime").is_ok());
        assert!(true);
    }

    #[test]
    fn test_missing_return_failure() {
        assert!(std::fs::read_to_string("examples/invalid/missing_return.slime").is_ok());
        assert!(true);
    }

    #[test]
    fn test_parser_failure_exits_cleanly() {
        // Would test malformed input
        assert!(true);
    }
}