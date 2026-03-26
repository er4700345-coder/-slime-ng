// src/pkg/mod.rs
// slimepkg — SLIME Package Manager
// Commands: init, install, add, remove, publish, search

pub mod lockfile;
pub mod manifest;
pub mod registry;

use std::fs;
use std::path::{Path, PathBuf};

use lockfile::Lockfile;
use manifest::{Manifest, Version, VersionReq};
use registry::Resolver;

const MANIFEST_FILE: &str = "slime.toml";
const LOCK_FILE: &str = "SLIME.lock";
const PKG_DIR: &str = ".slime/packages";

/// Entry point for all `slimepkg` CLI commands
pub fn run_pkg_command(args: &[String]) {
    if args.is_empty() {
        print_pkg_help();
        return;
    }

    match args[0].as_str() {
        "init"    => cmd_init(args),
        "install" => cmd_install(),
        "add"     => cmd_add(args),
        "remove"  => cmd_remove(args),
        "search"  => cmd_search(args),
        "publish" => cmd_publish(),
        "list"    => cmd_list(),
        "info"    => cmd_info(args),
        _         => {
            eprintln!("slimepkg: unknown command '{}'", args[0]);
            print_pkg_help();
        }
    }
}

fn print_pkg_help() {
    println!("slimepkg — SLIME Package Manager\n");
    println!("Usage: slimec pkg <command> [args]\n");
    println!("Commands:");
    println!("  init              Create a new slime.toml in the current directory");
    println!("  install           Install all dependencies from slime.toml");
    println!("  add <name>        Add a dependency and install it");
    println!("  remove <name>     Remove a dependency");
    println!("  search <query>    Search the registry");
    println!("  publish           Publish this package to the registry");
    println!("  list              List installed packages");
    println!("  info <name>       Show package info");
}

// ── Commands ─────────────────────────────────────────────────────────────────

/// slimec pkg init
fn cmd_init(args: &[String]) {
    let name = args.get(1)
        .cloned()
        .unwrap_or_else(|| {
            std::env::current_dir()
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
                .unwrap_or_else(|| "my-package".to_string())
        });

    let manifest_path = Path::new(MANIFEST_FILE);
    if manifest_path.exists() {
        eprintln!("slime.toml already exists.");
        return;
    }

    let toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
authors = []
description = ""
license = "MIT"
entry = "src/main.slime"

[dependencies]
"#,
        name
    );

    fs::write(manifest_path, toml).expect("Cannot write slime.toml");

    // Create src/main.slime stub
    fs::create_dir_all("src").ok();
    if !Path::new("src/main.slime").exists() {
        fs::write(
            "src/main.slime",
            "fn main() -> i32 {\n    0\n}\n",
        )
        .ok();
    }

    println!("Initialized package '{}'", name);
    println!("  slime.toml created");
    println!("  src/main.slime created");
}

/// slimec pkg install
fn cmd_install() {
    let manifest = load_manifest();
    println!("Installing dependencies for {}...", manifest.package.name);

    fs::create_dir_all(PKG_DIR).ok();

    let lock_path = Path::new(LOCK_FILE);

    // If lockfile exists, use it (reproducible installs)
    if lock_path.exists() {
        println!("Found SLIME.lock — installing exact versions");
        let lockfile = Lockfile::from_file(lock_path)
            .expect("Corrupt SLIME.lock");
        install_from_lockfile(&lockfile);
        return;
    }

    // No lockfile — resolve and create one
    let mut resolver = Resolver::new();
    let lockfile = resolver
        .resolve(&manifest.dependencies)
        .expect("Dependency resolution failed");

    lockfile.write(lock_path).expect("Cannot write SLIME.lock");
    install_from_lockfile(&lockfile);

    println!("\nDone. {} packages installed.", lockfile.packages.len());
    println!("SLIME.lock written.");
}

fn install_from_lockfile(lockfile: &Lockfile) {
    for (key, pkg) in &lockfile.packages {
        let pkg_dir = PathBuf::from(PKG_DIR)
            .join(format!("{}-{}", pkg.name, pkg.version.to_string()));

        if pkg_dir.exists() {
            println!("  {} {} (already installed)", pkg.name, pkg.version.to_string());
        } else {
            println!("  {} {} (installing...)", pkg.name, pkg.version.to_string());
            fs::create_dir_all(&pkg_dir).ok();
            // Actual download happens in Resolver — stub here for now
        }
    }
}

/// slimec pkg add <name> [version]
fn cmd_add(args: &[String]) {
    if args.len() < 2 {
        eprintln!("Usage: slimec pkg add <package-name> [version]");
        return;
    }

    let pkg_name = &args[1];
    let version_req = args.get(2)
        .map(|v| v.as_str())
        .unwrap_or("*");

    let req = VersionReq::parse(version_req)
        .expect("Invalid version requirement");

    // Load and update manifest
    let mut manifest = load_manifest();
    manifest.dependencies.insert(pkg_name.clone(), req);

    let manifest_path = Path::new(MANIFEST_FILE);
    fs::write(manifest_path, manifest.to_toml())
        .expect("Cannot update slime.toml");

    println!("Added {} to dependencies.", pkg_name);
    println!("Run `slimec pkg install` to install.");
}

/// slimec pkg remove <name>
fn cmd_remove(args: &[String]) {
    if args.len() < 2 {
        eprintln!("Usage: slimec pkg remove <package-name>");
        return;
    }

    let pkg_name = &args[1];
    let mut manifest = load_manifest();

    if manifest.dependencies.remove(pkg_name).is_none() {
        eprintln!("{} is not in your dependencies.", pkg_name);
        return;
    }

    fs::write(MANIFEST_FILE, manifest.to_toml())
        .expect("Cannot update slime.toml");

    // Remove from SLIME.lock
    let lock_path = Path::new(LOCK_FILE);
    if lock_path.exists() {
        let mut lockfile = Lockfile::from_file(lock_path).unwrap_or_else(|_| Lockfile::new());
        lockfile.packages.retain(|k, _| !k.starts_with(&format!("{}@", pkg_name)));
        lockfile.write(lock_path).ok();
    }

    println!("Removed {} from dependencies.", pkg_name);
}

/// slimec pkg search <query>
fn cmd_search(args: &[String]) {
    if args.len() < 2 {
        eprintln!("Usage: slimec pkg search <query>");
        return;
    }
    let query = &args[1];
    println!("Searching registry for '{}'...", query);
    println!("(Registry at https://registry.slime-lang.dev — coming soon)");
    // TODO: HTTP search when registry is live
}

/// slimec pkg publish
fn cmd_publish() {
    let manifest = load_manifest();
    println!("Publishing {} v{}...", manifest.package.name, manifest.package.version.to_string());
    println!("(Registry publishing coming in slimepkg v0.2)");
    // TODO: pack + upload when registry is live
}

/// slimec pkg list
fn cmd_list() {
    let lock_path = Path::new(LOCK_FILE);
    if !lock_path.exists() {
        println!("No packages installed. Run `slimec pkg install`.");
        return;
    }

    let lockfile = Lockfile::from_file(lock_path).expect("Corrupt SLIME.lock");
    if lockfile.packages.is_empty() {
        println!("No dependencies.");
        return;
    }

    println!("Installed packages:");
    let mut keys: Vec<_> = lockfile.packages.keys().collect();
    keys.sort();
    for key in keys {
        let pkg = &lockfile.packages[key];
        println!("  {} {}", pkg.name, pkg.version.to_string());
    }
}

/// slimec pkg info <name>
fn cmd_info(args: &[String]) {
    if args.len() < 2 {
        eprintln!("Usage: slimec pkg info <package-name>");
        return;
    }
    let name = &args[1];
    println!("Fetching info for '{}'...", name);
    println!("(Registry at https://registry.slime-lang.dev — coming soon)");
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn load_manifest() -> Manifest {
    let path = Path::new(MANIFEST_FILE);
    if !path.exists() {
        eprintln!("No slime.toml found. Run `slimec pkg init` first.");
        std::process::exit(1);
    }
    Manifest::from_file(path).unwrap_or_else(|e| {
        eprintln!("Error reading slime.toml: {}", e);
        std::process::exit(1);
    })
}
