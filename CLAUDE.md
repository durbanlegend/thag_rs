# thag_rs Development Guide

## Build Commands
- Build: `cargo build`
- Run: `cargo run -- [args]`
- Test all: `cargo test`
- Test single: `cargo test test_name -- --nocapture`
- Integration test: `cargo test --test integration_test`
- Lint: `cargo clippy --all-targets --all-features`
- Format: `cargo fmt`
- Flamegraph: `cargo flamegraph`
- Profile: `cargo run --features profiling -- [args]`

## Code Style Guidelines
- **Imports**: Group std imports first, then external crates, then internal modules
- **Conditional imports**: Use `#[cfg(feature = "feature_name")]` for feature-gated imports
- **Error handling**: Use `ThagResult<T>` and wrap errors with appropriate `From` implementations
- **Naming**: CamelCase for types, snake_case for functions and variables
- **Documentation**: Document all public items, especially interfaces and non-obvious behavior
- **Profiling**: Use `#[profile]` attribute on functions that should be profiled
- **Features**: Clearly mark feature-dependent code with `#[cfg(feature = "feature_name")]`
- **Testing**: Write unit tests for modules, with integration tests for full workflows
- **Formatting**: Follow rustfmt conventions; run `cargo fmt` before committing