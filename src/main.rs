use clap::{Parser, Subcommand};
use std::fs;
use std::process::Command;

#[derive(Parser)]
#[command(name = "cargo-tsn")]
#[command(about = "tsn project manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Create a new tsn project")]
    New {
        name: String,
    },
    #[command(about = "Add a crate dependency and generate plugin")]
    Add {
        crate_name: String,
    },
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::New { name } => cmd_new(&name),
        Commands::Add { crate_name } => cmd_add(&crate_name),
    }
}

fn cmd_new(name: &str) {
    println!("Creating tsn project: {}", name);
    
    fs::create_dir_all(name).expect("Failed to create directory");
    fs::create_dir_all(format!("{}/src", name)).expect("Failed to create src");
    fs::create_dir_all(format!("{}/tsnp", name)).expect("Failed to create tsnp");
    
    let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
"#, name);
    fs::write(format!("{}/Cargo.toml", name), cargo_toml).expect("Failed to write Cargo.toml");
    
    let lib_rs = r#"// Export FFI functions here
// Example:
// #[no_mangle]
// pub extern "C" fn my_func() -> i32 { 0 }
"#;
    fs::write(format!("{}/src/lib.rs", name), lib_rs).expect("Failed to write lib.rs");
    
    let main_ts = r#"function main() {
    print("Hello, tsn!");
    return 0;
}
"#;
    fs::write(format!("{}/main.ts", name), main_ts).expect("Failed to write main.ts");
    
    println!("✅ Created: {}", name);
    println!("   cd {} && tsn main.ts", name);
}

fn cmd_add(crate_name: &str) {
    println!("Adding crate: {}", crate_name);
    
    // 1. cargo add
    println!("Running: cargo add {}", crate_name);
    let status = Command::new("cargo")
        .args(["add", crate_name])
        .status()
        .expect("Failed to run cargo add");
    
    if !status.success() {
        eprintln!("❌ cargo add failed");
        return;
    }
    
    // 2. tsnp gen
    println!("Running: tsnp gen {}", crate_name);
    let status = Command::new("tsnp")
        .args(["gen", crate_name])
        .status()
        .expect("Failed to run tsnp gen");
    
    if !status.success() {
        eprintln!("⚠️  tsnp gen failed (crate may have no FFI functions)");
    }
    
    println!("✅ Added: {}", crate_name);
}
