# Environment Variables in thag_demo

This guide shows how to use environment variables with `thag_demo` to control demo script behavior, especially for profiling demos.

## Common Environment Variables

### THAG_PROFILER
Controls profiling output generation:
- `THAG_PROFILER=both` - Generate both time and memory profiling data (.folded files)
- `THAG_PROFILER=time` - Generate only time profiling data
- `THAG_PROFILER=memory` - Generate only memory profiling data
- Not set - Run demo without generating profiling output files

### RUST_LOG
Controls Rust logging verbosity:
- `RUST_LOG=debug` - Debug level logging
- `RUST_LOG=info` - Info level logging
- `RUST_LOG=error` - Error level only

### THAG_DEV_PATH
Automatically set by thag_demo to point to the thag_rs root directory for local development. You normally don't need to set this manually.

## Command-Line Usage (script command)

### Single Environment Variable

```bash
# Set THAG_PROFILER for a profiling demo
cargo run -p thag_demo -- script basic_profiling -E THAG_PROFILER=both

# Set RUST_LOG for debugging
cargo run -p thag_demo -- script my_script -E RUST_LOG=debug demo/input.rs
```

### Multiple Environment Variables

Use the `-E` flag multiple times:

```bash
# Set both THAG_PROFILER and RUST_LOG
cargo run -p thag_demo -- script profiling_demo \
  -E THAG_PROFILER=both \
  -E RUST_LOG=info \
  -- demo/input.txt
```

### Combined with Other Options

```bash
# Force rebuild, show timings, set env var, pass arguments
cargo run -p thag_demo -- script basic_profiling \
  -f \
  -t \
  -E THAG_PROFILER=both \
  -- demo/data.txt

# With features and env vars
cargo run -p thag_demo -- script advanced_profiling \
  --features full_profiling \
  -E THAG_PROFILER=memory \
  -E RUST_LOG=debug
```

## Interactive Usage (browse command)

When using `browse`, you'll be prompted for environment variables after selecting a demo and setting options.

### Example Session

```
$ cargo run -p thag_demo -- browse

🚀 Interactive Demo Browser
═══════════════════════════════════════════════════════════
📚 45 demo scripts available
💡 Start typing to filter demos by name • 📝 = accepts arguments
📂 Current shell directory: /Users/you/projects/thag_rs
═══════════════════════════════════════════════════════════

> 🔍 Select a demo script to run: basic_profiling

Running demo script: basic_profiling

⚙️  Thag options (press Enter to skip):
> Force rebuild? Yes
> Display timings? Yes
> Features to enable (comma-separated): 
> Just generate, don't run? No
> Just build, don't run? No
> Just check, don't run? No

📝 This demo accepts command-line arguments.
💡 Sample arguments: demo/data.txt
   (don't type '--' in interactive mode)

> Enter arguments (paths relative to /Users/you/projects/thag_rs): 
  (press Enter to skip)

🌍 Environment Variables (press Enter to skip):
💡 Suggested for profiling demos: THAG_PROFILER=both

> Enter environment variables (format: KEY=value, separate multiple with spaces): THAG_PROFILER=both

🌍 Set environment variable: THAG_PROFILER=both

... (demo runs with profiling enabled) ...
```

### Environment Variable Format in Interactive Mode

**Single variable:**
```
THAG_PROFILER=both
```

**Multiple variables (space-separated):**
```
THAG_PROFILER=both RUST_LOG=debug
```

**With quotes (if values contain spaces):**
```
VAR1=value1 "VAR2=value with spaces" VAR3=value3
```

## Practical Examples

### Example 1: Basic Profiling Demo

**Command-line:**
```bash
cargo run -p thag_demo -- script basic_profiling -E THAG_PROFILER=both
```

**What happens:**
1. Demo runs with profiling enabled
2. Generates `.folded` files in the output directory
3. Can be visualized with flamegraph tools

### Example 2: Memory Profiling Only

**Command-line:**
```bash
cargo run -p thag_demo -- script memory_profiling -E THAG_PROFILER=memory
```

**Result:**
- Only memory profiling data is collected
- Faster than collecting both time and memory

### Example 3: Debug Mode with Profiling

**Command-line:**
```bash
cargo run -p thag_demo -- script profiling_demo \
  -f \
  -E THAG_PROFILER=both \
  -E RUST_LOG=debug \
  -- demo/input.txt
```

**Result:**
- Forces rebuild (useful after code changes)
- Collects full profiling data
- Shows detailed debug logs
- Processes `demo/input.txt` as argument

### Example 4: Comparison Demo with Custom Settings

**Command-line:**
```bash
cargo run -p thag_demo -- script comparison \
  -E THAG_PROFILER=time \
  -E RUST_LOG=info \
  --features comparison_mode
```

## Tips and Best Practices

### 1. Always Use THAG_PROFILER for Profiling Demos

Without this variable, profiling demos run but don't generate output files:

```bash
# ❌ Runs but no profiling output
cargo run -p thag_demo -- script basic_profiling

# ✅ Generates profiling data
cargo run -p thag_demo -- script basic_profiling -E THAG_PROFILER=both
```

### 2. Check Demo Suggestions

Interactive mode suggests relevant environment variables:

```
💡 Suggested for profiling demos: THAG_PROFILER=both
```

Pay attention to these suggestions!

### 3. Use Verbose Mode to See Environment Setup

```bash
cargo run -p thag_demo -- -v script basic_profiling -E THAG_PROFILER=both
```

This shows:
```
🌍 Set environment variable: THAG_PROFILER=both
```

### 4. Verify Environment Variables Were Set

If a demo doesn't behave as expected, check that:
- Variable name is spelled correctly (case-sensitive!)
- Format is `KEY=value` with no spaces around `=`
- Multiple variables are properly separated

### 5. Common Mistakes to Avoid

**❌ Spaces around equals:**
```bash
-E THAG_PROFILER = both  # Wrong!
```

**✅ Correct format:**
```bash
-E THAG_PROFILER=both    # Right!
```

**❌ Quotes in wrong place:**
```bash
-E "THAG_PROFILER"="both"  # Unnecessary
```

**✅ Only quote if value has spaces:**
```bash
-E "MY_VAR=value with spaces"  # OK when needed
```

## Environment Variables by Demo Type

### Profiling Demos
- `basic_profiling`
- `memory_profiling`
- `async_profiling`
- `interactive_profiling`

**Recommended:**
```bash
-E THAG_PROFILER=both
```

### Comparison Demos
- `comparison`
- `differential_comparison`

**Recommended:**
```bash
-E THAG_PROFILER=time -E RUST_LOG=info
```

### Benchmark Demos
- `benchmark`
- `flamegraph`

**Recommended:**
```bash
-E THAG_PROFILER=both -E RUST_LOG=error
```

### General Demos

Most demos work fine without environment variables, but you can always add `RUST_LOG` for debugging:

```bash
cargo run -p thag_demo -- script any_demo -E RUST_LOG=debug
```

## Troubleshooting

### Problem: No profiling output generated

**Solution:** Make sure `THAG_PROFILER` is set:
```bash
cargo run -p thag_demo -- script basic_profiling -E THAG_PROFILER=both
```

### Problem: "Invalid env var format" warning

**Cause:** Missing `=` or incorrect format

**Fix:**
```bash
# ❌ Wrong
-E THAG_PROFILER

# ✅ Correct
-E THAG_PROFILER=both
```

### Problem: Environment variable not taking effect

**Check:**
1. Spelling and case (environment variables are case-sensitive)
2. No spaces around `=`
3. Variable is recognized by the demo (check demo documentation)

### Problem: Need to unset a variable

Environment variables only apply to the current demo run. Just run again without `-E` flag.

## See Also

- `USAGE_IMPROVEMENTS.md` - Complete usage guide
- Main thag_rs documentation - More on profiling features
- `cargo run -p thag_demo -- script --help` - Command-line reference