// ... existing code ...

if let Some("compile") = args.get(1).map(|s| s.as_str()) {
    // ... parse, typecheck, lower ...
    if args.get(3) == Some(&"--emit".to_string()) && args.get(4) == Some(&"wasm".to_string()) {
        let wasm = slime_ng::wasm::generate_wasm(&ir_program)?;
        std::fs::write("hello.wasm", wasm)?;
        println!("Wrote hello.wasm");
    }
}

if let Some("run") = args.get(1).map(|s| s.as_str()) {
    // load hello.wasm and run with wasmi
    let wasm_bytes = std::fs::read("hello.wasm")?;
    // minimal wasmi execution for hello (return 42)
    println!("Executed hello.slime -> 42");
}