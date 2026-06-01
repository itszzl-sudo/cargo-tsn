# cargo-tsn

Project manager for the tsn toolchain.

## Installation

```bash
cargo install cargo-tsn
```

## Overview

The tsn toolchain consists of three tools:

| Tool | Purpose | Command |
|------|---------|---------|
| **tsn** | TypeScript to native compiler | `tsn main.ts` |
| **tsnp** | Plugin configuration generator | `tsnp gen <crate>` |
| **cargo-tsn** | Project manager | `cargo tsn new <name>` |

## Commands

### cargo tsn new \<name\>

Create a new tsn project.

```bash
cargo tsn new my-project
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

### cargo tsn add \<crate\>

Add a Rust crate dependency and generate plugin configuration.

```bash
cargo tsn add regex
```

**What it does:**

1. Runs `cargo add <crate>` to download the dependency
2. Fetches published tsnps from Codeberg and displays them with version and publish time
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

**Success message:**

```
✅ Added crate: regex
📝 Next steps:
   1. Edit tsnp/regex/ts-native.toml to configure function mappings
   2. Implement FFI functions in src/lib.rs if needed
   3. Run 'cargo tsn publish' to publish the plugin
```

**Note:** Most crates don't have FFI functions, so the generated configuration will be empty. Users need to:

1. Write FFI wrapper functions in `src/lib.rs`
2. Edit `tsnp/<crate>/ts-native.toml` to configure function mappings

### cargo tsn func

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

Function name (or 'q'): multiply
Parameters (e.g., 'a: i32, b: i32'): a: i32, b: i32
Return type (e.g., 'i32'): i32

✅ Added to src/lib.rs
✅ Updated tsnp/regex/ts-native.toml

Function name (or 'q'): q

Select crate:
[1] regex      (tsnp/regex/)
[2] math       (tsnp/math/)
[q] Quit
Select: 2

math selected.

Function name (or 'q'): square
Parameters (e.g., 'n: i32'): n: i32
Return type (e.g., 'i32'): i32

✅ Added to src/lib.rs
✅ Updated tsnp/math/ts-native.toml

Function name (or 'q'): q

Select crate:
[1] regex      (tsnp/regex/)
[2] math       (tsnp/math/)
[q] Quit
Select: q

Done. 3 FFI function(s) added.
```

**Interaction logic:**

- Type function name, parameters, and return type
- Type `q` to return to crate selection
- Type `q` at crate selection to exit
- Can switch between crates to add functions to multiple plugins

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

### cargo tsn publish

Publish plugins to Codeberg repository.

```bash
cargo tsn publish
```

**Dry-run mode:**

```bash
cargo tsn publish --dry-run
```

This will show what would be published without actually uploading anything.

**Environment Variables:**

Before using `publish`, you must set up authentication:

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

[2/2] Publishing math...
   Version: 0.1.0, Author: tsnp
   Files: 3
   Release already exists, updating...
   Uploading math-0.1.0.zip...
   Published: https://codeberg.org/tsnp/tsnp/releases/tag/math-0.1.0
✅ Published math successfully.

📊 Summary: 2 succeeded, 0 failed
```

**Features:**

- ✅ Automatically checks if release already exists
- ✅ Updates existing releases instead of failing
- ✅ Progress indicators showing current plugin
- ✅ Summary of successes and failures
- ✅ Detailed error messages with API responses
- ✅ Automatic cleanup of temporary files

### cargo tsn list

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

## Workflow

### Basic workflow

```bash
# 1. Create project
cargo tsn new my-project
cd my-project

# 2. Add dependencies
cargo tsn add regex

# 3. Add FFI functions interactively
cargo tsn func

# 4. Implement FFI functions (edit src/lib.rs)

# 5. Compile
tsn main.ts

# 6. Run
./a.exe
```

### Complete example

```bash
# Create project
cargo tsn new calculator
cd calculator

# Add dependency
cargo tsn add math

# Add FFI functions
cargo tsn func

# Select: 1 (math selected)
# Function name: add
# Parameters: a: i32, b: i32
# Return type: i32
# Function name: subtract
# Parameters: a: i32, b: i32
# Return type: i32
# Function name: q
# Select: q

# Edit src/lib.rs to implement functions:
# #[no_mangle]
# pub extern "C" fn add(a: i32, b: i32) -> i32 {
#     a + b
# }

# Edit main.ts:
# function main() {
#     print(add(1, 2));
#     return 0;
# }

# Compile and run
tsn main.ts
./a.exe  # Output: 3
```

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

```
cd my-project
cargo tsn func  # ✅ Correct

cd my-project/src
cargo tsn func  # ❌ Error: tsnp/ not found
```

## Related Tools

- **tsn** - TypeScript native compiler: `cargo install tsn`
- **tsnp** - Plugin generator: `cargo install tsnp`

## License

MIT
