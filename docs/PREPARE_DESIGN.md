# prepare 命令详细设计

> 本文档包含 `cargo tsn prepare` 命令的技术实现细节、架构设计和原理说明。

## 架构设计

### 整体流程

```
TypeScript 源码
    ↓
[AST 解析器] (swc)
    ↓
函数调用检测 + 插件映射
    ↓
[插件分组]
    ↓
┌─────────────────┐
│ 官方插件        │ → templates/tsnp/<plugin>/
│ 自定义插件      │ → tsnp/<plugin>/
│ 依赖声明        │ → <file>.ts.toml
└─────────────────┘
```

### 核心组件

1. **AST 解析器** - 使用 swc 解析 TypeScript
2. **插件检测器** - 分析函数调用，映射到插件
3. **官方/自定义分类** - 分离官方插件和自定义插件
4. **模板生成器** - 生成 C 文件和配置文件
5. **依赖声明生成器** - 生成 .ts.toml 文件

## AST 插件检测

### API 映射表

```rust
fn build_api_plugin_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    
    // HTTP 相关
    map.insert("fetch".to_string(), "http".to_string());
    map.insert("http_get".to_string(), "http".to_string());
    map.insert("http_post".to_string(), "http".to_string());
    
    // 文件系统
    map.insert("writeFileSync".to_string(), "fs".to_string());
    map.insert("fs_writeFile".to_string(), "fs".to_string());
    
    // 加密
    map.insert("sha256".to_string(), "crypto".to_string());
    map.insert("crypto_sha256".to_string(), "crypto".to_string());
    
    // ... 更多映射
    map
}
```

### 成员表达式检测

```rust
fn check_member_expression(&mut self, member: &MemberExpr) {
    // 检查属性部分（方法名）
    if let MemberProp::Ident(prop) = &member.prop {
        self.check_function_name(&prop.sym);
    }
    
    // 递归检查对象部分
    if let Expr::Member(inner) = member.obj.as_ref() {
        self.check_member_expression(inner);
    } else if let Expr::Ident(obj) = member.obj.as_ref() {
        // 也检查对象名（如 fs、crypto、http）
        self.check_function_name(&obj.sym);
    }
}
```

### 入口文件检测

使用 AST 分析检测 main 函数：

```rust
fn has_main_function(ts_file: &str) -> Result<bool> {
    // 解析 AST
    let module = parse_typescript(ts_file)?;
    
    // 遍历查找 main 函数声明
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
```

## 官方插件分类

### 官方插件列表

来自 tsnp-contrib 子模块的 12 个官方插件：

- http, fs, crypto, os, path, cli
- timer, json, net, process, log, env

### 目录结构

```
prepare/
├── has-tsnp-contrib.txt          ← 官方插件清单
├── templates/                    ← 官方插件空模板
│   └── tsnp/
│       ├── http/
│       │   ├── http_win.c
│       │   ├── http_linux.c
│       │   ├── http_macos.c
│       │   └── ts-native.toml
│       └── ...
└── <file>.ts.toml                ← 入口文件依赖声明
```

## 依赖声明生成

### 按文件分析

每个包含 main 函数的 TS 文件生成独立的 .ts.toml：

```rust
fn generate_ts_toml(ts_files: &[String], output: &str) -> Result<()> {
    for ts_file in ts_files {
        // 只处理入口文件
        if !has_main_function(ts_file)? {
            continue;
        }
        
        // 分析这个文件需要的插件
        let file_plugins = detect_plugins_from_ast(&[ts_file.clone()])?;
        
        // 生成 .ts.toml
        let toml_content = format!(
            "[dependencies]\ntsnp = [{}]\n",
            plugins.join(", ")
        );
        
        fs::write(format!("{}.ts.toml", ts_file), toml_content)?;
    }
    Ok(())
}
```

### 示例输出

**main.ts.toml**：
```toml
[dependencies]
tsnp = ["http", "crypto", "fs"]
```

**worker.ts.toml**：
```toml
[dependencies]
tsnp = ["os", "timer"]
```

## 进度通报机制

使用 `indicatif` 提供实时进度：

```rust
let pb = ProgressBar::new(total_steps);
pb.set_style(ProgressStyle::default_bar()
    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
    .expect("Failed to create progress bar style")
    .progress_chars("#>-"));

for plugin in plugins {
    generate_plugin(plugin)?;
    pb.inc(1);
    pb.set_message(format!("Generating {}", plugin));
}
```

## 错误处理

### AST 解析错误

```rust
let module = parser.parse_module()
    .map_err(|e| anyhow::anyhow!("Failed to parse {}: {:?}", ts_file, e))?;
```

### 文件 I/O 错误

```rust
fs::write(&path, content)
    .context(format!("Failed to write {}", path.display()))?;
```

### 溢出保护

在 C 代码中添加容量溢出检查：

```c
if (arr->capacity > 0x3FFFFFFF) {
    return bits_to_val(UNDEFINED);
}
uint32_t new_cap = arr->capacity * 2;
```

## 测试策略

### 单元测试

```rust
#[test]
fn test_detect_plugins_from_ast() {
    let plugins = detect_plugins_from_ast(&vec![
        "test.ts".to_string()
    ]).unwrap();
    
    assert!(plugins.contains("http"));
    assert!(plugins.contains("crypto"));
}
```

### 集成测试

```bash
# 测试 1：单入口文件
cargo tsn prepare --input main.ts

# 测试 2：多入口文件
cargo tsn prepare --input main.ts --input worker.ts

# 测试 3：预览模式
cargo tsn prepare --dry-run
```

## 性能优化

### API Map 缓存

使用 `once_cell::Lazy` 避免重复构建：

```rust
use once_cell::sync::Lazy;

static API_PLUGIN_MAP: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let mut map = HashMap::new();
    // ... 初始化
    map
});
```

### 增量更新

未来可以支持只更新变化的文件：

```rust
fn should_regenerate(ts_file: &str, plugin_dir: &str) -> bool {
    let ts_mtime = fs::metadata(ts_file).unwrap().modified().unwrap();
    let plugin_mtime = fs::metadata(plugin_dir).unwrap().modified().unwrap();
    ts_mtime > plugin_mtime
}
```

## 未来规划

### 阶段 2：crates.io 集成

- 自动搜索匹配的 crate
- 解析 crate 的 FFI 导出
- 生成映射配置

### 阶段 3：智能优化

- 智能类型推断
- 代码补全建议
- 错误检测与提示
- 自定义模板支持

## 相关文档

- [README.md](../README.md) - 命令使用指南
- [CHANGELOG.md](../CHANGELOG.md) - 版本历史
- [tsnp-contrib](../tsnp-contrib/) - 官方插件实现
