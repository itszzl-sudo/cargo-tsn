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
cargo tsn add serde_json

# 3. 写 Rust FFI（如果需要）
# src/lib.rs:
# #[no_mangle]
# pub extern "C" fn my_func() { ... }

# 4. 写 TypeScript
# main.ts:
# function main() { ... }

# 5. 编译
tsn main.ts

# 6. 运行
./a.exe
```

## 注意

**大部分 crate 没有 FFI 函数。**

`cargo tsn add tokio` 会：
- 下载 tokio
- 生成空插件（因为 tokio 没有 `#[no_mangle]`）

**解决：**

在 `src/lib.rs` 中包装：

```rust
use tokio::runtime::Runtime;

#[no_mangle]
pub extern "C" fn tokio_run() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        // async code
    });
}
```

然后：

```bash
tsnp gen my-project  # 生成你自己写的 FFI
```

## 与 tsnp 的关系

- `cargo tsn add` = `cargo add` + `tsnp gen`
- `cargo tsn new` = 创建项目结构

## 许可证

MIT
