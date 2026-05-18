use slime_ng::parser::Parser;
use slime_ng::lexer::Lexer;
use slime_ng::typechecker::TypeChecker;
use slime_ng::ir::lowering::LoweringContext;

#[test]
fn test_hello_pipeline() {
    let source = include_str!("../examples/hello.slime");
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let decls = parser.parse().expect("parse failed");
    
    let mut checker = TypeChecker::new();
    checker.check_program(&decls).expect("typecheck failed");
    assert!(!checker.has_errors());
    
    let mut lower = LoweringContext::new();
    let ir = lower.lower_program(&decls).expect("lowering failed");
    
    assert!(!ir.is_empty());
    println!("Pipeline success: {} decls parsed and lowered", decls.len());
}