# ts-native

TypeScript to native executable compiler.

## Quick Start

```bash
# Install
cargo install ts-native

# Compile and run (output: main.exe in same directory as main.ts)
ts-native main.ts
./main.exe
```

## What is ts-native

A compiler that converts TypeScript to native executable (8-23KB), no Node.js required.

Also published as **`tsn`** (`cargo install tsn`) — same compiler, shorter name.

## Key Features

- **Tiny executables**: 8-23KB native binaries
- **No runtime**: Zero dependencies, no Node.js
- **FFI integration**: Call C/Rust functions directly
- **NaN-boxing**: Efficient value representation
- **Syntax pre-check**: swc-based AST analysis detects unsupported syntax before compilation
- **Standalone**: No project management tools required

## Architecture

```
TypeScript source
      ↓
  Syntax Check (swc)
      ↓
    Lexer
      ↓
    Parser
      ↓
     HIR (High-level IR)
      ↓
    Codegen (Cranelift)
      ↓
    Object file (.o)
      ↓
    Linker (.exe)
      ↓
Native executable
```

## Value Representation

All JavaScript values fit in 64 bits using NaN-boxing:

```
STRING_TAG  = 0x7FFC_0000_0000_0000
ARRAY_TAG   = 0x7FFB_0000_0000_0000
OBJECT_TAG  = 0x7FFA_0000_0000_0000
UNDEFINED   = 0x7FFF_8000_0000_0001
NULL        = 0x7FFF_8000_0000_0002
TRUE        = 0x7FFF_0000_0000_0001
FALSE       = 0x7FFF_0000_0000_0000
```

## C Runtime

ts-native includes a monolithic C runtime (`runtime_nocrt.c`) with:
- String operations (js_string_new, js_string_concat, js_number_to_string)
- Array operations (js_array_new, js_array_push, js_array_get, js_array_set)
- Object operations (js_object_new, js_object_get, js_object_set)
- Math operators (js_add, js_sub, js_mul, js_div, js_mod)
- Comparison operators (js_eq, js_ne, js_lt, js_le, js_gt, js_ge)
- Print operations (js_print, js_print_space, js_print_newline)
- Math functions (js_math_floor, js_math_ceil, js_math_random, js_math_sin, ...)
- Date/JSON (js_date_now, js_json_stringify, js_json_parse)
- Exception handling (js_try_begin, js_try_end, js_throw, js_get_exception)
- FFI stubs (argc, argv, now_ms, sleep, http_get, file_append, ...)

## Supported TypeScript

| 语法 | 状态 | 示例 | 分类 |
|------|------|------|------|
| function declaration | ✅ | `function foo() {}` | 函数 |
| function expression | ✅ | `let f = function() {}` | 函数 |
| arrow function | ✅ | `(x) => x + 1` | 函数 |
| return | ✅ | `return x;` | 控制流 |
| if/else | ✅ | `if (x) {} else {}` | 控制流 |
| while | ✅ | `while (x) {}` | 控制流 |
| for | ✅ | `for (let i=0; i<n; i++) {}` | 控制流 |
| for...in | ✅ | `for (let k in obj) {}` | 控制流 |
| for...of | ✅ | `for (let x of arr) {}` | 控制流 |
| break/continue | ✅ | `break; continue;` | 控制流 |
| try/catch/finally | ✅ | `try {} catch(e) {} finally {}` | 控制流 |
| throw | ✅ | `throw new Error()` | 控制流 |
| ternary (?:) | ✅ | `x ? a : b` | 表达式 |
| let/const/var | ✅ | `let x = 1; const y = 2;` | 声明 |
| assignment | ✅ | `x = 1; x += 2;` | 表达式 |
| binary operators | ✅ | `+, -, *, /, %, ==, <, >, &&, ||` | 表达式 |
| unary operators | ✅ | `-x, !x, typeof x` | 表达式 |
| increment/decrement | ✅ | `x++; x--;` | 表达式 |
| string literal | ✅ | `"hello", 'world'` | 字面量 |
| template literal | ✅ | `` `hello ${name}` `` | 字面量 |
| number literal | ✅ | `42, 3.14, 0xFF` | 字面量 |
| boolean literal | ✅ | `true, false` | 字面量 |
| null/undefined | ✅ | `null, undefined` | 字面量 |
| array literal | ✅ | `[1, 2, 3]` | 字面量 |
| object literal | ✅ | `{ key: value }` | 字面量 |
| property access | ✅ | `obj.key, obj[0]` | 表达式 |
| function call | ✅ | `foo(1, 2)` | 表达式 |
| method call | ✅ | `obj.method()` | 表达式 |
| import declaration | ✅ | `import { x } from 'mod'` | 模块 |
| export declaration | ✅ | `export function foo() {}` | 模块 |
| declare function | ✅ | `declare function foo(): void;` | TS扩展 |
| interface (declare only) | ✅ | `interface Config { x: number }` | TS扩展 |
| type annotation | ✅ | `let x: number = 1;` | TS扩展 |
| as type assertion | ✅ | `x as number` | TS扩展 |
| typeof operator | ✅ | `typeof x` | 表达式 |
| console.log/print | ✅ | `console.log(x), print(x)` | 内置 |
| Math methods | ✅ | `Math.sin, Math.random, ...` | 内置 |
| JSON.parse/stringify | ✅ | `JSON.parse(s), JSON.stringify(o)` | 内置 |
| Date.now | ✅ | `Date.now()` | 内置 |
| parseInt/parseFloat | ✅ | `parseInt(s)` | 内置 |
| class | ❌ | `class Foo {}` | 面向对象 |
| class inheritance | ❌ | `class Bar extends Foo {}` | 面向对象 |
| decorator | ❌ | `@decorator class Foo {}` | 面向对象 |
| enum | ❌ | `enum E { A, B }` | 声明 |
| async/await | ❌ | `async function f() { await p }` | 异步 |
| generator | ❌ | `function* g() { yield 1 }` | 函数 |
| Promise | ❌ | `new Promise((r) => r(1))` | 异步 |
| namespace | ❌ | `namespace N { }` | TS扩展 |
| abstract class | ❌ | `abstract class A {}` | 面向对象 |
| with statement | ❌ | `with (obj) { }` | 遗留 |
| labeled statement | ❌ | `label: for (;;) { break label; }` | 控制流 |
| switch | ❌ | `switch(x) { case 1: break; }` | 控制流 |
| do...while | ❌ | `do {} while (x)` | 控制流 |
| new expression | ❌ | `new Foo()` | 表达式 |
| delete/void/in | ❌ | `delete obj.key; void 0; x in obj` | 表达式 |
| spread/rest | ❌ | `...args, fn(...a)` | 表达式 |
| destructuring | ❌ | `let { a, b } = obj;` | 声明 |
| default param | ❌ | `function f(x = 1) {}` | 函数 |
| computed property | ❌ | `{ [key]: value }` | 表达式 |
| optional chaining | ❌ | `obj?.key?.method()` | 表达式 |
| nullish coalescing | ❌ | `x ?? y` | 表达式 |
| regex literal | ❌ | `/pattern/flags` | 字面量 |
| satisfies operator | ❌ | `x as const satisfies T` | TS扩展 |
| type alias (body) | ❌ | `type T = { a: number }` | TS扩展 |

## CLI Options

```
ts-native <input.ts> [-o <output>] [--skip-check] [--gen-syntax-md]

  -o, --output <file>    Output file (default: <input>.exe in input directory)
  --skip-check           Skip syntax pre-check (may cause compile errors)
  --gen-syntax-md        Print syntax support list as Markdown table
```

## Syntax Pre-check

ts-native uses **swc** to parse TypeScript into an AST and detect unsupported syntax **before** compilation. This provides clear error messages instead of cryptic compile failures:

```bash
$ ts-native test.ts
❌ 检测到不支持的语法:
   第 1 行: class (ts-native 暂不支持)

提示: 使用 --skip-check 跳过检查（可能导致编译错误）
```

## Project Structure

### Minimal Project (No Config Files)

```
my-project/
└── main.ts          # Only file needed!
```

**Compile**:
```bash
ts-native main.ts    # → main.exe (same directory)
```

### FFI Project (With Extensions)

```
my-project/
├── main.ts          # TypeScript source
├── main.ts.toml     # Per-file dependency declaration (optional)
└── tsnp/            # FFI extensions (optional)
    └── my-ext/
        ├── ts-native.toml    # Extension config
        └── index.d.ts        # Type definitions
```

### Per-file Dependencies (`<input>.ts.toml`)

Each TypeScript file can have a companion `<filename>.ts.toml` that declares which tsnp extensions it depends on:

**`main.ts.toml`**:
```toml
[dependencies]
tsnp = ["http", "fs", "cli"]
```

- If `<input>.ts.toml` exists → only the listed extensions are loaded
- If `<input>.ts.toml` does **not** exist → all tsnp extensions are loaded (backward-compatible)
- `tsnp` tool auto-generates this file when creating plugins

---

## FFI Extensions

### When Do You Need FFI?

**Need FFI**:
- ✅ Call C/Rust functions
- ✅ Use system libraries (curl, ssl, etc.)
- ✅ Access OS APIs (socket, file, etc.)

**Don't Need FFI**:
- ❌ Pure TypeScript logic
- ❌ Only use built-in runtime (js_print, js_add, etc.)
- ❌ No external dependencies

### ts-native.toml Example

```toml
[package]
name = "tsnp-http"
version = "0.1.0"

[functions]
"http_get" = { args = ["string"], ret = "string", impl_name = "http_get" }
"http_post" = { args = ["string", "string"], ret = "string", impl_name = "http_post" }

[link]
libs = ["curl", "ssl", "crypto"]
```

### Using FFI in TypeScript

```typescript
// Declare FFI function
declare function http_get(url: string): string;

// Use it
let response = http_get("https://example.com");
print(response);
```

### Auto-generating `<input>.ts.toml`

When you use `tsnp` to generate a plugin, it automatically creates or updates the `.ts.toml` dependency file for all `.ts` files in the current directory:

```bash
$ cargo tsnp gen my-crate
  ✓ ts-native.toml
  ✓ index.d.ts
  ✓ main.ts.toml      # auto-generated
```

### Official Contrib Plugins (`tsnp-contrib/`)

ts-native ships with official plugins in `tsnp-contrib/`:

| Plugin | Description |
|--------|-------------|
| `cli` | Command-line arguments (argc, argv) |
| `fs` | File system operations (file_append, file_exists) |
| `metric-api` | Metric API client (http_get) |
| `dom-iot` | DOM runtime for IoT |
| `mqtt` | MQTT protocol support |

Copy plugins from `tsnp-contrib/` to your project's `tsnp/` directory to use them.

---

## Examples

See `examples/` directory for complete projects:

- **metric-collector**: Monitoring CLI with FFI (http, fs, cli)

---

## Source Structure

```
ts-native/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── ts_parser.rs     # TypeScript lexer & parser
│   ├── codegen.rs       # Cranelift code generation
│   ├── syntax_check.rs  # swc-based syntax pre-check
│   ├── linker.rs        # Native linker
│   ├── extension.rs     # Plugin loading
│   ├── config.rs        # Configuration
│   ├── runtime.rs       # Runtime helpers
│   └── pe_builder.rs    # PE executable builder
├── tsnp-contrib/        # Official contrib plugins
├── runtime_nocrt.c      # Monolithic C runtime
├── start_nocrt.c        # nocrt entry point (_start)
└── Cargo.toml
```

## Related Tools

| Tool | Description | Install |
|------|-------------|---------|
| **tsn** | Same as ts-native, shorter name | `cargo install tsn` |
| **tsnp** | Auto-generate FFI plugin configs from Rust crates | `cargo install tsnp` |
| **cargo-tsn** | Project manager for ts-native | `cargo install cargo-tsn` |

## License

MIT
