# Documentation Strategy for thag_profiler

This document outlines the dual-level documentation strategy implemented for `thag_profiler` to maintain a clean public API while providing comprehensive internal documentation.

## Problem Statement

The `thag_profiler` library has many functions that need to be `pub` for technical reasons:
- Cross-module usage within the crate
- Access by procedural macros
- Integration with generated code

However, most of these functions are implementation details that shouldn't clutter the public API documentation for end users.

## Solution: Feature-Gated Documentation

We implemented a feature-based approach using the `internal_docs` feature flag to control documentation visibility.

### Key Components

1. **Feature Flag**: `internal_docs` in `Cargo.toml`
2. **Convenience Macro**: `#[internal_doc]` attribute macro for easy marking
3. **Manual Attribute**: `#[cfg_attr(not(feature = "internal_docs"), doc(hidden))]` for edge cases
4. **Private Items**: `--document-private-items` flag for comprehensive internal docs
5. **Organized Re-exports**: Grouped public API items with clear documentation
6. **Build Scripts**: Automated documentation generation for different audiences

## Implementation Details

### 1. Cargo.toml Configuration

```toml
[features]
## Enable documentation of internal APIs and implementation details.
internal_docs = []

[package.metadata.docs.rs]
all-features = true
default-features = false
features = [
    "document-features",
    "full_profiling",
    "debug_logging",
    "internal_docs",
]
```

### 2. Code Attribution Patterns

#### Option A: Using the `#[internal_doc]` macro (recommended)

```rust
use thag_profiler::internal_doc;

/// Internal utility function for formatting numbers.
///
/// This is used internally by the profiling system for output formatting.
#[internal_doc]
pub fn thousands<T: Display>(n: T) -> String {
    // Implementation...
}
```

#### Option B: Using the manual attribute

For contexts where the macro isn't available (e.g., binary crates):

```rust
/// Internal utility function for formatting numbers.
///
/// This is used internally by the profiling system for output formatting.
#[cfg_attr(not(feature = "internal_docs"), doc(hidden))]
pub fn thousands<T: Display>(n: T) -> String {
    // Implementation...
}
```

#### Option C: Private items (no attribute needed)

For truly private implementation details:

```rust
/// Private helper function.
///
/// This is only visible with --document-private-items.
fn private_helper() {
    // Implementation...
}
```

### 3. Module Organization

```rust
/// Core profiling functionality and configuration.
pub use {
    errors::{ProfileError, ProfileResult},
    profiling::{
        disable_profiling, is_profiling_enabled, Profile, ProfileConfiguration, ProfileType,
    },
    thag_proc_macros::{fn_name, internal_doc},
};

/// Advanced profiling configuration and utilities.
///
/// These are typically used by advanced users or the profiling macros themselves.
#[internal_doc]
pub use profiling::{
    clear_profile_config_cache, get_config_profile_type, get_global_profile_type,
    is_detailed_memory, parse_env_profile_config, strip_hex_suffix_slice,
};
```

## Usage

### For Library Users (Public API)

Generate clean, focused documentation:

```bash
cargo doc --package thag_profiler --features document-features,full_profiling,debug_logging --no-deps
```

**Shows:**
- Essential public APIs
- Core profiling attributes
- Main configuration types
- User-facing functions only

### For Developers (Internal API)

Generate comprehensive documentation:

```bash
cargo doc --package thag_profiler --features document-features,full_profiling,debug_logging,internal_docs --no-deps
```

**Shows:**
- Everything from public API
- Internal utility functions
- Debug infrastructure
- Implementation details
- Development tools

### For Comprehensive Development (Internal + Private)

Generate documentation including private items:

```bash
cargo doc --package thag_profiler --features document-features,full_profiling,debug_logging,internal_docs --no-deps --document-private-items
```

**Shows:**
- Everything from internal docs
- Private functions and modules
- Private struct fields and methods
- Complete implementation details

### Automated Tools

We provide several tools to make this easy:

1. **Shell Scripts**:
   - `doc_examples.sh` - Demonstrates both modes (including private items)
   - `verify_docs.sh` - Verifies the feature works correctly

2. **Makefile Targets**:
   - `make doc-public` - Build public API docs
   - `make doc-internal` - Build internal docs (with private items)
   - `make doc-open` - Build and open public docs
   - `make doc-internal-open` - Build and open internal docs

3. **Examples**:
   - `examples/doc_attributes.rs` - Shows all attribution patterns

## Benefits

### For Library Users
- **Clean Documentation**: Only see what they need to use
- **Focused Learning**: No distraction from implementation details
- **Better Discoverability**: Important functions aren't buried in utilities

### For Developers
- **Complete Reference**: All functions and modules documented
- **Implementation Context**: Understanding how components work together
- **Debugging Aid**: Access to internal debugging tools and utilities

### For Maintainers
- **Flexibility**: Can document everything without cluttering user experience
- **CI/CD Integration**: Can verify both documentation modes work
- **docs.rs Compatibility**: Full documentation published online

## Best Practices

### When to Use Different Attribution Patterns

#### Use `#[internal_doc]` macro for:
- Internal utility functions in the main library
- Implementation helpers
- Debug/logging infrastructure
- Advanced configuration functions
- Functions required to be `pub` for technical reasons

#### Use `#[cfg_attr(not(feature = "internal_docs"), doc(hidden))]` for:
- Binary crates (where the macro isn't available)
- Macro exports that have import issues
- Legacy code being migrated

#### Use private items (no attribute) for:
- True implementation details
- Helper functions that don't need to be `pub`
- Module-internal utilities
- Functions that can't be called from other modules

#### Don't use any attribute for:
- Core user-facing APIs
- Main feature functions
- Public configuration types
- Functions users should directly call

### Documentation Guidelines

1. **Always document internal items**: Even if hidden, they should have good docs
2. **Use clear module organization**: Group related functionality
3. **Provide examples**: Show how to use all documentation modes
4. **Maintain consistency**: Apply the pattern uniformly across the codebase
5. **Prefer the macro**: Use `#[internal_doc]` over manual attributes when possible
6. **Consider privacy**: Use private items when they don't need to be `pub`

### Testing Strategy

1. **Automated verification**: Use `verify_docs.sh` to ensure hiding/showing works
2. **Manual review**: Periodically check all documentation modes
3. **CI integration**: Include documentation builds in continuous integration
4. **User feedback**: Monitor for confusion about missing/too many functions
5. **Example maintenance**: Keep `examples/doc_attributes.rs` up to date

## Alternative Approaches Considered

### 1. Multiple Crates
- **Pros**: Natural separation, clear public API
- **Cons**: Complex build setup, macro complications, maintenance overhead

### 2. Pure `#[doc(hidden)]`
- **Pros**: Simple to implement
- **Cons**: No way to generate internal docs, loses implementation context

### 3. Separate Documentation Features
- **Pros**: More granular control
- **Cons**: Complex feature matrix, user confusion

## Conclusion

The feature-gated documentation approach provides the best balance of:
- Clean public API for users
- Complete internal documentation for developers
- Maintainable codebase with good separation of concerns
- Compatibility with docs.rs and standard tooling

This pattern can be applied to other Rust libraries facing similar challenges with internal vs. public APIs.

## Related Files

- `Cargo.toml` - Feature definition and docs.rs configuration
- `doc_examples.sh` - Example script showing both modes
- `verify_docs.sh` - Verification script for testing
- `Makefile` - Convenient build targets
- `README.md` - User-facing documentation explanation
- `lib.rs` - Implementation of the pattern
- `examples/doc_attributes.rs` - Comprehensive examples of all approaches
