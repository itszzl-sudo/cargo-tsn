use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, Write};
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
    #[command(about = "Interactively add FFI function")]
    Func,
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::New { name } => cmd_new(&name),
        Commands::Add { crate_name } => cmd_add(&crate_name),
        Commands::Func => cmd_func(),
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

fn cmd_func() {
    println!("Current directory: .");
    
    // 检查 tsnp 目录
    let tsnp_dir = std::path::Path::new("tsnp");
    if !tsnp_dir.exists() {
        eprintln!("❌ tsnp/ directory not found. Run this command in a tsn project root.");
        return;
    }
    
    let mut total_count = 0;
    
    loop {
        // 列出 tsnp 下的子目录
        let crates: Vec<String> = fs::read_dir(tsnp_dir)
            .expect("Failed to read tsnp directory")
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_dir())
            .filter_map(|entry| entry.file_name().to_string_lossy().into_string().ok())
            .collect();
        
        if crates.is_empty() {
            eprintln!("❌ No crates found in tsnp/. Run 'cargo tsn add <crate>' first.");
            return;
        }
        
        // 选择 crate
        println!("\nSelect crate:");
        for (i, crate_name) in crates.iter().enumerate() {
            println!("[{}] {} (tsnp/{}/)", i + 1, crate_name, crate_name);
        }
        println!("[q] Quit");
        
        print!("\nSelect: ");
        io::stdout().flush().expect("Failed to flush");
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        let input = input.trim();
        
        if input == "q" {
            break;
        }
        
        let selected_idx: usize = match input.parse() {
            Ok(n) if n >= 1 && n <= crates.len() => n - 1,
            _ => {
                eprintln!("Invalid selection.");
                continue;
            }
        };
        
        let selected_crate = &crates[selected_idx];
        println!("\n{} selected.", selected_crate);
        
        // 函数添加循环
        loop {
            print!("\nFunction name (or 'q'): ");
            io::stdout().flush().expect("Failed to flush");
            
            let mut func_name = String::new();
            io::stdin().read_line(&mut func_name).expect("Failed to read input");
            let func_name = func_name.trim();
            
            if func_name == "q" || func_name.is_empty() {
                break;
            }
            
            // 输入参数
            print!("Parameters (e.g., 'a: i32, b: i32'): ");
            io::stdout().flush().expect("Failed to flush");
            
            let mut params = String::new();
            io::stdin().read_line(&mut params).expect("Failed to read input");
            let params = params.trim();
            
            // 输入返回值
            print!("Return type (e.g., 'i32'): ");
            io::stdout().flush().expect("Failed to flush");
            
            let mut ret_type = String::new();
            io::stdin().read_line(&mut ret_type).expect("Failed to read input");
            let ret_type = ret_type.trim();
            
            // 生成 FFI 函数
            add_ffi_function(selected_crate, func_name, params, ret_type);
            total_count += 1;
            
            println!("✅ Added to src/lib.rs");
            println!("✅ Updated tsnp/{}/ts-native.toml", selected_crate);
        }
    }
    
    println!("\nDone. {} FFI function(s) added.", total_count);
}

fn add_ffi_function(crate_name: &str, func_name: &str, params: &str, ret_type: &str) {
    // 生成 Rust FFI 代码
    let ffi_code = if ret_type.is_empty() || ret_type == "void" {
        format!(
            r#"
#[no_mangle]
pub extern "C" fn {}({}) {{
    // TODO: implement
}}
"#,
            func_name, params
        )
    } else {
        format!(
            r#"
#[no_mangle]
pub extern "C" fn {}({}) -> {} {{
    // TODO: implement
    0 as {}
}}
"#,
            func_name, params, ret_type, ret_type
        )
    };
    
    // 追加到 src/lib.rs
    let lib_rs_path = "src/lib.rs";
    if let Ok(mut content) = fs::read_to_string(lib_rs_path) {
        content.push_str(&ffi_code);
        fs::write(lib_rs_path, content).expect("Failed to write lib.rs");
    }
    
    // 更新 ts-native.toml
    let toml_path = format!("tsnp/{}/ts-native.toml", crate_name);
    if let Ok(mut content) = fs::read_to_string(&toml_path) {
        // 推断 TypeScript 类型
        let ts_params = infer_ts_params(params);
        let ts_ret = infer_ts_type(ret_type);
        
        let func_entry = format!(
            r#""{}" = {{ args = [{}], ret = "{}", impl_name = "{}" }}
"#,
            func_name,
            ts_params.iter().map(|t| format!("\"{}\"", t)).collect::<Vec<_>>().join(", "),
            ts_ret,
            func_name
        );
        
        // 在 [functions] 下添加
        if let Some(pos) = content.find("[functions]\n") {
            let insert_pos = pos + "[functions]\n".len();
            content.insert_str(insert_pos, &func_entry);
            fs::write(&toml_path, content).expect("Failed to write toml");
        }
    }
}

fn infer_ts_params(params: &str) -> Vec<String> {
    if params.trim().is_empty() {
        return vec![];
    }
    
    params.split(',')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .map(|p| {
            if let Some(colon_pos) = p.find(':') {
                let ty = p[colon_pos + 1..].trim();
                infer_ts_type(ty)
            } else {
                "any".to_string()
            }
        })
        .collect()
}

fn infer_ts_type(rust_type: &str) -> String {
    let ty = rust_type.trim();
    
    if ty.is_empty() || ty == "void" || ty == "()" {
        return "void".to_string();
    }
    
    if ty.contains("c_char") || ty.starts_with("&str") {
        return "string".to_string();
    }
    
    "number".to_string()
}
