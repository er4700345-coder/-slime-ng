use slime::lexer::Lexer;
use slime::parser::Parser;

fn main() {
    let input = r#"
target wasm {
    dom = true
}

fn main() {
    let x = 42;
    let msg = "Hello SLIME!";
    print(msg);
    return x;
}
"#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let ast = parser.parse();
    
    for decl in ast {
        println!("{:#?}", decl);
    }
}
