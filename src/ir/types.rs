use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum IrType {
    I32,
    I64,
    F32,
    F64,
    Bool,
    String,
    Void,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Value {
    pub id: usize,
    pub ty: IrType,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Literal(Value),
    Binary { op: String, lhs: Value, rhs: Value, result: Value },
    Call { name: String, args: Vec<Value>, result: Value },
    Return(Value),
    Assign { name: String, value: Value },
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: usize,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<(String, IrType)>,
    pub ret_type: IrType,
    pub blocks: Vec<BasicBlock>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
    pub entry: Option<String>,
}

impl Program {
    pub fn new() -> Self {
        Program {
            functions: vec![],
            entry: None,
        }
    }
}

impl Value {
    pub fn new(id: usize, ty: IrType) -> Self {
        Value { id, ty }
    }
}