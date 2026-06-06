use anyhow::{Context, Result};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use indicatif::{ProgressBar, ProgressStyle};
use swc_common::{FileName, SourceMap};
use swc_ecma_ast::*;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};
use swc_ecma_visit::{Visit, VisitWith};
use std::sync::Arc;

/// API 到插件的映射表（与 tsnp-contrib 中的实际函数完全对齐）
fn build_api_plugin_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    
    // ===== HTTP 插件 =====
    map.insert("http_get".to_string(), "http".to_string());
    map.insert("http_post".to_string(), "http".to_string());
    map.insert("http_put".to_string(), "http".to_string());
    map.insert("http_delete".to_string(), "http".to_string());
    map.insert("http_head".to_string(), "http".to_string());
    map.insert("http_get_status".to_string(), "http".to_string());
    map.insert("http_get_header".to_string(), "http".to_string());
    map.insert("http_set_timeout".to_string(), "http".to_string());
    
    // ===== FS 插件 =====
    map.insert("file_write".to_string(), "fs".to_string());
    map.insert("file_append".to_string(), "fs".to_string());
    map.insert("file_read".to_string(), "fs".to_string());
    map.insert("file_exists".to_string(), "fs".to_string());
    map.insert("file_size".to_string(), "fs".to_string());
    
    // ===== CRYPTO 插件 =====
    map.insert("crypto_md5".to_string(), "crypto".to_string());
    map.insert("crypto_sha256".to_string(), "crypto".to_string());
    map.insert("crypto_sha1".to_string(), "crypto".to_string());
    map.insert("crypto_crc32".to_string(), "crypto".to_string());
    map.insert("crypto_base64_encode".to_string(), "crypto".to_string());
    map.insert("crypto_base64_decode".to_string(), "crypto".to_string());
    
    // ===== OS 插件 =====
    map.insert("os_type".to_string(), "os".to_string());
    map.insert("os_arch".to_string(), "os".to_string());
    map.insert("os_hostname".to_string(), "os".to_string());
    map.insert("os_homedir".to_string(), "os".to_string());
    map.insert("os_tmpdir".to_string(), "os".to_string());
    map.insert("os_env".to_string(), "os".to_string());
    map.insert("os_setenv".to_string(), "os".to_string());
    map.insert("os_cpus".to_string(), "os".to_string());
    map.insert("os_totalmem".to_string(), "os".to_string());
    map.insert("os_freemem".to_string(), "os".to_string());
    map.insert("os_uptime".to_string(), "os".to_string());
    
    // ===== PATH 插件 =====
    map.insert("path_join".to_string(), "path".to_string());
    map.insert("path_dirname".to_string(), "path".to_string());
    map.insert("path_basename".to_string(), "path".to_string());
    map.insert("path_extname".to_string(), "path".to_string());
    map.insert("path_is_absolute".to_string(), "path".to_string());
    map.insert("path_normalize".to_string(), "path".to_string());
    map.insert("path_cwd".to_string(), "path".to_string());
    map.insert("path_resolve".to_string(), "path".to_string());
    
    // ===== PROCESS 插件 =====
    map.insert("process_pid".to_string(), "process".to_string());
    map.insert("process_ppid".to_string(), "process".to_string());
    map.insert("process_exit".to_string(), "process".to_string());
    map.insert("process_memory".to_string(), "process".to_string());
    map.insert("process_exec".to_string(), "process".to_string());
    map.insert("process_spawn".to_string(), "process".to_string());
    map.insert("process_kill".to_string(), "process".to_string());
    map.insert("process_exists".to_string(), "process".to_string());
    
    // ===== CLI 插件 =====
    map.insert("argc".to_string(), "cli".to_string());
    map.insert("argv".to_string(), "cli".to_string());
    map.insert("now_ms".to_string(), "cli".to_string());
    map.insert("sleep".to_string(), "cli".to_string());
    map.insert("getenv".to_string(), "cli".to_string());
    map.insert("exit".to_string(), "cli".to_string());
    
    // ===== TIMER 插件 =====
    map.insert("timer_now_us".to_string(), "timer".to_string());
    map.insert("timer_measure".to_string(), "timer".to_string());
    map.insert("timer_sleep_us".to_string(), "timer".to_string());
    map.insert("timer_format".to_string(), "timer".to_string());
    
    // ===== JSON 插件 =====
    map.insert("json_parse".to_string(), "json".to_string());
    map.insert("json_stringify".to_string(), "json".to_string());
    map.insert("json_get".to_string(), "json".to_string());
    map.insert("json_set".to_string(), "json".to_string());
    map.insert("json_has".to_string(), "json".to_string());
    map.insert("json_delete".to_string(), "json".to_string());
    map.insert("json_keys".to_string(), "json".to_string());
    map.insert("json_length".to_string(), "json".to_string());
    map.insert("json_validate".to_string(), "json".to_string());
    map.insert("json_pretty".to_string(), "json".to_string());
    
    // ===== NET 插件 =====
    // (暂无官方插件，预留)
    
    // ===== LOG 插件 =====
    map.insert("log_debug".to_string(), "log".to_string());
    map.insert("log_info".to_string(), "log".to_string());
    map.insert("log_warn".to_string(), "log".to_string());
    map.insert("log_error".to_string(), "log".to_string());
    map.insert("log_fatal".to_string(), "log".to_string());
    map.insert("log_set_level".to_string(), "log".to_string());
    map.insert("log_set_file".to_string(), "log".to_string());
    map.insert("log_set_console".to_string(), "log".to_string());
    map.insert("log_get_level".to_string(), "log".to_string());
    map.insert("log_format".to_string(), "log".to_string());
    
    // ===== ENV 插件 =====
    // (功能已包含在 os 插件中)
    
    // ===== MATH 插件 =====
    map.insert("math_abs".to_string(), "math".to_string());
    map.insert("math_floor".to_string(), "math".to_string());
    map.insert("math_ceil".to_string(), "math".to_string());
    map.insert("math_round".to_string(), "math".to_string());
    map.insert("math_pow".to_string(), "math".to_string());
    map.insert("math_sqrt".to_string(), "math".to_string());
    map.insert("math_sin".to_string(), "math".to_string());
    map.insert("math_cos".to_string(), "math".to_string());
    map.insert("math_tan".to_string(), "math".to_string());
    map.insert("math_log".to_string(), "math".to_string());
    map.insert("math_log2".to_string(), "math".to_string());
    map.insert("math_log10".to_string(), "math".to_string());
    map.insert("math_exp".to_string(), "math".to_string());
    map.insert("math_max".to_string(), "math".to_string());
    map.insert("math_min".to_string(), "math".to_string());
    map.insert("math_random".to_string(), "math".to_string());
    map.insert("math_pi".to_string(), "math".to_string());
    map.insert("math_e".to_string(), "math".to_string());
    
    // ===== STRING 插件 =====
    map.insert("str_len".to_string(), "string".to_string());
    map.insert("str_substr".to_string(), "string".to_string());
    map.insert("str_replace".to_string(), "string".to_string());
    map.insert("str_replace_all".to_string(), "string".to_string());
    map.insert("str_find".to_string(), "string".to_string());
    map.insert("str_to_upper".to_string(), "string".to_string());
    map.insert("str_to_lower".to_string(), "string".to_string());
    map.insert("str_trim".to_string(), "string".to_string());
    map.insert("str_trim_left".to_string(), "string".to_string());
    map.insert("str_trim_right".to_string(), "string".to_string());
    map.insert("str_split".to_string(), "string".to_string());
    map.insert("str_join".to_string(), "string".to_string());
    map.insert("str_repeat".to_string(), "string".to_string());
    map.insert("str_starts_with".to_string(), "string".to_string());
    map.insert("str_ends_with".to_string(), "string".to_string());
    map.insert("str_contains".to_string(), "string".to_string());
    map.insert("str_pad_left".to_string(), "string".to_string());
    map.insert("str_pad_right".to_string(), "string".to_string());
    
    map
}

/// AST 访问器，用于检测插件需求
struct PluginDetector {
    api_map: HashMap<String, String>,
    detected_plugins: HashSet<String>,
}

impl PluginDetector {
    fn new() -> Self {
        Self {
            api_map: build_api_plugin_map(),
            detected_plugins: HashSet::new(),
        }
    }
    
    /// 检查函数名是否需要插件
    fn check_function_name(&mut self, name: &str) {
        if let Some(plugin) = self.api_map.get(name) {
            self.detected_plugins.insert(plugin.clone());
        }
    }
    
    /// 检查成员表达式（如 fs.writeFileSync）
    fn check_member_expression(&mut self, member: &MemberExpr) {
        // 检查属性部分（方法名）
        if let MemberProp::Ident(prop) = &member.prop {
            self.check_function_name(&prop.sym);
        }
        
        // 递归检查对象部分（处理 obj.prop.method() 的情况）
        if let Expr::Member(inner) = member.obj.as_ref() {
            self.check_member_expression(inner);
        } else if let Expr::Ident(obj) = member.obj.as_ref() {
            // 也检查对象名（如 fs、crypto、http）
            self.check_function_name(&obj.sym);
        }
    }
}

impl Visit for PluginDetector {
    fn visit_call_expr(&mut self, call: &CallExpr) {
        match &call.callee {
            Callee::Expr(expr) => {
                match expr.as_ref() {
                    // 直接调用: fetch(), sha256()
                    Expr::Ident(ident) => {
                        self.check_function_name(&ident.sym);
                    }
                    // 成员调用: fs.writeFileSync(), crypto.sha256()
                    Expr::Member(member) => {
                        self.check_member_expression(member);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        
        // 继续遍历子节点
        call.visit_children_with(self);
    }
    
    fn visit_new_expr(&mut self, new: &NewExpr) {
        if let Expr::Ident(ident) = new.callee.as_ref() {
            self.check_function_name(&ident.sym);
        }
        new.visit_children_with(self);
    }
}

/// FFI 函数信息
#[derive(Debug, Clone)]
pub struct FFIFunction {
    pub name: String,
    pub params: Vec<ParamInfo>,
    pub return_type: String,
}

#[derive(Debug, Clone)]
pub struct ParamInfo {
    pub name: String,
    pub param_type: String,
}

/// 解析 TypeScript 文件提取 FFI 函数声明
pub fn parse_ffi_declarations(files: &[String]) -> Result<Vec<FFIFunction>> {
    let mut functions = Vec::new();
    
    // 正则表达式匹配 declare function
    let re = Regex::new(
        r#"declare\s+function\s+(\w+)\s*\(([^)]*)\)\s*:\s*(\w+)"#
    ).context("Failed to compile regex")?;
    
    for file in files {
        let content = fs::read_to_string(file)
            .context(format!("Failed to read file: {}", file))?;
        
        for cap in re.captures_iter(&content) {
            let name = cap[1].to_string();
            let params_str = &cap[2];
            let return_type = cap[3].to_string();
            
            // 解析参数
            let params = if params_str.trim().is_empty() {
                Vec::new()
            } else {
                params_str
                    .split(',')
                    .filter_map(|p| {
                        let parts: Vec<&str> = p.trim().split(':').collect();
                        if parts.len() == 2 {
                            Some(ParamInfo {
                                name: parts[0].trim().to_string(),
                                param_type: parts[1].trim().to_string(),
                            })
                        } else {
                            None
                        }
                    })
                    .collect()
            };
            
            functions.push(FFIFunction {
                name,
                params,
                return_type,
            });
        }
    }
    
    Ok(functions)
}

/// 使用 AST 分析 TypeScript 代码，检测需要的插件
pub fn detect_plugins_from_ast(files: &[String]) -> Result<HashSet<String>> {
    let mut all_plugins = HashSet::new();
    let cm: Arc<SourceMap> = Default::default();
    
    for file in files {
        let content = fs::read_to_string(file)
            .context(format!("Failed to read file: {}", file))?;
        
        let fm = cm.new_source_file(
            FileName::Real(file.into()).into(),
            content,
        );
        
        let lexer = Lexer::new(
            Syntax::Typescript(TsSyntax {
                tsx: false,
                decorators: false,
                dts: false,
                no_early_errors: false,
                disallow_ambiguous_jsx_like: false,
            }),
            EsVersion::Es2020,
            StringInput::from(&*fm),
            None,
        );
        
        let mut parser = Parser::new_from(lexer);
        let module = parser.parse_module()
            .map_err(|e| anyhow::anyhow!("Failed to parse {}: {:?}", file, e))?;
        
        let mut detector = PluginDetector::new();
        module.visit_with(&mut detector);
        
        all_plugins.extend(detector.detected_plugins);
    }
    
    Ok(all_plugins)
}

/// 按插件分组（根据函数名前缀）
pub fn group_by_plugin(functions: &[FFIFunction]) -> HashMap<String, Vec<FFIFunction>> {
    let mut groups: HashMap<String, Vec<FFIFunction>> = HashMap::new();
    
    for func in functions {
        // 提取插件名：第一个下划线前的部分
        let plugin_name = if let Some(pos) = func.name.find('_') {
            func.name[..pos].to_string()
        } else {
            // 如果没有下划线，使用 "default"
            "default".to_string()
        };
        
        groups.entry(plugin_name).or_insert_with(Vec::new).push(func.clone());
    }
    
    groups
}

/// 从 groups 生成插件（公共逻辑）
fn generate_plugins_from_groups(
    official_groups: &HashMap<String, Vec<FFIFunction>>,
    custom_groups: &HashMap<String, Vec<FFIFunction>>,
    output: &str,
    dry_run: bool,
    no_stubs: bool,
    ts_files: &[String],
) -> Result<()> {
    let output_path = Path::new(output);
    
    if dry_run {
        println!("\n🔍 Preview mode (no files will be written):\n");
        
        // 官方插件
        if !official_groups.is_empty() {
            println!("📦 Official Plugins (available from tsnp-contrib):");
            println!("  📄 has-tsnp-contrib.txt (list of official plugins)");
            println!("  📁 templates/tsnp/ (empty templates for each plugin)");
            for plugin_name in official_groups.keys() {
                println!("    - {} → templates/tsnp/{}/", plugin_name, plugin_name);
            }
            println!();
        }
        
        // 自定义插件（包括官方没有的）
        if !custom_groups.is_empty() {
            println!("🔧 Custom Plugins (need implementation):");
            for (plugin_name, funcs) in custom_groups {
                let plugin_dir = output_path.join("tsnp").join(plugin_name);
                println!("  📁 {}", plugin_dir.display());
                
                // Windows C 文件
                let win_c = format!("{}_win.c", plugin_name.replace("-", "_"));
                println!("     📄 {}", win_c);
                
                // Linux/macOS C 文件
                println!("     📄 {}_linux.c", plugin_name.replace("-", "_"));
                println!("     📄 {}_macos.c", plugin_name.replace("-", "_"));
                
                // 配置文件
                println!("     📄 ts-native.toml");
                println!("     📄 ts-native-win.toml");
                println!("     📄 ts-native-linux.toml");
                println!("     📄 ts-native-macos.toml");
                
                if !funcs.is_empty() {
                    println!("     📝 Functions:");
                    for func in funcs {
                        let params: Vec<String> = func.params.iter()
                            .map(|p| format!("{}: {}", p.name, p.param_type))
                            .collect();
                        println!("        - {}({}) -> {}", func.name, params.join(", "), func.return_type);
                    }
                } else {
                    println!("     📝 Functions: (will be added manually)");
                }
                println!();
            }
        }
        
        let total_plugins = official_groups.len() + custom_groups.len();
        let total_funcs: usize = official_groups.values().chain(custom_groups.values()).map(|v| v.len()).sum();
        println!("✅ Would prepare {} plugin(s) with {} function(s)", total_plugins, total_funcs);
        println!("   - {} official (ready to use)", official_groups.len());
        println!("   - {} custom (need implementation)", custom_groups.len());
    } else {
        println!("\n⚙️  Generating plugins...\n");
        
        // 检查输出目录是否已存在
        if output_path.exists() {
            anyhow::bail!(
                "Output directory '{}' already exists.\n\
                 Please remove it or specify a different output directory with --output.\n\
                 Example: cargo tsn prepare --output prepare-v2",
                output
            );
        }
        
        // 确保输出目录存在
        fs::create_dir_all(output_path)?;
        
        // 生成官方插件目录（只生成引用，不生成代码）
        if !official_groups.is_empty() {
            println!("📦 Official Plugins:");
            
            // 生成 has-tsnp-contrib.txt 文件
            let contrib_list_path = output_path.join("has-tsnp-contrib.txt");
            let mut contrib_list_content = String::from("# Official Plugins (from tsnp-contrib)\n\n");
            contrib_list_content.push_str("These plugins are already implemented. To use them:\n");
            contrib_list_content.push_str("1. Copy template from prepare/templates/tsnp/<plugin>/ to your project's tsnp/<plugin>/\n");
            contrib_list_content.push_str("2. Add to your .ts.toml: tsnp = [\"<plugin>\"]\n\n");
            contrib_list_content.push_str("## Available Plugins\n\n");
            
            for plugin_name in official_groups.keys() {
                contrib_list_content.push_str(&format!("- {}\n", plugin_name));
            }
            
            fs::write(&contrib_list_path, contrib_list_content)?;
            println!("  ✓ has-tsnp-contrib.txt ({} plugins)", official_groups.len());
            
            // 为每个官方插件生成空模板
            let templates_dir = output_path.join("templates").join("tsnp");
            for plugin_name in official_groups.keys() {
                let template_dir = templates_dir.join(plugin_name);
                fs::create_dir_all(&template_dir)?;
                
                // 生成空的 Windows C 文件（注释模板）
                let win_c_path = template_dir.join(format!("{}_win.c", plugin_name.replace("-", "_")));
                let win_c_content = format!(
                    "// ============================================================\n\
                     // {} Plugin - Windows Implementation\n\
                     // ============================================================\n\
                     // This is an empty template. The actual implementation is in tsnp-contrib/.\n\
                     // To use the official plugin, copy it from tsnp-contrib/{}// to your project.\n\
                     // ============================================================\n",
                    plugin_name, plugin_name
                );
                fs::write(&win_c_path, win_c_content)?;
                
                // 生成空的 Linux C 文件
                let linux_c_path = template_dir.join(format!("{}_linux.c", plugin_name.replace("-", "_")));
                let linux_c_content = format!(
                    "// ============================================================\n\
                     // {} Plugin - Linux Implementation\n\
                     // ============================================================\n\
                     // TODO: Implement for Linux\n\
                     // ============================================================\n",
                    plugin_name
                );
                fs::write(&linux_c_path, linux_c_content)?;
                
                // 生成空的 macOS C 文件
                let macos_c_path = template_dir.join(format!("{}_macos.c", plugin_name.replace("-", "_")));
                let macos_c_content = format!(
                    "// ============================================================\n\
                     // {} Plugin - macOS Implementation\n\
                     // ============================================================\n\
                     // TODO: Implement for macOS\n\
                     // ============================================================\n",
                    plugin_name
                );
                fs::write(&macos_c_path, macos_c_content)?;
                
                // 生成配置文件
                let toml_path = template_dir.join("ts-native.toml");
                let toml_content = format!(
                    "[package]\n\
                     name = \"{}\"\n\
                     version = \"0.1.0\"\n\
                     priority = 0\n\
                     \n\
                     [capabilities]\n\
                     functions = []\n",
                    plugin_name
                );
                fs::write(&toml_path, toml_content)?;
                
                println!("  ✓ templates/tsnp/{}/ (empty template)", plugin_name);
            }
            println!();
        }
        
        // 生成自定义插件
        if !custom_groups.is_empty() {
            println!("🔧 Custom Plugins:");
            for (plugin_name, funcs) in custom_groups {
                generate_plugin(plugin_name, funcs, output, no_stubs)?;
                println!();
            }
        }
        
        // 生成 .ts.toml 文件（为每个有 main 函数的 TS 文件生成）
        generate_ts_toml(ts_files, output)?;
        
        let total_plugins = official_groups.len() + custom_groups.len();
        let total_funcs: usize = official_groups.values().chain(custom_groups.values()).map(|v| v.len()).sum();
        println!("✅ Prepared {} plugin(s) with {} function(s)", total_plugins, total_funcs);
        println!("   - {} official (copy from tsnp-contrib/)", official_groups.len());
        println!("   - {} custom (implement in tsnp/)", custom_groups.len());
    }
    
    Ok(())
}

/// 使用 AST 检测是否包含 main 函数
fn has_main_function(ts_file: &str) -> Result<bool> {
    let content = fs::read_to_string(ts_file)
        .context(format!("Failed to read file: {}", ts_file))?;
    
    let cm: Arc<SourceMap> = Default::default();
    let fm = cm.new_source_file(
        FileName::Real(ts_file.into()).into(),
        content,
    );
    
    let lexer = Lexer::new(
        Syntax::Typescript(TsSyntax {
            tsx: false,
            decorators: false,
            dts: false,
            no_early_errors: false,
            disallow_ambiguous_jsx_like: false,
        }),
        EsVersion::Es2020,
        StringInput::from(&*fm),
        None,
    );
    
    let mut parser = Parser::new_from(lexer);
    let module = parser.parse_module()
        .map_err(|e| anyhow::anyhow!("Failed to parse {}: {:?}", ts_file, e))?;
    
    struct MainDetector { found: bool }
    
    impl Visit for MainDetector {
        fn visit_fn_decl(&mut self, decl: &FnDecl) {
            if decl.ident.sym == "main" {
                self.found = true;
            }
        }
    }
    
    let mut detector = MainDetector { found: false };
    module.visit_with(&mut detector);
    
    Ok(detector.found)
}

/// 生成 .ts.toml 文件（为每个包含 main 函数的 TS 文件生成依赖声明，输出到项目根目录）
fn generate_ts_toml(ts_files: &[String], _output: &str) -> Result<()> {
    use std::path::Path;
    
    // 为每个包含 main 函数的 TS 文件生成 .ts.toml
    for ts_file in ts_files {
        // 使用 AST 检测是否包含 main 函数
        let has_main = has_main_function(ts_file).unwrap_or(false);
        
        if !has_main {
            continue;
        }
        
        // 分析这个文件需要哪些插件
        let file_plugins = detect_plugins_from_ast(&[ts_file.clone()])?;
        
        let path = Path::new(ts_file);
        let file_stem = path.file_stem()
            .ok_or_else(|| anyhow::anyhow!("Invalid file name: {}", ts_file))?;
        
        // 生成到 TS 文件所在目录（项目根目录），而非 prepare/ 目录
        let ts_dir = path.parent().unwrap_or(Path::new("."));
        let toml_name = format!("{}.ts.toml", file_stem.to_string_lossy());
        let toml_path = ts_dir.join(&toml_name);
        
        // 过滤出这个文件需要的插件
        let plugins: Vec<String> = file_plugins.into_iter()
            .filter(|k| k != "default")
            .collect();
        
        // 即使没有检测到插件，也生成空的 .ts.toml（入口文件标记）
        let toml_content = if plugins.is_empty() {
            format!(
                "# Entry point: {}\n# No plugin dependencies detected\n\n[dependencies]\ntsnp = []\n",
                file_stem.to_string_lossy()
            )
        } else {
            format!(
                "[dependencies]\ntsnp = [{}]\n",
                plugins.iter()
                    .map(|p| format!("\"{}\"", p))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        
        fs::write(&toml_path, &toml_content)
            .context(format!("Failed to write {}", toml_path.display()))?;
        
        if plugins.is_empty() {
            println!("  ✓ {} (project root, no plugins)", toml_name);
        } else {
            println!("  ✓ {} (project root)", toml_name);
        }
    }
    
    Ok(())
}

/// 生成插件文件
pub fn generate_plugin(plugin_name: &str, functions: &[FFIFunction], output_dir: &str, no_stubs: bool) -> Result<()> {
    let plugin_dir = format!("{}/tsnp/{}", output_dir, plugin_name);
    fs::create_dir_all(&plugin_dir)?;
    
    let total_steps = 7; // 7 个文件
    let pb = ProgressBar::new(total_steps);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
        .unwrap()
        .progress_chars("#>-"));
    
    pb.set_message(format!("Generating {}", plugin_name));
    
    // 1. 生成 C 文件（win/linux/macos）
    generate_c_file(&plugin_dir, plugin_name, functions, "win", no_stubs)?;
    pb.inc(1);
    pb.set_message(format!("  ✓ {}_win.c", plugin_name));
    
    generate_c_file(&plugin_dir, plugin_name, functions, "linux", no_stubs)?;
    pb.inc(1);
    pb.set_message(format!("  ✓ {}_linux.c", plugin_name));
    
    generate_c_file(&plugin_dir, plugin_name, functions, "macos", no_stubs)?;
    pb.inc(1);
    pb.set_message(format!("  ✓ {}_macos.c", plugin_name));
    
    // 2. 生成配置文件
    generate_main_toml(&plugin_dir, plugin_name, functions)?;
    pb.inc(1);
    pb.set_message("  ✓ ts-native.toml");
    
    generate_platform_toml(&plugin_dir, plugin_name, functions, "win")?;
    pb.inc(1);
    pb.set_message("  ✓ ts-native-win.toml");
    
    generate_platform_toml(&plugin_dir, plugin_name, functions, "linux")?;
    pb.inc(1);
    pb.set_message("  ✓ ts-native-linux.toml");
    
    generate_platform_toml(&plugin_dir, plugin_name, functions, "macos")?;
    pb.inc(1);
    pb.set_message("  ✓ ts-native-macos.toml");
    
    pb.finish_with_message(format!("✅ Plugin '{}' generated", plugin_name));
    
    Ok(())
}

/// 生成 C 实现文件
fn generate_c_file(plugin_dir: &str, plugin_name: &str, functions: &[FFIFunction], platform: &str, no_stubs: bool) -> Result<()> {
    let file_name = format!("{}_{}.c", plugin_name.replace("-", "_"), platform);
    let file_path = format!("{}/{}", plugin_dir, file_name);
    
    let mut content = format!(
        "// {} Implementation for {}\n\
         // Auto-generated by cargo tsn prepare\n\
         // TODO: Implement your platform-specific functions here\n\n",
        get_platform_name(platform),
        plugin_name
    );
    
    // 仅为 Windows 平台生成头文件和函数声明模板
    if platform == "win" {
        content.push_str("#include <windows.h>\n\n");
        content.push_str("// Runtime external functions\n");
        content.push_str("extern double js_string_new(const char* data, unsigned int len);\n");
        content.push_str("extern const char* js_string_unpack(double val);\n");
        content.push_str("extern double js_number_new(double val);\n");
        content.push_str("extern double js_boolean_new(int val);\n");
        content.push_str("extern double js_undefined();\n");
        content.push_str("extern double js_null();\n");
        content.push_str("extern double js_undefined_public();\n");
        content.push_str("extern double js_array_new(double capacity);\n");
        content.push_str("extern double js_object_new();\n");
        content.push_str("extern double console_log(double msg_val);\n");
        content.push_str("extern double console_debug(double msg_val);\n");
        content.push_str("extern double console_err(double msg_val);\n\n");
        
        // 生成函数实现
        if no_stubs {
            // 仅生成注释模板
            content.push_str("// Function templates (copy and implement as needed):\n\n");
            for func in functions {
                content.push_str(&format!("// {}\n", "=".repeat(60)));
                content.push_str(&format!("// {}({}) -> {}\n", func.name, 
                    func.params.iter().map(|p| format!("{}: {}", p.name, p.param_type)).collect::<Vec<_>>().join(", "),
                    func.return_type
                ));
                content.push_str("//\n");
                content.push_str(&format!("// double {}({}) {{\n", func.name,
                    (0..func.params.len()).map(|i| format!("double p{}", i)).collect::<Vec<_>>().join(", ")
                ));
                content.push_str("//     // TODO: Implement your logic here\n");
                content.push_str("//     return 0;\n");
                content.push_str("// }\n\n");
            }
        } else {
            // 生成带 console 警告的桩函数
            content.push_str("// Function implementations (stub with console warnings):\n\n");
            for func in functions {
                content.push_str(&generate_function_stub_with_console(func));
            }
        }
    } else {
        // Linux/macOS 占位文件
        content.push_str(&format!(
            "// TODO: Implement {} functions for {}\n\
             // This is a placeholder file.\n\
             // Add your platform-specific implementations here.\n",
            plugin_name,
            get_platform_name(platform)
        ));
    }
    
    fs::write(&file_path, content)?;
    Ok(())
}

/// 生成单个函数的 C 实现（保留以备将来使用）
#[allow(dead_code)]
fn generate_function_impl(func: &FFIFunction) -> String {
    let mut impl_str = String::new();
    
    // 函数签名 - 所有参数都是 double 类型
    let param_decls: Vec<String> = (0..func.params.len())
        .map(|i| format!("double p{}", i))
        .collect();
    
    impl_str.push_str(&format!("double {}({}) {{\n", func.name, param_decls.join(", ")));
    
    // 参数解包（标记为未使用以避免编译警告）
    for (i, param) in func.params.iter().enumerate() {
        let param_name = format!("p{}", i);
        let local_name = param.name.clone();
        
        match param.param_type.as_str() {
            "string" => {
                impl_str.push_str(&format!(
                    "    const char* {} = js_string_unpack({});\n",
                    local_name, param_name
                ));
                impl_str.push_str(&format!("    (void){}; // Stub: parameter unpacked but not used\n", local_name));
            }
            "number" => {
                impl_str.push_str(&format!(
                    "    double {} = {};\n",
                    local_name, param_name
                ));
                impl_str.push_str(&format!("    (void){}; // Stub: parameter not used\n", local_name));
            }
            "boolean" => {
                impl_str.push_str(&format!(
                    "    int {} = (int){};\n",
                    local_name, param_name
                ));
                impl_str.push_str(&format!("    (void){}; // Stub: parameter not used\n", local_name));
            }
            _ => {
                impl_str.push_str(&format!(
                    "    // TODO: Unpack parameter '{}' of type '{}'\n",
                    local_name, param.param_type
                ));
            }
        }
    }
    
    impl_str.push_str("\n");
    impl_str.push_str("    // ⚠️ STUB: Replace with real implementation\n");
    impl_str.push_str(&format!("    console_err(\"⚠️ {} called (stub implementation)\");\n", func.name));
    
    // 添加参数调试输出
    if !func.params.is_empty() {
        impl_str.push_str(&format!("    console_debug(\"  Parameters:\");\n"));
        for param in &func.params {
            impl_str.push_str(&format!("    console_debug(\"    {}: \");\n", param.name));
            impl_str.push_str(&format!("    console_debug({});\n", param.name));
        }
    }
    
    impl_str.push_str("\n");
    
    // 返回值打包（桩实现）
    match func.return_type.as_str() {
        "string" => {
            impl_str.push_str("    // Stub: return empty string\n");
            impl_str.push_str("    return js_string_new(\"\", 0);\n");
        }
        "number" => {
            impl_str.push_str("    // Stub: return 0\n");
            impl_str.push_str("    return 0.0;\n");
        }
        "boolean" => {
            impl_str.push_str("    // Stub: return false\n");
            impl_str.push_str("    return 0;\n");
        }
        "array" => {
            impl_str.push_str("    // Stub: return empty array []\n");
            impl_str.push_str("    return js_array_new(0);\n");
        }
        "object" => {
            impl_str.push_str("    // Stub: return empty object {}\n");
            impl_str.push_str("    return js_object_new();\n");
        }
        "null" => {
            impl_str.push_str("    // Stub: return null\n");
            impl_str.push_str("    return js_null();\n");
        }
        "undefined" => {
            impl_str.push_str("    // Stub: return undefined\n");
            impl_str.push_str("    return js_undefined_public();\n");
        }
        "void" => {
            impl_str.push_str("    // Stub: void function\n");
            impl_str.push_str("    return 0;\n");
        }
        _ => {
            impl_str.push_str(&format!(
                "    // TODO: Pack return value of type '{}'\n",
                func.return_type
            ));
            impl_str.push_str("    return 0;\n");
        }
    }
    
    impl_str.push_str("}\n");
    
    impl_str
}

/// 生成主配置文件
fn generate_main_toml(plugin_dir: &str, plugin_name: &str, functions: &[FFIFunction]) -> Result<()> {
    let file_path = format!("{}/ts-native.toml", plugin_dir);
    
    // 自定义插件默认优先级 1000
    let priority = 1000.0;
    
    let mut content = format!(
        "# ts-native Plugin Configuration\n\
         # Auto-generated by cargo tsn prepare\n\
         # Plugin: {}\n\n\
         [package]\n\
         name = \"tsnp-{}\"\n\
         version = \"0.1.0\"\n\
         priority = {}\n\n",
        plugin_name, plugin_name, priority
    );
    
    content.push_str("[includes]\n");
    content.push_str("win = \"ts-native-win.toml\"\n");
    content.push_str("linux = \"ts-native-linux.toml\"\n");
    content.push_str("macos = \"ts-native-macos.toml\"\n\n");
    
    content.push_str("[signatures]\n");
    for func in functions {
        let params: Vec<String> = func.params.iter()
            .map(|p| format!("{}: {}", p.name, p.param_type))
            .collect();
        
        content.push_str(&format!(
            "\"{}\" = \"function({}): {}\"\n",
            func.name,
            params.join(", "),
            func.return_type
        ));
    }
    
    content.push_str("\n[build]\n");
    content.push_str("warn_on_missing = true\n");
    content.push_str("error_on_mismatch = true\n");
    
    fs::write(&file_path, content)?;
    Ok(())
}

/// 生成平台配置文件
fn generate_platform_toml(
    plugin_dir: &str,
    plugin_name: &str,
    functions: &[FFIFunction],
    platform: &str
) -> Result<()> {
    let file_path = format!("{}/ts-native-{}.toml", plugin_dir, platform);
    
    let (os_name, arch) = match platform {
        "win" => ("Windows", vec!["x86_64", "arm64"]),
        "linux" => ("Linux", vec!["x86_64", "arm64"]),
        "macos" => ("macOS", vec!["x86_64", "arm64"]),
        _ => ("Unknown", vec!["x86_64"]),
    };
    
    let arch_str = arch.iter()
        .map(|a| format!("\"{}\"", a))
        .collect::<Vec<_>>()
        .join(", ");
    
    let mut content = format!(
        "# {} Platform Configuration\n\
         # Auto-generated by cargo tsn prepare for plugin '{}'\n\n\
         [platform]\n\
         name = \"{}\"\n\
         os = \"{}\"\n\
         arch = [{}]\n\
         description = \"{} platform\"\n\
         author = \"ts-native user\"\n\n",
        os_name,
        plugin_name,
        os_name,
        platform,
        arch_str,
        os_name
    );
    
    content.push_str("[libs.default]\n");
    content.push_str("description = \"Default library\"\n");
    content.push_str("required = true\n\n");
    
    content.push_str("[libs.default.functions]\n");
    for func in functions {
        let c_func_name = func.name.clone();
        let system = if platform == "win" { "SystemAPI" } else { "POSIX" };
        
        let args: Vec<String> = func.params.iter()
            .map(|p| format!("\"{}\"", p.param_type))
            .collect();
        
        content.push_str(&format!(
            "\"{}\" = {{\n",
            func.name
        ));
        content.push_str(&format!("    impl = \"{}\",\n", c_func_name));
        content.push_str("    enabled = true,\n");
        content.push_str(&format!("    system = \"{}\",\n", system));
        content.push_str(&format!("    args = [{}],\n", args.join(", ")));
        content.push_str(&format!("    ret = \"{}\",\n", func.return_type));
        content.push_str("    description = \"TODO: Add description\"\n");
        content.push_str("}\n\n");
    }
    
    fs::write(&file_path, content)?;
    Ok(())
}

/// 获取平台名称
fn get_platform_name(platform: &str) -> &'static str {
    match platform {
        "win" => "Windows",
        "linux" => "Linux",
        "macos" => "macOS",
        _ => "Unknown",
    }
}

/// 查找 TypeScript 文件
pub fn find_ts_files(dir: &str) -> Result<Vec<String>> {
    let mut files = Vec::new();
    
    if Path::new(dir).exists() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "ts" {
                        if let Some(path_str) = path.to_str() {
                            files.push(path_str.to_string());
                        }
                    }
                }
            }
        }
    }
    
    Ok(files)
}

/// 官方插件列表（来自 tsnp-contrib）
fn get_official_plugins() -> HashSet<String> {
    let mut plugins = HashSet::new();
    plugins.insert("http".to_string());
    plugins.insert("fs".to_string());
    plugins.insert("crypto".to_string());
    plugins.insert("os".to_string());
    plugins.insert("path".to_string());
    plugins.insert("cli".to_string());
    plugins.insert("timer".to_string());
    plugins.insert("json".to_string());
    plugins.insert("net".to_string());
    plugins.insert("process".to_string());
    plugins.insert("log".to_string());
    plugins.insert("env".to_string());
    plugins
}

/// prepare 命令主入口
pub fn cmd_prepare(input: Option<&str>, output: &str, dry_run: bool, no_stubs: bool) -> Result<()> {
    println!("📦 Analyzing TypeScript files...");
    
    // 1. 查找 TS 文件
    let ts_files = if let Some(file) = input {
        vec![file.to_string()]
    } else {
        find_ts_files(".")?
    };
    
    if ts_files.is_empty() {
        anyhow::bail!("No TypeScript files found in current directory");
    }
    
    println!("  ✓ Found: {}", ts_files.join(", "));
    
    // 2. 使用 AST 分析检测插件需求
    println!("\n🔍 Analyzing code with AST...");
    let detected_plugins = detect_plugins_from_ast(&ts_files)?;
    
    if detected_plugins.is_empty() {
        // 如果没有检测到插件，尝试使用旧的 declare function 方式
        println!("  ⚠️  No plugin APIs detected via AST, trying declare function parsing...");
        let functions = parse_ffi_declarations(&ts_files)?;
        
        if functions.is_empty() {
            anyhow::bail!("No FFI function declarations or plugin API usage found");
        }
        
        println!("  ✓ Parsed {} FFI functions via declare statements", functions.len());
        
        // 按插件分组
        let groups = group_by_plugin(&functions);
        
        // 分类：官方 vs 自定义
        let official_plugins = get_official_plugins();
        let (official_groups, custom_groups): (HashMap<_, _>, HashMap<_, _>) = groups.into_iter()
            .partition(|(name, _)| official_plugins.contains(name));
        
        // 生成插件
        generate_plugins_from_groups(&official_groups, &custom_groups, output, dry_run, no_stubs, &ts_files)?;
    } else {
        println!("  ✓ Detected {} plugin(s) via AST analysis:", detected_plugins.len());
        for plugin in &detected_plugins {
            println!("    - {}", plugin);
        }
        
        // 构建虚拟的 groups 用于生成
        let mut groups: HashMap<String, Vec<FFIFunction>> = HashMap::new();
        for plugin in &detected_plugins {
            groups.insert(plugin.clone(), vec![]);
        }
        
        // 分类：官方 vs 自定义
        let official_plugins = get_official_plugins();
        let (official_groups, custom_groups): (HashMap<_, _>, HashMap<_, _>) = groups.into_iter()
            .partition(|(name, _)| official_plugins.contains(name));
        
        // 生成插件
        generate_plugins_from_groups(&official_groups, &custom_groups, output, dry_run, no_stubs, &ts_files)?;
    }
    
    Ok(())
}

/// 生成带 console 警告的桩函数
fn generate_function_stub_with_console(func: &FFIFunction) -> String {
    let mut impl_str = String::new();
    
    // 函数签名
    let param_decls: Vec<String> = (0..func.params.len())
        .map(|i| format!("double p{}", i))
        .collect();
    
    impl_str.push_str(&format!("// ============================================================\n"));
    impl_str.push_str(&format!("// {}({}) -> {}\n", 
        func.name,
        func.params.iter().map(|p| format!("{}: {}", p.name, p.param_type)).collect::<Vec<_>>().join(", "),
        func.return_type
    ));
    impl_str.push_str(&format!("// ⚠️ STUB: Returns default value, replace with real implementation\n"));
    impl_str.push_str(&format!("// ============================================================\n"));
    impl_str.push_str(&format!("double {}({}) {{\n", func.name, param_decls.join(", ")));
    
    // 参数解包
    for (i, param) in func.params.iter().enumerate() {
        let param_name = format!("p{}", i);
        let local_name = param.name.clone();
        
        match param.param_type.as_str() {
            "string" => {
                impl_str.push_str(&format!(
                    "    const char* {} = js_string_unpack({});\n",
                    local_name, param_name
                ));
            }
            "number" => {
                impl_str.push_str(&format!(
                    "    double {} = {};\n",
                    local_name, param_name
                ));
            }
            "boolean" => {
                impl_str.push_str(&format!(
                    "    int {} = (int){};\n",
                    local_name, param_name
                ));
            }
            _ => {
                impl_str.push_str(&format!(
                    "    // TODO: Unpack parameter '{}' of type '{}'\n",
                    local_name, param.param_type
                ));
            }
        }
    }
    
    impl_str.push_str("\n");
    impl_str.push_str("    // ⚠️ STUB: Replace with real implementation\n");
    impl_str.push_str(&format!("    console_err(\"⚠️ {} called (stub implementation)\");\n", func.name));
    
    // 添加参数调试输出
    if !func.params.is_empty() {
        impl_str.push_str("    console_debug(\"  Parameters:\");\n");
        for param in &func.params {
            impl_str.push_str(&format!("    console_debug(\"    {}: \");\n", param.name));
            if param.param_type == "string" {
                impl_str.push_str(&format!("    console_debug({});\n", param.name));
            } else {
                impl_str.push_str(&format!("    // TODO: Debug output for {}\n", param.name));
            }
        }
    }
    
    impl_str.push_str("\n");
    
    // 返回值打包（桩实现）
    match func.return_type.as_str() {
        "string" => {
            impl_str.push_str("    // Stub: return empty string\n");
            impl_str.push_str("    return js_string_new(\"\", 0);\n");
        }
        "number" => {
            impl_str.push_str("    // Stub: return 0\n");
            impl_str.push_str("    return 0.0;\n");
        }
        "boolean" => {
            impl_str.push_str("    // Stub: return false\n");
            impl_str.push_str("    return 0;\n");
        }
        "array" => {
            impl_str.push_str("    // Stub: return empty array []\n");
            impl_str.push_str("    return js_array_new(0);\n");
        }
        "object" => {
            impl_str.push_str("    // Stub: return empty object {}\n");
            impl_str.push_str("    return js_object_new();\n");
        }
        "null" => {
            impl_str.push_str("    // Stub: return null\n");
            impl_str.push_str("    return js_null();\n");
        }
        "undefined" => {
            impl_str.push_str("    // Stub: return undefined\n");
            impl_str.push_str("    return js_undefined_public();\n");
        }
        "void" => {
            impl_str.push_str("    // Stub: void function\n");
            impl_str.push_str("    return 0;\n");
        }
        _ => {
            impl_str.push_str(&format!(
                "    console_err(\"⚠️ Unknown return type '{}', returning 0\");\n",
                func.return_type
            ));
            impl_str.push_str("    return 0;\n");
        }
    }
    
    impl_str.push_str("}\n\n");
    impl_str
}
