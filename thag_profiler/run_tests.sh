THAG_PROFILER=both,,announce cargo test --features=full_profiling --test test_enable_profiling -- --nocapture || exit
THAG_PROFILER=both,,announce cargo test --features=full_profiling --test test_mem_attribution -- --nocapture || exit
THAG_PROFILER=both,,announce cargo test --features=full_profiling --test test_mem_tracking -- --nocapture || exit
THAG_PROFILER=both,,announce cargo test --features=full_profiling --test test_profile_section -- --nocapture || exit
THAG_PROFILER=both,,announce cargo test --features=full_profiling --test test_profiled_behavior -- --nocapture || exit
THAG_PROFILER=time,,announce cargo test --features=time_profiling --test test_profiling -- --nocapture || exit
THAG_PROFILER=both,,announce cargo test --features=full_profiling --test test_profiling -- --nocapture || exit
