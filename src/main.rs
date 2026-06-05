use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, Write};
use std::process::Command;
use anyhow::{Context, Result};

mod publish;
mod prepare;

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
    #[command(about = "Interactively add FFI function to existing plugin")]
    Func,
    // #[command(about = "Publish plugins to codeberg (DISABLED)")]
    // Publish {
    //     #[arg(long, help = "Show what would be published without actually publishing")]
    //     dry_run: bool,
    // },
    #[command(about = "List local plugins")]
    List,
    // #[command(about = "Install a published plugin from Codeberg (DISABLED)")]
    // Install {
    //     name: String,
    //     #[arg(long, help = "Specific version to install")]
    //     version: Option<String>,
    // },
    #[command(about = "Generate C stubs from TypeScript FFI declarations")]
    Prepare {
        #[arg(long, help = "Input TypeScript file(s)")]
        input: Option<String>,
        #[arg(long, help = "Output directory (default: ./prepared for no args, or specified dir)")]
        output: Option<String>,
        #[arg(long, help = "Preview mode (no files will be written)")]
        dry_run: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::New { name } => cmd_new(&name),
        Commands::Add { crate_name } => cmd_add(&crate_name),
        Commands::Func => cmd_func(),
        // Commands::Publish { dry_run } => cmd_publish(dry_run),  // DISABLED
        Commands::List => cmd_list(),
        // Commands::Install { name, version } => cmd_install(&name, version.as_deref()),  // DISABLED
        Commands::Prepare { input, output, dry_run } => {
            let input_ref = input.as_deref();
            // 无参时默认输出到 ./prepare 目录
            let output_dir = output.as_deref().unwrap_or("prepare");
            prepare::cmd_prepare(input_ref, output_dir, dry_run)
        }
    }
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

fn cmd_add(crate_name: &str) -> Result<()> {
    println!("Adding crate: {}", crate_name);
    
    // 1. cargo add
    println!("Running: cargo add {}", crate_name);
    let status = Command::new("cargo")
        .args(["add", crate_name])
        .status()
        .context("Failed to run cargo add")?;
    
    if !status.success() {
        anyhow::bail!("cargo add failed");
    }
    
    // 2. Check if tsnp/ directory exists and list existing tsnps
    let tsnp_dir = std::path::Path::new("tsnp");
    let mut use_existing = false;
    let mut existing_tsnp_name = String::new();
    
    if tsnp_dir.exists() {

        match publish::fetch_published_tsnps() {
            Ok(tsnps) if !tsnps.is_empty() => {
                println!("\n📦 Published tsnps available:");
                for (i, (name, version, time)) in tsnps.iter().enumerate() {
                    println!("  [{}] {} v{} (published: {})", i + 1, name, version, time);
                }
                println!("  [n] Create new tsnp for {}", crate_name);
                println!("[q] Cancel");
                
                print!("\nSelect an existing tsnp or create new: ");
                io::stdout().flush().context("Failed to flush")?;
                
                let mut input = String::new();
                io::stdin().read_line(&mut input).context("Failed to read input")?;
                let input = input.trim();
                
                if input == "q" {
                    println!("Cancelled.");
                    return Ok(());
                } else if input == "n" {
                    use_existing = false;
                } else if let Ok(idx) = input.parse::<usize>() {
                    if idx >= 1 && idx <= tsnps.len() {
                        use_existing = true;
                        existing_tsnp_name = tsnps[idx - 1].0.clone();
                    } else {
                        eprintln!("Invalid selection. Creating new tsnp.");
                        use_existing = false;
                    }
                } else {
                    eprintln!("Invalid input. Creating new tsnp.");
                    use_existing = false;
                }
            }
            _ => {
                // No published tsnps or error, proceed with new
                use_existing = false;
            }
        }
    }
    
    // 3. Generate tsnp configuration
    if use_existing {
        println!("\nUsing existing tsnp: {}", existing_tsnp_name);
        if let Err(e) = publish::download_tsnp(&existing_tsnp_name, None) {
            eprintln!("Failed to download tsnp: {:#}", e);
            eprintln!("Falling back to tsnp gen...");
            let status = Command::new("tsnp")
                .args(["gen", crate_name])
                .status()
                .context("Failed to run tsnp gen")?;
            if !status.success() {
                eprintln!("tsnp gen failed (crate may have no FFI functions)");
            }
        }
    } else {
        // Generate new tsnp
        println!("\nGenerating new tsnp for: {}", crate_name);
        println!("Running: tsnp gen {}", crate_name);
        let status = Command::new("tsnp")
            .args(["gen", crate_name])
            .status()
            .context("Failed to run tsnp gen")?;
        
        if !status.success() {
            eprintln!("⚠️  tsnp gen failed (crate may have no FFI functions)");
        } else {
            // 设置默认优先级为 1000
            let toml_path = format!("tsnp/{}/ts-native.toml", crate_name);
            if let Ok(mut content) = fs::read_to_string(&toml_path) {
                // 替换或添加 priority 字段
                if content.contains("priority =") {
                    // 替换现有 priority
                    content = content
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
                        // 找到下一个段或文件结尾
                        let next_section = content[insert_pos..].find('\n')
                            .map(|p| insert_pos + p)
                            .unwrap_or(content.len());
                        content.insert_str(next_section, "\npriority = 1000");
                    }
                }
                let _ = fs::write(&toml_path, content);
            }
        }
    }
    
    println!("\n✅ Added crate: {}", crate_name);
    println!("📝 Next steps:");
    println!("   1. Edit tsnp/{}/ts-native.toml to configure function mappings", crate_name);
    println!("   2. Implement FFI functions in src/lib.rs if needed");
    println!("   3. Run 'cargo tsn publish' to publish the plugin");
    
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
    if let Ok(mut content) = fs::read_to_string(lib_rs_path) {
        content.push_str(&ffi_code);
        fs::write(lib_rs_path, content).context("Failed to write lib.rs")?;
    }
    
    // 更新 ts-native.toml
    let toml_path = format!("tsnp/{}/ts-native.toml", crate_name);
    if let Ok(mut content) = fs::read_to_string(&toml_path) {
        // 确保优先级为 1000
        if content.contains("priority =") {
            content = content
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
            if let Some(pos) = content.find("[package]\n") {
                let insert_pos = pos + "[package]\n".len();
                let next_section = content[insert_pos..].find('\n')
                    .map(|p| insert_pos + p)
                    .unwrap_or(content.len());
                content.insert_str(next_section, "\npriority = 1000");
            }
        }
        
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
        }
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

fn cmd_publish(dry_run: bool) -> Result<()> {
    publish::cmd_publish(dry_run)
}

fn cmd_list() -> Result<()> {
    publish::cmd_list()
}

fn cmd_install(name: &str, version: Option<&str>) -> Result<()> {
    publish::download_tsnp(name, version)
}
