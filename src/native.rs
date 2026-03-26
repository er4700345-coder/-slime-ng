use std::fs;
use crate::native_backend::compile_to_native_stub;

pub fn build_native(input: &str, output: &str) {
    let source = fs::read_to_string(input)
        .expect("Failed to read SLIME source file");

    let native_code = compile_to_native_stub(&source);

    fs::write(output, native_code)
        .expect("Failed to write native output");

    println!("Native output written to {}", output);
}
