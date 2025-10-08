# thag_demo Usage Improvements

This document describes the improvements made to `thag_demo` to address usability issues with the `script` and `browse` commands.

## Problems Fixed

### 1. Current Working Directory Issues
**Problem**: When running `cargo run -- browse` from `./thag_demo`, selecting a script required using paths like `../demo/hello.rs` because the code changed the working directory to the demo parent directory.

**Solution**: The current working directory is now left unchanged. Users can specify paths relative to their actual working directory, making behavior predictable and consistent with standard command-line tools.

### 2. Missing Options Support
**Problem**: There was no way to pass thag options like `-f` (force rebuild), `-t` (timings), `--features`, etc., to demo scripts.

**Solution**: The `script` command now accepts all common thag options:
- `-f, --force` - Force rebuild even if script unchanged
- `-t, --timings` - Display execution timings
- `--features <FEATURES>` - Enable specific features (comma-separated)
- `-g, --generate` - Just generate, don't run
- `-b, --build` - Just build, don't run
- `-c, --check` - Just check, don't run

### 3. Inconsistent Argument Handling
**Problem**: The `script` command didn't properly separate thag options from script arguments, and the `browse` command had different argument collection behavior.

**Solution**: 
- Both `script` and `browse` now handle arguments consistently
- **Command-line mode**: Use `--` to separate thag_demo/thag options from script arguments (optional but recommended when arguments start with `-`)
- **Interactive mode**: DON'T type `--` when entering arguments - it will be automatically handled
- Interactive browser prompts for both options and arguments when needed

### 4. Unreliable Root Directory Detection
**Problem**: `THAG_DEV_PATH` was set incorrectly depending on where the command was run from, causing inconsistent behavior between running from `thag_rs` vs `thag_demo` directories.

**Solution**: Added `find_thag_rs_root()` function that reliably locates the thag_rs workspace root by:
- Walking up the directory tree looking for the workspace Cargo.toml
- Checking for workspace members or package name matching "thag_rs"
- Falling back to manifest directory's parent if needed

### 5. Confusing `--` Separator in Interactive Mode
**Problem**: Sample arguments in demo scripts showed `-- demo/hello.rs` format, which users would copy into interactive prompts. This caused failures because `--` is a command-line separator that shouldn't be typed in interactive input.

**Solution**:
- Interactive mode now strips `--` from sample argument displays
- If user accidentally types `--`, it's automatically removed from input
- Clear messages now explain: "(don't type '--' in interactive mode)"
- Command-line mode still uses `--` as optional separator (standard CLI convention)

## Path Resolution - IMPORTANT

**All file paths are relative to your shell's current working directory (PWD)**, not to any internal thag_rs or demo directory. This is standard command-line behavior.

Examples:
```bash
# If your PWD is /Users/you/projects/thag_rs
cd /Users/you/projects/thag_rs
cargo run -p thag_demo -- script syn_dump_syntax -- demo/hello.rs
# Looks for: /Users/you/projects/thag_rs/demo/hello.rs

# If your PWD is /Users/you/projects/thag_rs/thag_demo
cd /Users/you/projects/thag_rs/thag_demo
cargo run -- script syn_dump_syntax -- ../demo/hello.rs
# Looks for: /Users/you/projects/thag_rs/demo/hello.rs (same file!)
```

The demo scripts themselves are located by name (without path) from the demo directory, but **arguments you pass** are resolved from your shell's PWD.

## Usage Examples

### Running with Options (script command)

**Command-line usage** (you type these commands):

```bash
# Force rebuild with timings (with -- separator, recommended)
cargo run -p thag_demo -- script syn_dump_syntax -f -t -- demo/hello.rs

# Same thing without -- (works since argument doesn't start with -)
cargo run -p thag_demo -- script syn_dump_syntax -f -t demo/hello.rs

# Just check without running
cargo run -p thag_demo -- script my_script -c

# Enable specific features
cargo run -p thag_demo -- script profiling_demo --features full_profiling -- input.txt

# When script argument starts with -, use -- to separate
cargo run -p thag_demo -- script my_script -- --script-flag value.txt

# Multiple options with script arguments
cargo run -p thag_demo -- script syn_dump_syntax -f -t -- ../demo/hello_main.rs
```

### Running from Different Directories

```bash
# From thag_rs root - paths relative to root
cd thag_rs
cargo run -p thag_demo -- script syn_dump_syntax -f -- demo/hello.rs

# From thag_demo directory - paths still relative to current location
cd thag_rs/thag_demo
cargo run -- script syn_dump_syntax -f -- ../demo/hello.rs

# Both work correctly and find thag_rs root automatically
```

### Interactive Browser

The browse command now prompts for both options and arguments:

```bash
cargo run -p thag_demo -- browse
```

When you select a demo that accepts arguments:
1. You'll be prompted for thag options (force rebuild, timings, etc.)
2. You'll be prompted for script arguments
   - **Important**: Paths are relative to your current shell directory
   - The prompt will show your current directory for reference
   - **DON'T type `--` in interactive mode** - just enter the arguments directly
   - Example: Type `demo/hello.rs` NOT `-- demo/hello.rs`
3. The demo runs with your specified options and arguments

**Why no `--` in interactive mode?**
The `--` separator is a command-line shell convention to separate options from arguments. In interactive prompts, you're only entering arguments, so the separator isn't needed. The system handles this automatically.

### Visual Comparison: Command-Line vs Interactive

**✅ Command-Line Mode (script command):**
```bash
cargo run -p thag_demo -- script syn_dump_syntax -f -t -- demo/hello.rs
#                                                      ^^ separator needed here
```

**✅ Interactive Mode (browse command):**
```
Prompt: Enter arguments (paths relative to /Users/you/projects/thag_rs): demo/hello.rs
                                                                          ^^^^^^^^^^^^^^
                                                                          NO -- prefix!
```

**❌ Common Mistake:**
```
Prompt: Enter arguments: -- demo/hello.rs
                         ^^ DON'T TYPE THIS in interactive mode!
```

The system will auto-strip the `--` if you accidentally type it, but it's clearer not to include it.

## Consistency with thag

The syntax now mirrors standard thag usage:

```bash
# Standard thag (from thag_rs directory)
thag demo/syn_dump_syntax.rs -f -- demo/hello_main.rs

# thag_demo equivalent (from thag_rs directory)
cargo run -p thag_demo -- script syn_dump_syntax -f -- demo/hello_main.rs

# Or without -- (works for most cases)
cargo run -p thag_demo -- script syn_dump_syntax -f demo/hello_main.rs
```

Key differences:
- **Script location**: `demo/syn_dump_syntax.rs` → `syn_dump_syntax` (name only, no path/extension)
- **Separator**: `--` is optional in `script` command unless arguments start with `-`
- **Path resolution**: Both resolve file arguments relative to your shell's PWD

## Technical Details

### DemoOptions Structure
```rust
struct DemoOptions {
    force: bool,
    timings: bool,
    features: Option<String>,
    generate: bool,
    build: bool,
    check: bool,
}
```

These options are:
- Collected via CLI arguments in `script` command
- Collected interactively in `browse` command
- Passed to `create_demo_cli_with_args()` to configure the thag Cli

### Root Directory Detection
The `find_thag_rs_root()` function:
1. Starts from current directory or manifest directory
2. Walks up to 10 levels searching for Cargo.toml
3. Checks if Cargo.toml contains workspace with "thag_rs" or package named "thag_rs"
4. Returns the directory containing the matching Cargo.toml
5. Falls back to manifest parent directory if search fails

This ensures `THAG_DEV_PATH` is set correctly regardless of where the command is run from.

## Migration Guide

### Old Way (Broken)
```bash
# Had to use confusing relative paths
cd thag_demo
cargo run -- browse
# Select syn_dump_syntax
# Type: -- ../demo/hello.rs  (confusing! Where is this relative to?)
# No way to add -f flag
```

### New Way (Fixed)
```bash
# From thag_rs root - paths relative to thag_rs
cd thag_rs
cargo run -p thag_demo -- script syn_dump_syntax -f demo/hello.rs

# From thag_demo - paths relative to thag_demo  
cd thag_rs/thag_demo
cargo run -- script syn_dump_syntax -f ../demo/hello.rs

# Or use browse with clear prompts showing your current directory
cargo run -p thag_demo -- browse
# Shows: 📂 Current shell directory: /Users/you/projects/thag_rs
# Shows: 💡 Sample arguments: demo/hello_main.rs
#        (don't type '--' in interactive mode)
# Select syn_dump_syntax
# Prompt: Force rebuild? [y/N]: y
# Prompt: Enter arguments (paths relative to /Users/you/projects/thag_rs): demo/hello.rs
#         ^^^^^^^^^^^^^^^ Type ONLY the arguments, no -- prefix!
```

## Benefits

1. **Predictable behavior**: Working directory stays where you expect it (your shell's PWD)
2. **Clear path resolution**: Prompts and messages show exactly where paths are resolved from
3. **Full control**: Access to all thag options for debugging and optimization
4. **Consistency**: Same argument syntax as thag itself (with optional `--` in CLI mode)
5. **Flexibility**: Works correctly from any directory in the project
6. **Discoverability**: Help text shows all available options
7. **User-friendly**: Interactive mode auto-strips `--` if accidentally typed, and prompts explain the correct format

## See Also

- Main thag documentation for complete option descriptions
- `demo/` directory for available demo scripts
- `cargo run -p thag_demo -- script --help` for quick reference