# cargo-tsn

tsn 项目管理工具。

## 安装

```bash
cargo install cargo-tsn
```

## 命令

### cargo tsn new <name>

创建 tsn 项目。

```bash
cargo tsn new my-project

# 生成：
# my-project/
# ├── Cargo.toml
# ├── src/
# │   └── lib.rs
# ├── main.ts
# └── tsnp/
```

### cargo tsn add <crate>

添加 Rust crate 依赖并生成插件。

```bash
cargo tsn add regex

# 做了什么：
# 1. cargo add regex          # 下载依赖
# 2. tsnp gen regex           # 生成插件
# 3. 放到 tsnp/regex/
```

## 工作流

```bash
# 1. 创建项目
cargo tsn new my-project
cd my-project

# 2. 添加依赖
cargo tsn add regex

# 3. 编辑 src/lib.rs 写 FFI（如果需要）

# 4. 编辑 tsnp/ 下的配置（如果需要）

# 5. 编译
tsn main.ts
```

## 许可证

MIT
