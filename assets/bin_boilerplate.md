## Running the built-in tools

`thag_rs` includes several built-in tools that are compiled as separate binaries. These tools are available after installing `thag_rs` with the `tools` feature enabled.

### Installation with tools

```bash
cargo install thag_rs --features tools
```

### Basic usage

Each tool can be run directly by name:

```bash
thag_convert_themes --help
thag_clippy
thag_gen_readme
# ... etc
```

### Getting help

Most tools support `--help` or `-h` for usage information:

```bash
thag_convert_themes --help
thag_clippy --help
```

### Tool categories

The tools are organized into several categories:

- **Development tools**: Code analysis, formatting, and development utilities
- **Theme tools**: Theme conversion and management utilities  
- **Documentation tools**: README generation and documentation utilities
- **Analysis tools**: AST analysis, profiling, and debugging tools
- **Utility tools**: General-purpose utilities and helpers

### Building from source

If you're building from source, you can build all tools with:

```bash
cargo build --features tools
```

Or build individual tools:

```bash
cargo build --bin thag_convert_themes --features tools
```

### Integration with thag

Many of these tools integrate with the main `thag` command and can be used as part of your Rust scripting workflow.
