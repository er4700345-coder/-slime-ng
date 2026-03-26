use std::process::Command;

pub fn run_wasm_output(output_file: &str) {
    println!("Running SLIME output: {}", output_file);

    let result = Command::new("node")
        .arg(output_file)
        .status();

    match result {
        Ok(status) => {
            println!("Execution finished with status: {}", status);
        }
        Err(err) => {
            eprintln!("Failed to run output: {}", err);
        }
    }
}
