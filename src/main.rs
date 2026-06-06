use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, Write};
use anyhow::{Context, Result};

mod prepare;

#[derive(Parser)]
#[command(name = "cargo-tsn")]
#[command(about = "ts-native project manager - plugin generation, dependency management, project scaffolding")]
#[command(long_about = "cargo-tsn is a project management tool for ts-native compiler.\n\nIt provides:\n- Project scaffolding (cargo tsn new)\n- Plugin generation from TypeScript source (cargo tsn prepare)\n- Plugin listing (cargo tsn list)\n- Interactive FFI function addition (cargo tsn func)\n\nFor detailed documentation, see: https://github.com/itszzl-sudo/cargo-tsn")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Create a new ts-native project")]
    #[command(long_about = "Create a new ts-native project with basic structure.\n\nExample:\n  cargo tsn new my-project\n\nThis creates:\n  my-project/\n  ├── main.ts           # TypeScript entry point\n  ├── Cargo.toml        # Rust configuration\n  └── tsnp/             # FFI plugin directory")]
    New {
        #[arg(help = "Project name")]
        name: String,
    },
    #[command(about = "Interactively add FFI function to existing plugin")]
    #[command(long_about = "Interactively add FFI function to an existing plugin.\n\nThis command provides a wizard to:\n1. Select an existing plugin from tsnp/\n2. Enter function name, parameters, and return type\n3. Automatically generate the FFI function stub\n\nExample:\n  cargo tsn func\n\nNote: This is for adding individual functions to existing plugins.\nFor generating plugins from TypeScript source, use 'cargo tsn prepare' instead.")]
    Func,
    #[command(about = "List available local plugins")]
    #[command(long_about = "List available local plugins from tsnp/ and tsnp-contrib/.\n\nShows:\n- Developer plugins (tsnp/) with priority 1000\n- Official plugins (tsnp-contrib/) with priority 0\n\nExample:\n  cargo tsn list")]
    List,
    #[command(about = "Analyze TypeScript code and generate plugin templates")]
    #[command(long_about = "Analyze TypeScript source code using AST parsing to detect plugin API usage,\nthen generate plugin templates and dependency declarations.\n\nWhat it does:\n1. Scans TypeScript files for plugin API calls (http_get, fs_writeFile, etc.)\n2. Detects which plugins are needed (http, fs, crypto, etc.)\n3. Generates empty templates for official plugins in prepare/templates/tsnp/\n4. Generates .ts.toml dependency files for each entry point (files with main function)\n\nOfficial plugins (from tsnp-contrib):\n  http, fs, crypto, os, path, cli, timer, json, net, process, log, env\n\nExamples:\n  cargo tsn prepare                     # Scan all *.ts files, output to ./prepare\n  cargo tsn prepare --input main.ts     # Only analyze main.ts\n  cargo tsn prepare --output my-plugins # Output to my-plugins/\n  cargo tsn prepare --dry-run           # Preview without writing files\n\nAfter running prepare:\n  1. Check prepare/has-tsnp-contrib.txt for official plugin list\n  2. Copy needed plugins: cp -r tsnp-contrib/http tsnp/\n  3. Compile: ts-native main.ts")]
    Prepare {
        #[arg(long, help = "Input TypeScript file(s) to analyze (default: auto-scan *.ts)")]
        input: Option<String>,
        #[arg(long, help = "Output directory (default: ./prepare)")]
        output: Option<String>,
        #[arg(long, help = "Preview mode - show what would be generated without writing files")]
        dry_run: bool,
        #[arg(long, help = "Disable stub generation - only generate comment templates")]
        no_stubs: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::New { name } => cmd_new(&name),
        Commands::Func => cmd_func(),
        Commands::List => cmd_list(),
        Commands::Prepare { input, output, dry_run, no_stubs } => {
            let input_ref = input.as_deref();
            // 无参时默认输出到 ./prepare 目录
            let output_dir = output.as_deref().unwrap_or("prepare");
            prepare::cmd_prepare(input_ref, output_dir, dry_run, no_stubs)
        }
    }
}

/// 确保 TOML 文件的 priority 字段为 1000
fn ensure_priority_1000(toml_path: &str) -> Result<()> {
    let content = fs::read_to_string(toml_path)
        .with_context(|| format!("Failed to read {}", toml_path))?;
    
    let mut updated = content.clone();
    
    if content.contains("priority =") {
        // 替换现有 priority
        updated = content
            .lines()
            .map(|line| {
                if line.trim().starts_with("priority =") {
                    "priority = 1000".to_string()
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
    } else {
        // 在 [package] 下添加
        if let Some(pos) = content.find("[package]\n") {
            let insert_pos = pos + "[package]\n".len();
            let next_section = content[insert_pos..].find('\n')
                .map(|p| insert_pos + p)
                .unwrap_or(content.len());
            updated.insert_str(next_section, "\npriority = 1000");
        }
    }
    
    fs::write(toml_path, updated)
        .with_context(|| format!("Failed to update priority in {}", toml_path))?;
    
    Ok(())
}

fn cmd_new(name: &str) -> Result<()> {
    println!("Creating tsn project: {}", name);
    
    fs::create_dir_all(name).context("Failed to create directory")?;
    fs::create_dir_all(format!("{}/src", name)).context("Failed to create src")?;
    fs::create_dir_all(format!("{}/tsnp", name)).context("Failed to create tsnp")?;
    
    let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
"#, name);
    fs::write(format!("{}/Cargo.toml", name), cargo_toml).context("Failed to write Cargo.toml")?;
    
    let lib_rs = r#"// Export FFI functions here
// Example:
// #[no_mangle]
// pub extern "C" fn my_func() -> i32 { 0 }
"#;
    fs::write(format!("{}/src/lib.rs", name), lib_rs).context("Failed to write lib.rs")?;
    
    let main_ts = r#"function main() {
    print("Hello, tsn!");
    return 0;
}
"#;
    fs::write(format!("{}/main.ts", name), main_ts).context("Failed to write main.ts")?;
    
    println!("✅ Created: {}", name);
    println!("   cd {} && ts-native main.ts", name);
    
    Ok(())
}

fn cmd_func() -> Result<()> {
    println!("Current directory: .");
    
    // 检查 tsnp 目录
    let tsnp_dir = std::path::Path::new("tsnp");
    if !tsnp_dir.exists() {
        anyhow::bail!("tsnp/ directory not found. Run this command in a tsn project root.");
    }
    
    let mut total_count = 0;
    
    loop {
        // 列出 tsnp 下的子目录
        let crates: Vec<String> = fs::read_dir(tsnp_dir)
            .context("Failed to read tsnp directory")?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_dir())
            .filter_map(|entry| entry.file_name().to_str().map(|s| s.to_string()))
            .collect();
        
        if crates.is_empty() {
            anyhow::bail!("No crates found in tsnp/. Run 'cargo tsn add <crate>' first.");
        }
        
        // 选择 crate
        println!("\nSelect crate:");
        for (i, crate_name) in crates.iter().enumerate() {
            println!("[{}] {} (tsnp/{}/)", i + 1, crate_name, crate_name);
        }
        println!("[q] Quit");
        
        print!("\nSelect: ");
        io::stdout().flush().context("Failed to flush")?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).context("Failed to read input")?;
        let input = input.trim();
        
        if input == "q" {
            break;
        }
        
        let selected_idx: usize = match input.parse::<usize>() {
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
            io::stdout().flush().context("Failed to flush")?;
            
            let mut func_name = String::new();
            io::stdin().read_line(&mut func_name).context("Failed to read input")?;
            let func_name = func_name.trim();
            
            if func_name == "q" || func_name.is_empty() {
                break;
            }
            
            // 输入参数
            print!("Parameters (e.g., 'a: i32, b: i32'): ");
            io::stdout().flush().context("Failed to flush")?;
            
            let mut params = String::new();
            io::stdin().read_line(&mut params).context("Failed to read input")?;
            let params = params.trim();
            
            // 输入返回值
            print!("Return type (e.g., 'i32'): ");
            io::stdout().flush().context("Failed to flush")?;
            
            let mut ret_type = String::new();
            io::stdin().read_line(&mut ret_type).context("Failed to read input")?;
            let ret_type = ret_type.trim();
            
            // 生成 FFI 函数
            add_ffi_function(selected_crate, func_name, params, ret_type)?;
            total_count += 1;
            
            println!("✅ Added to src/lib.rs");
            println!("✅ Updated tsnp/{}/ts-native.toml", selected_crate);
        }
    }
    
    println!("\nDone. {} FFI function(s) added.", total_count);
    
    Ok(())
}

fn add_ffi_function(crate_name: &str, func_name: &str, params: &str, ret_type: &str) -> Result<()> {
    // 验证函数名合法性
    if !func_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        anyhow::bail!("Invalid function name: {}", func_name);
    }
    
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
    let mut content = fs::read_to_string(lib_rs_path)
        .unwrap_or_default(); // 不存在则创建新文件
    content.push_str(&ffi_code);
    fs::write(lib_rs_path, content).context("Failed to write lib.rs")?;
    
    // 更新 ts-native.toml
    let toml_path = format!("tsnp/{}/ts-native.toml", crate_name);
    ensure_priority_1000(&toml_path)?;
    
    // 读取更新后的内容
    let mut content = fs::read_to_string(&toml_path)
        .context("Failed to read ts-native.toml")?;
    
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
        fs::write(&toml_path, content).context("Failed to write toml")?;
    } else {
        // 如果 [functions] 段不存在，添加它
        content.push_str("\n[functions]\n");
        content.push_str(&func_entry);
        fs::write(&toml_path, content).context("Failed to write toml")?;
    }
    
    Ok(())
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
    
    if ty == "bool" {
        return "boolean".to_string();
    }
    
    if ty.contains("Vec") || ty.contains("slice") {
        return "array".to_string();
    }
    
    if ty.starts_with("struct") || ty.starts_with("&mut") && !ty.contains("c_char") {
        return "object".to_string();
    }
    
    "number".to_string()
}

fn cmd_list() -> Result<()> {
    use std::path::Path;
    
    // 1. 显示开发者自定义插件（tsnp/）
    println!("📦 Developer Plugins (tsnp/):");
    
    let tsnp_dir = Path::new("tsnp");
    if tsnp_dir.exists() {
        let mut found = false;
        for entry in fs::read_dir(tsnp_dir).context("Failed to read tsnp directory")?.filter_map(|e| e.ok()) {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    let toml_path = entry.path().join("ts-native.toml");
                    if toml_path.exists() {
                        if let Ok(content) = fs::read_to_string(&toml_path) {
                            if let Ok(toml_val) = content.parse::<toml::Value>() {
                                let version = toml_val.get("package")
                                    .and_then(|p| p.get("version"))
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("unknown");
                                let priority = toml_val.get("package")
                                    .and_then(|p| p.get("priority"))
                                    .and_then(|p| p.as_float())
                                    .unwrap_or(1000.0);
                                println!("  - {} v{} (priority: {})", name, version, priority);
                                found = true;
                            } else {
                                println!("  - {} (error parsing toml)", name);
                            }
                        }
                    }
                }
            }
        }
        if !found {
            println!("  (no plugins found)");
        }
    } else {
        println!("  (no tsnp/ directory)");
    }
    
    // 2. 显示官方插件（tsnp-contrib/）
    println!("\n🏛️  Official Plugins (tsnp-contrib/):");
    
    // 查找 tsnp-contrib 目录（可能在当前目录或上级目录）
    let contrib_dirs = vec![
        Path::new("tsnp-contrib").to_path_buf(),
        Path::new("../tsnp-contrib").to_path_buf(),
        Path::new("../ts-native/tsnp-contrib").to_path_buf(),
    ];
    
    let mut contrib_dir_found = false;
    for contrib_dir in &contrib_dirs {
        if contrib_dir.exists() {
            let mut found = false;
            for entry in fs::read_dir(contrib_dir).context("Failed to read tsnp-contrib directory")?.filter_map(|e| e.ok()) {
                if entry.path().is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        let toml_path = entry.path().join("ts-native.toml");
                        if toml_path.exists() {
                            if let Ok(content) = fs::read_to_string(&toml_path) {
                                if let Ok(toml_val) = content.parse::<toml::Value>() {
                                    let version = toml_val.get("package")
                                        .and_then(|p| p.get("version"))
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unknown");
                                    let priority = toml_val.get("package")
                                        .and_then(|p| p.get("priority"))
                                        .and_then(|p| p.as_float())
                                        .unwrap_or(0.0);
                                    println!("  - {} v{} (priority: {})", name, version, priority);
                                    found = true;
                                }
                            }
                        }
                    }
                }
            }
            if !found {
                println!("  (no plugins found)");
            }
            contrib_dir_found = true;
            break;
        }
    }
    
    if !contrib_dir_found {
        println!("  (tsnp-contrib/ not found)");
    }
    
    Ok(())
}
