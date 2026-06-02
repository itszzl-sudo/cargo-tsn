# cargo-tsn

**Project manager for the tsn toolchain.**

[![Crates.io](https://img.shields.io/crates/v/cargo-tsn.svg)](https://crates.io/crates/cargo-tsn)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/itszzl-sudo/cargo-tsn)

---

## Installation

```bash
cargo install cargo-tsn
```

---

## Overview

The **tsn toolchain** consists of three tools:

| Tool | Purpose | Repository |
|------|---------|------------|
| **tsn** | TypeScript to native compiler | [tsn](https://github.com/itszzl-sudo/tsn) |
| **tsnp** | Plugin configuration generator | [tsiot/tsnp](https://github.com/itszzl-sudo/tsiot) |
| **cargo-tsn** | Project manager | [cargo-tsn](https://github.com/itszzl-sudo/cargo-tsn) |

**Workflow:**
```
cargo-tsn (project manager)
    ↓ creates/manages
tsn project (TypeScript + Rust FFI)
    ↓ compiles with
tsn (compiler)
    ↓ uses
tsnp (plugins)
```

---

## Commands

### `cargo tsn new <name>`

Create a new tsn project.

```bash
cargo tsn new my-project
cd my-project
```

**Generated structure:**
```
my-project/
├── Cargo.toml       # Rust project configuration
├── src/
│   └── lib.rs       # FFI functions (user writes here)
├── main.ts          # TypeScript entry point
└── tsnp/            # Plugin configurations
```

**Cargo.toml:**
```toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
```

**src/lib.rs:**
```rust
// Export FFI functions here
// Example:
// #[no_mangle]
// pub extern "C" fn my_func() -> i32 { 0 }
```

---

### `cargo tsn add <crate>`

Add a Rust crate dependency and generate plugin configuration.

```bash
cargo tsn add regex
```

**What it does:**
1. Runs `cargo add <crate>` to download the dependency
2. Fetches published tsnps from Codeberg and displays them
3. Prompts you to select an existing tsnp or create a new one
4. Runs `tsnp gen <crate>` to generate plugin configuration
5. Places output in `tsnp/<crate>/`

**Interactive selection:**
```
📦 Published tsnps available:
  [1] regex v0.2.1 (published: 2026-05-27)
  [2] math v0.1.0 (published: 2026-05-26)
  [n] Create new tsnp for regex
[q] Cancel

Select an existing tsnp or create new: 
```

**Generated files:**
```
tsnp/<crate>/
├── ts-native.toml    # Function mapping configuration
├── index.d.ts        # TypeScript type definitions
└── README.md         # Usage documentation
```

---

### `cargo tsn func`

Interactively add FFI functions.

```bash
cargo tsn func
```

**Interactive flow:**
```
Current directory: .

Select crate:
[1] regex      (tsnp/regex/)
[2] math       (tsnp/math/)
[q] Quit
Select: 1

regex selected.

Function name (or 'q'): add
Parameters (e.g., 'a: i32, b: i32'): a: i32, b: i32
Return type (e.g., 'i32'): i32

✅ Added to src/lib.rs
✅ Updated tsnp/regex/ts-native.toml

Function name (or 'q'): q

Done. 1 FFI function(s) added.
```

**What gets generated:**

1. **src/lib.rs** - FFI function stub:
```rust
#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    // TODO: implement
    0 as i32
}
```

2. **tsnp/\<crate\>/ts-native.toml** - Function mapping:
```toml
[functions]
"add" = { args = ["number", "number"], ret = "number", impl_name = "add" }
```

---

### `cargo tsn publish`

Publish plugins to Codeberg repository.

```bash
cargo tsn publish
```

**Dry-run mode:**
```bash
cargo tsn publish --dry-run
```

**Environment Variables:**
```bash
# Required: Your Codeberg API token
export CODEBERG_TOKEN="your-api-token-here"

# Optional: Customize target repository
export CODEBERG_USER="tsnp"          # Default: tsnp
export CODEBERG_REPO="tsnp"          # Default: tsnp
export CODEBERG_API="https://codeberg.org/api/v1"  # Default: Codeberg API
export CODEBERG_AUTHOR="your-name"   # Default: tsnp
```

**How to get a Codeberg API token:**
1. Go to https://codeberg.org/user/settings/applications
2. Generate a new token with `repo` permissions
3. Set it as environment variable

**Interactive flow:**
```
Publishing plugins to codeberg.org
Target: https://codeberg.org/tsnp/tsnp

Available plugins:
[1] regex
[2] math
[a] Publish all
[q] Cancel

Select: a

[1/2] Publishing regex...
   Version: 0.1.0, Author: tsnp
   Files: 3
   Creating release...
   Uploading regex-0.1.0.zip...
   Published: https://codeberg.org/tsnp/tsnp/releases/tag/regex-0.1.0
✅ Published regex successfully.

📊 Summary: 1 succeeded, 0 failed
```

---

### `cargo tsn list`

List local plugins in the current project.

```bash
cargo tsn list
```

**Output:**
```
Listing local plugins:
  - regex v0.1.0
  - math v0.1.0
```

---

### `cargo tsn install <name>`

Install a published plugin from Codeberg.

```bash
cargo tsn install regex
```

**Install specific version:**
```bash
cargo tsn install regex --version 0.2.1
```

**What it does:**
1. Fetches plugin from Codeberg releases
2. Extracts to `tsnp/<name>/`
3. Updates `Cargo.toml` with dependency

---

## Complete Workflow

### Basic Example

```bash
# 1. Create project
cargo tsn new my-project
cd my-project

# 2. Add dependencies
cargo tsn add regex

# 3. Add FFI functions interactively
cargo tsn func

# 4. Implement FFI functions (edit src/lib.rs)
# Example:
# #[no_mangle]
# pub extern "C" fn add(a: i32, b: i32) -> i32 {
#     a + b
# }

# 5. Edit main.ts
# function main() {
#     print(add(1, 2));
#     return 0;
# }

# 6. Compile
tsn main.ts

# 7. Run
./a.exe  # Output: 3
```

---

### Publishing Example

```bash
# 1. Create and develop plugin
cargo tsn new my-plugin
cd my-plugin
cargo tsn add some-crate
cargo tsn func

# 2. Set up Codeberg token
export CODEBERG_TOKEN="your-token"

# 3. Publish
cargo tsn publish

# 4. Others can now install
cargo tsn install my-plugin
```

---

## Type Mapping

When using `cargo tsn func`, Rust types are automatically mapped to TypeScript types:

| Rust Type | TypeScript Type |
|-----------|-----------------|
| i8, u8, i16, u16, i32, u32, i64, u64, isize, usize | number |
| f32, f64 | number |
| *const c_char, *mut c_char | string |
| &str | string |
| *const T, *mut T | number (pointer) |
| () | void |

---

## Project Structure

A complete tsn project:

```
my-project/
├── Cargo.toml           # Rust configuration
├── Cargo.lock           # Dependency lock file
├── src/
│   └── lib.rs           # FFI functions
├── main.ts              # TypeScript code
├── tsnp/                # Plugin configurations
│   ├── regex/
│   │   ├── ts-native.toml
│   │   ├── index.d.ts
│   │   └── README.md
│   └── math/
│       ├── ts-native.toml
│       ├── index.d.ts
│       └── README.md
├── a.o                  # Compiled object file
└── a.exe                # Native executable
```

---

## ts-native.toml Format

```toml
[package]
name = "tsnp-regex"
version = "0.1.0"
tsnpVersion = "0.1.0"

[functions]
"add" = { args = ["number", "number"], ret = "number", impl_name = "add" }
"multiply" = { args = ["number", "number"], ret = "number", impl_name = "multiply" }

[link]
lib = "regex"
```

**Version Fields:**
- `version` - The original Rust crate version
- `tsnpVersion` - The tsnp plugin version (independent versioning)

---

## Notes

### Crates without FFI functions

Most Rust crates don't export FFI functions (`#[no_mangle] extern "C"`). When running `cargo tsn add <crate>`:

- Dependency is downloaded
- Empty plugin configuration is generated
- User must write FFI wrappers in `src/lib.rs`

### Writing FFI wrappers

To use a crate without FFI functions, wrap its functionality:

```rust
use some_crate::SomeType;

#[no_mangle]
pub extern "C" fn my_wrapper(arg: i32) -> i32 {
    // Call crate functionality
    let result = SomeType::new(arg);
    result.compute()
}
```

Then run:
```bash
cargo tsn func
# Add the wrapper function to ts-native.toml
```

### Running from correct directory

`cargo tsn func` must be run from the project root (where `tsnp/` directory exists).

```bash
cd my-project
cargo tsn func  # ✅ Correct

cd my-project/src
cargo tsn func  # ❌ Error: tsnp/ not found
```

---

## Related Tools

- **tsn** - TypeScript native compiler: [GitHub](https://github.com/itszzl-sudo/tsn)
- **tsnp** - Plugin generator (in tsn repository)

---

## Repository

- **GitHub**: https://github.com/itszzl-sudo/cargo-tsn
- **Issues**: https://github.com/itszzl-sudo/cargo-tsn/issues

---

## License

MIT
