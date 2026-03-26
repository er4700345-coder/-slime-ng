pub fn compile_to_native_stub(source: &str) -> String {
    format!(
        "// Native backend stub for SLIME\n// Source length: {}\nint main() {{ return 0; }}",
        source.len()
    )
}
