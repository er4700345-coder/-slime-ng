#[derive(Debug, Clone)]
pub enum Expr {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Identifier(String),
    Binary(Box<Expr>, BinOp, Box<Expr>),
    Unary(UnOp, Box<Expr>),
    Call(String, Vec<Expr>),
    Assign(String, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add, Sub, Mul, Div,
    Eq, Neq, Lt, Gt, Lte, Gte,
    And, Or,
}

#[derive(Debug, Clone)]
pub enum UnOp {
    Not, Neg,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(String, Option<Type>, Box<Expr>),
    Return(Option<Box<Expr>>),
    Expr(Box<Expr>),
    If(Box<Expr>, Vec<Stmt>, Option<Vec<Stmt>>),
    While(Box<Expr>, Vec<Stmt>),
    Block(Vec<Stmt>),
}

#[derive(Debug, Clone)]
pub enum Type {
    I32, I64, F32, F64, Bool, String, Void,
    Func(Vec<Type>, Box<Type>),
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub ret_type: Type,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct Target {
    pub name: String,
    pub options: Vec<(String, Literal)>,
}

#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Bool(bool),
    Integer(i64),
}

#[derive(Debug, Clone)]
pub enum Decl {
    Function(Function),
    Target(Target),
    Import(String),
}
