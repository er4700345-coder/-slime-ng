use std::env;
use std::fs;

use slime_ng::manifest::SlimeManifest;
use slime_ng::native::build_native;
use slime_ng::registry::{fetch_package, install_all};
use slime_ng::run::run_output;

fn print_help() {
    println!("SLIME CLI");
    println!("Usage:");
    println!("  slimec lex <file>");
    println!("  slimec parse <file>");
    println!("  slimec check <file>");
    println!("  slimec build <file>");
    println!("  slimec run <output>");
    println!("  slimec native <input> <output.c>");
    println!("  slimec manifest");
    println!("  slimec pkg add <name>");
    println!("  slimec pkg install");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "lex" => {
            if args.len() < 3 {
                eprintln!("Missing input file");
                return;
            }

            let source = fs::read_to_string(&args[2])
                .expect("Failed to read source file");

            println!("Lexing file: {}", &args[2]);
            println!("{}", source);
        }

        "parse" => {
            if args.len() < 3 {
                eprintln!("Missing input file");
                return;
            }

            let source = fs::read_to_string(&args[2])
                .expect("Failed to read source file");

            println!("Parsing file: {}", &args[2]);
            println!("{}", source);
        }

        "check" => {
            if args.len() < 3 {
                eprintln!("Missing input file");
                return;
            }

            let source = fs::read_to_string(&args[2])
                .expect("Failed to read source file");

            println!("Type checking file: {}", &args[2]);
            println!("{}", source);
        }

        "build" => {
            if args.len() < 3 {
                eprintln!("Missing input file");
                return;
            }

            println!("Building SLIME source: {}", &args[2]);
            println!("WASM build flow should connect here.");
        }

        "run" => {
            if args.len() < 3 {
                eprintln!("Missing output file");
                return;
            }

            run_output(&args[2]);
        }

        "native" => {
            if args.len() < 4 {
                eprintln!("Usage: slimec native <input.slime> <output.c>");
                return;
            }

            build_native(&args[2], &args[3]);
        }

        "manifest" => {
            let manifest = SlimeManifest::load("slime.toml");
            println!("{:#?}", manifest);
        }

        "pkg" => {
            if args.len() < 3 {
                eprintln!("Usage: slimec pkg <add|install> [name]");
                return;
            }

            match args[2].as_str() {
                "add" => {
                    if args.len() < 4 {
                        eprintln!("Missing package name");
                        return;
                    }

                    fetch_package(&args[3]);
                }
                "install" => {
                    install_all();
                }
                _ => {
                    eprintln!("Unknown pkg command");
                }
            }
        }

        _ => {
            print_help();
        }
    }
}
