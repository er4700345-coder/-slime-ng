use slime::lexer::Lexer;
use slime::parser::Parser;
use slime::typechecker::TypeChecker;

fn main() {
    let input = r#"
fn main() {
    let x: i32 = 42;
    let y = "hello";
    let z = x + 10;
    let bad = x + y;
}
"#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let ast = parser.parse();
    
    let mut checker = TypeChecker::new();
    checker.check_program(&ast);
    
    if checker.has_errors() {
        checker.report_errors();
    } else {
        println!("Type check passed!");
    }
}
