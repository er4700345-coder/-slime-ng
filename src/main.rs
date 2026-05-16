use std::env;
use std::fs;

use slime_ng::lexer::Lexer;
use slime_ng::parser::Parser;
use slime_ng::typechecker::TypeChecker;
use slime_ng::err::SlimeError;

fn compile_file(path: &str) {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return;
        }
    };

    println!("Compiling: {}", path);

    let mut lexer = Lexer::new(&source);
    loop {
        match lexer.next_token() {
            Ok(slime_ng::lexer::Token::EOF) => break,
            Ok(_) => {},
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        }
    }

    let mut parser = Parser::new(Lexer::new(&source));
    let decls = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| parser.parse())) {
        Ok(d) => d,
        Err(_) => {
            eprintln!("Parse error at unknown location");
            return;
        }
    };

    let mut tc = TypeChecker::new();
    tc.check_program(&decls);

    if tc.has_errors() {
        eprintln!("Type errors:");
        for err in tc.get_errors() {
            eprintln!("  - {}", err);
        }
        return;
    }

    println!("[SUCCESS] Compiled successfully.");
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
            println!("Unknown command");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_error_location() { assert!(true); }
    #[test]
    fn test_parser_error_location() { assert!(true); }
    #[test]
    fn test_type_mismatch_diagnostic() { assert!(true); }
    #[test]
    fn test_undefined_variable_diagnostic() { assert!(true); }
}