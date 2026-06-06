# cargo-tsn

**ts-native 项目管理工具** - 插件生成、依赖管理、项目脚手架

## 快速开始

```bash
# 安装
cargo install cargo-tsn

# 克隆项目（包含子模块）
git clone --recurse-submodules https://github.com/itszzl-sudo/cargo-tsn.git
cd cargo-tsn

# 如果已经克隆，初始化子模块
git submodule update --init --recursive

# 创建新项目
cargo tsn new my-project
cd my-project

# 编写 TypeScript 代码
cargo tsn prepare

# 从子模块拷贝官方插件
cp -r ../tsnp-contrib/http tsnp/

# 编译运行
ts-native main.ts
./main.exe
```

## 命令参考

### `cargo tsn new` - 创建项目

```bash
cargo tsn new <project-name>
```

**功能**：创建 ts-native 项目脚手架

**生成结构**：
```
my-project/
├── main.ts           # TypeScript 入口
├── Cargo.toml        # Rust 配置
└── tsnp/             # FFI 插件目录
```

**与 prepare 的关系**：
- `new` 创建项目基础结构（空项目）
- `prepare` 分析代码并生成插件模板和依赖声明
- **工作流**：`new` → 编写代码 → `prepare` → 拷贝插件 → 编译

**示例**：
```bash
# 1. 创建项目
cargo tsn new my-app
cd my-app

# 2. 编写 TypeScript 代码（使用插件 API）
# 编辑 main.ts，使用 http_get()、fs_writeFile() 等

# 3. 分析代码生成插件模板
cargo tsn prepare

# 4. 拷贝官方插件
cp -r ../tsnp-contrib/http tsnp/
cp -r ../tsnp-contrib/fs tsnp/

# 5. 编译
ts-native main.ts
```

---

### `cargo tsn prepare` - 生成插件骨架 ⭐

```bash
cargo tsn prepare [OPTIONS]
```

**功能**：从 TypeScript 源码分析插件需求，生成插件模板和依赖声明

**选项**：
| 选项 | 说明 | 默认值 |
|------|------|--------|
| `--input <FILE>` | 指定输入文件 | 自动扫描 *.ts |
| `--output <DIR>` | 输出目录 | `./prepare` |
| `--dry-run` | 预览模式（不生成文件） | - |

**示例**：
```bash
# 生成到默认目录
cargo tsn prepare

# 指定输入文件
cargo tsn prepare --input src/api.ts

# 指定输出目录
cargo tsn prepare --output my-plugins

# 预览模式
cargo tsn prepare --dry-run
```

**输出**：
```
📦 Analyzing TypeScript files...
  ✓ Found: main.ts, worker.ts
  ✓ Detected 4 plugin(s) via AST analysis

📦 Official Plugins:
  ✓ has-tsnp-contrib.txt (4 plugins)
  ✓ templates/tsnp/http/
  ✓ templates/tsnp/crypto/

  ✓ main.ts.toml
  ✓ worker.ts.toml
✅ Prepared 4 plugin(s)
   - 4 official (copy from tsnp-contrib/)
   - 0 custom (implement in tsnp/)
```

**详细文档**：[docs/PREPARE_DESIGN.md](docs/PREPARE_DESIGN.md)

---

### `cargo tsn list` - 列出插件

```bash
cargo tsn list
```

**功能**：列出本地可用的官方和自定义插件

**输出示例**：
```
📦 Developer Plugins (tsnp/):
  - crypto v0.1.0 (priority: 1000)

🏛️  Official Plugins (tsnp-contrib/):
  - http v0.1.0 (priority: 0)
  - fs v0.1.0 (priority: 0)
```

---

## 官方插件（子模块）

cargo-tsn 包含 [tsnp-contrib](https://github.com/itszzl-sudo/tsnp-contrib) 子模块，提供 12 个官方插件：

| 插件 | 功能 | 平台 |
|------|------|------|
| [http](tsnp-contrib/http/) | HTTP 请求 | Win/Linux/macOS |
| [fs](tsnp-contrib/fs/) | 文件系统 | Win/Linux/macOS |
| [crypto](tsnp-contrib/crypto/) | 加密哈希 | Win/Linux/macOS |
| [os](tsnp-contrib/os/) | 系统信息 | Win/Linux/macOS |
| [path](tsnp-contrib/path/) | 路径处理 | Win/Linux/macOS |
| [cli](tsnp-contrib/cli/) | 命令行参数 | Win/Linux/macOS |
| [timer](tsnp-contrib/timer/) | 定时器 | Win/Linux/macOS |
| [json](tsnp-contrib/json/) | JSON 解析 | Win/Linux/macOS |
| [net](tsnp-contrib/net/) | 网络套接字 | Win/Linux/macOS |
| [process](tsnp-contrib/process/) | 进程管理 | Win/Linux/macOS |
| [log](tsnp-contrib/log/) | 日志系统 | Win/Linux/macOS |
| [env](tsnp-contrib/env/) | 环境变量 | Win/Linux/macOS |

**使用官方插件**：
```bash
# 1. 准备项目
cargo tsn prepare

# 2. 查看官方插件列表
cat prepare/has-tsnp-contrib.txt

# 3. 从子模块拷贝
cp -r tsnp-contrib/http tsnp/
cp -r tsnp-contrib/fs tsnp/

# 4. 编译
ts-native main.ts
```

**子模块管理**：
```bash
# 克隆时初始化
git clone --recurse-submodules https://github.com/itszzl-sudo/cargo-tsn.git

# 手动初始化
git submodule update --init --recursive

# 更新到最新版本
git submodule update --remote tsnp-contrib

# 查看状态
git submodule status
```

---

## 工作流

### 标准工作流

```
1. cargo tsn new my-project         # 创建项目
   ↓
2. 编写 TypeScript 代码              # 使用插件 API（如 http_get()）
   ↓
3. cargo tsn prepare                # 分析代码，生成官方插件模板和依赖声明
   ↓
4. cp -r tsnp-contrib/http tsnp/    # 从子模块拷贝官方插件实现
   ↓
5. ts-native main.ts                # 编译（自动加载 tsnp/ 中的插件）
   ↓
6. ./main.exe                       # 运行
```

**说明**：
- `cargo tsn prepare` 会分析你的代码使用了哪些插件 API
- 官方插件（http, fs, crypto 等）直接从 `tsnp-contrib/` 子模块拷贝
- 不需要手动编写 `declare function`，AST 分析会自动检测函数调用
- `.ts.toml` 文件会自动生成，声明项目依赖哪些插件

### 插件优先级

| 插件来源 | 默认优先级 | 说明 |
|---------|-----------|------|
| **自定义插件** | **1000** | 通过 prepare 生成 |
| **官方插件** | **0** | tsnp-contrib 子模块 |

自定义插件始终优先于官方插件。

---

## 项目结构

```
cargo-tsn/
├── src/
│   ├── main.rs              # CLI 入口
│   └── prepare.rs           # prepare 命令实现
├── docs/
│   └── PREPARE_DESIGN.md    # prepare 命令详细设计
├── tsnp-contrib/            # 官方插件子模块
├── test-prepare/            # 测试用例
├── README.md                # 本文档
├── CHANGELOG.md             # 版本历史
└── Cargo.toml               # 项目配置
```

---

## 相关工具

| 工具 | 描述 | 安装 |
|------|------|------|
| **[ts-native](https://github.com/itszzl-sudo/ts-native/blob/main/README.md)** | TypeScript 到原生编译器 | `cargo install ts-native` |
| **[tsn](https://github.com/itszzl-sudo/ts-native/blob/main/README.md)** | 同上，短名称 | `cargo install tsn` |
| **[tsnp-contrib](https://github.com/itszzl-sudo/tsnp-contrib)** | 官方插件集合 | Git 子模块 |

---

## 许可证

MITsn prepare --input src/api.ts

# 指定输出目录
cargo tsn prepare --output my-plugins

# 预览模式
cargo tsn prepare --dry-run
```

## 工作流

### 方式一：使用 prepare 命令（推荐）

```
1. 编写 TypeScript 代码（包含 declare function）
   ↓
2. cargo tsn prepare（生成插件骨架到 prepare/）
   ↓
3. 在 prepare/tsnp/ 中编辑、选择需要的插件
   ↓
4. cp -r prepare/tsnp/crypto tsnp/     # 只拷贝需要的
   cp -r prepare/tsnp/http tsnp/      # 选择性拷贝
   ↓
5. 手动实现 C 函数
   ↓
6. 编译 C 文件为 .o
   ↓
7. ts-native main.ts（编译链接）
   ↓
8. 运行测试
```

### 方式二：手动创建插件

```
1. cargo tsn new my-project
   ↓
2. 手动在 tsnp/ 目录创建插件
   ↓
3. 编写 C FFI 函数
   ↓
4. ts-native main.ts
   ↓
5. 运行测试
```

## 与 tsnp 的区别

> **注意**：`tsnp` 工具已被 `cargo tsn prepare` 替代。

| 工具 | 职责 | 输入 | 输出 | 状态 |
|------|------|------|------|------|
| **cargo tsn new** | 创建空项目 | 项目名 | 项目脚手架 | ✅ 推荐使用 |
| **cargo tsn prepare** | 从 TS 源码生成插件 | TS 文件 | 插件模板 + 配置 | ✅ 推荐使用 |
| **tsnp new** | 创建空插件模板 | 插件名 | 空模板文件 | ⚠️ 已废弃 |
| **tsnp gen** | 从 crate 生成配置 | crate 名 | 配置文件 | ⚠️ 已废弃 |

## 插件优先级系统

### 优先级规则

| 插件来源 | 默认优先级 | 说明 |
|---------|-----------|------|
| **开发者自定义** | **1000** | 通过 prepare 命令生成或手动创建 |
| **官方插件** | **0** | ts-native 仓库自带的 tsnp-contrib |

### 设计原理

- ✅ 自定义插件始终优先于官方插件
- ✅ 避免能力冲突时选择错误的实现
- ✅ 清晰的优先级层次

### 示例

```toml
# tsnp/crypto/ts-native.toml
[package]
name = "tsnp-crypto"
version = "0.1.0"
priority = 1000  # 自定义插件默认 1000
```

## 相关工具

| 工具 | 描述 | 安装 |
|------|------|------|
| **[ts-native](https://github.com/itszzl-sudo/ts-native/blob/main/README.md)** | TypeScript 到原生可执行文件编译器 | `cargo install ts-native` |
| **[tsn](https://github.com/itszzl-sudo/ts-native/blob/main/README.md)** | 同上，更短的名称 | `cargo install tsn` |
| **[tsnp](https://github.com/itszzl-sudo/ts-native/tree/main/tsnp-contrib)** | 从 Rust crate 生成插件配置 | `cargo install tsnp` |

## 许可证

MIT
