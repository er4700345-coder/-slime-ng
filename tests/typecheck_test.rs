use slime::lexer::Lexer;
use slime::parser::Parser;
use slime::typechecker::TypeChecker;

#[test]
fn test_valid_program() {
    let input = "fn main() { let x: i32 = 42; }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let ast = parser.parse();
    
    let mut checker = TypeChecker::new();
    checker.check_program(&ast);
    
    assert!(!checker.has_errors());
}

#[test]
fn test_type_mismatch() {
    let input = "fn main() { let x: i32 = \"hello\"; }";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let ast = parser.parse();
    
    let mut checker = TypeChecker::new();
    checker.check_program(&ast);
    
    assert!(checker.has_errors());
}
