// src/lib.rs
pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod stdlib;       // ← new
pub mod typechecker;
pub mod wasm;
pub mod wasm_binary;
