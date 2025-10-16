# thag_rs Project-Specific Claude Instructions

## Testing Guidelines
- **Test Serialization**: Do NOT serialize unit tests. Tests should be designed to run independently, even if they modify global state.
- **Integration Tests**: When writing integration tests, ensure they can run in parallel with other tests.

## thag_profiler Design Constraints
- **Thread-Local Storage**: Do NOT propose or implement Thread-Local Storage (TLS) in the thag_profiler crate. The profiler must work with alternative thread scheduling models.
- **Global State**: The profiler uses controlled global state by design; do not attempt to refactor this into thread-local state.

## Feature Flag Management
- **Feature Granularity**: Maintain the current feature flag architecture. Do not over-complicate feature flags.
- **Runtime Capabilities**: Use the bitflags approach for runtime capability detection.

## Profiling Macros
- **Macro Testing**: Ensure profiling macros have comprehensive test coverage.
- **Test Isolation**: Tests for profiling macros must be isolated and run sequentially when they modify global state.