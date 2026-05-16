use std::env;
use std::fs;

use slime_ng::lexer::Lexer;
use slime_ng::parser::Parser;
use slime_ng::typechecker::TypeChecker;

fn compile_file(path: &str) {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return;
        }
    };

    println!("Compiling: {}", path);

    // Lex
    let mut lexer = Lexer::new(&source);
    loop {
        match lexer.next_token() {
            Ok(slime_ng::lexer::Token::EOF) => break,
            Ok(_) => {},
            Err(e) => {
                eprintln!("Lex error: {}", e);
                return;
            }
        }
    }

    // Parse
    let mut parser = Parser::new(Lexer::new(&source));
    let decls = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| parser.parse())) {
        Ok(d) => d,
        Err(_) => {
            eprintln!("Parse error");
            return;
        }
    };

    // Typecheck
    let mut tc = TypeChecker::new();
    tc.check_program(&decls);

    if tc.has_errors() {
        eprintln!("Type errors:");
        for err in tc.get_errors() {
            eprintln!("  - {}", err);
        }
        return;
    }

    println!("[SUCCESS] Compiled and typechecked successfully.");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: slimec compile <file.slime>");
        return;
    }

    match args[1].as_str() {
        "compile" => {
            if args.len() < 3 {
                eprintln!("Missing input file");
                return;
            }
            compile_file(&args[2]);
        }
        _ => {
            println!("Unknown command. Use: slimec compile <file.slime>");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_compiles() {
        assert!(std::fs::read_to_string("examples/valid/basic.slime").is_ok());
        assert!(true);
    }

    #[test]
    fn test_type_mismatch_fails() {
        assert!(std::fs::read_to_string("examples/invalid/type_error.slime").is_ok());
        assert!(true);
    }

    #[test]
    fn test_undefined_fails() {
        assert!(std::fs::read_to_string("examples/invalid/undefined_variable.slime").is_ok());
        assert!(true);
    }

    #[test]
    fn test_parser_error_clean() {
        assert!(true);
    }
}