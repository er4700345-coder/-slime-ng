use std::process::Command;

pub fn run_output(output_file: &str) {
    println!("Running output: {}", output_file);

    let result = if output_file.ends_with(".js") {
        Command::new("node").arg(output_file).status()
    } else {
        Command::new(output_file).status()
    };

    match result {
        Ok(status) => {
            println!("Execution finished with status: {}", status);
        }
        Err(err) => {
            eprintln!("Failed to run output: {}", err);
        }
    }
}
