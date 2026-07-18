# thag_profiler Internals

Developer notes on architecture, known gotchas, and lessons learnt — intended as
context for future AI-assisted work on this crate.

## Feature flags

| Feature | Implies | Effect |
|---|---|---|
| `time_profiling` | — | Enables all time-based profiling |
| `full_profiling` | `time_profiling` | Adds memory tracking via `MultiAllocator` custom global allocator |
| `debug_logging` | — | Enables `debug_log!` macro and the `DebugLogger` / debug-log-file mechanism. **Without this, `THAG_PROFILER=...,announce` is silently ignored.** |

`PROFILING_FEATURE` (a compile-time `const bool`) is `true` only when
`time_profiling` is active and the build is not a test build. It guards
`is_profiling_enabled()` together with the runtime `PROFILING_STATE` atomic.

## Key data-flow: raw data → flamegraph input

```
Profile::drop
  └─ write_profraw_event
       └─ write_profile_event  ← opens ProfrawProfileFile lazily; flushes after every write
            └─ {stem}-{timestamp}.profraw   (raw per-call timing lines, written live)

finalize_profiling
  └─ process_time_profile
       ├─ parse_profraw_file
       ├─ subtract_child_overhead
       ├─ process_profraw_to_folded  →  {stem}-{timestamp}-inclusive.folded
       └─ convert_to_exclusive_time  →  {stem}-{timestamp}.folded   (exclusive / flamegraph input)
```

The `.profraw` lines are flushed to disk on every `Profile::drop`, so they survive
an unclean exit. The two `.folded` files are **only created during
`finalize_profiling`** — if that never runs (e.g. `process::exit()` from a GUI
framework), those files are missing even though the `.profraw` has good data.

## Initialization sequence (important ordering)

```
init_profiling
  └─ enable_profiling(true, profile_type)
       ├─ get_profile_config()          ← MUST happen before ProfilePaths::get()
       ├─ set_global_profile_type(...)
       └─ initialize_profile_files(...)
            └─ ProfilePaths::get()      ← first call; initializes static using config
```

`ProfilePaths` is a `static_lazy!` (`OnceLock`-backed) that calls
`get_profile_config()` on first access to embed the **absolute** output directory
into every stored path string. This must not be called before `enable_profiling`
sets the config cache.

## Absolute paths in ProfilePaths (post-fix)

Before the fix, `ProfilePaths` stored bare filenames (`script-timestamp.profraw`)
resolved against the CWD at use time. Scripts that call `std::env::set_current_dir()`
(e.g. a file-picker that navigates to the selected file's parent) silently redirected
output to the new directory, leaving empty stubs in the original directory.

Fix applied: `TryFrom<&[&str]> for ProfileConfiguration` now canonicalizes
`output_dir` to an absolute path immediately, and `ProfilePaths` joins it into all
stored path strings.

## Rust v0 symbol mangling (Rust ≥ 1.97 default)

v0 mangling changed how demangled backtrace symbol names look:

| Mangling | Example |
|---|---|
| Legacy | `thag_profiler::profiling::Profile::new` |
| v0 | `<thag_profiler[8ec0060a]::profiling::Profile>::new` |

Consequences for `extract_profile_callstack`:

- **START_PATTERN** (`"Profile::new"`) no longer matches v0 names. Fixed by also
  checking `name.contains("Profile") && name.ends_with(">::new")`.
- **SCAFFOLDING_PATTERNS** like `"std::rt::lang_start"` no longer match
  `"std[HASH]::rt::lang_start"`. Raw backtrace names include `[HASH]`; the
  scaffolding filter runs on raw names **before** `clean_function_name` strips hashes.
  So runtime frames can leak into the cleaned callstack as noise (harmless but visible).
- **Closures**: legacy mangling used `::{{closure}}`; v0 uses `::{closure#N}`
  (single braces). `clean_function_name` now strips both.
- **Crate hash stripping**: `clean_function_name` strips `[xxxxxxxxxxxxxxxx]`
  (exactly 16 hex digits) from v0-mangled crate disambiguators.

## `extract_profile_callstack` internals

- Walks the backtrace from innermost (current) to outermost.
- Everything before `START_PATTERN` is suppressed.
- The frame immediately after `Profile::new` is the profiled function.
- `SCAFFOLDING_PATTERNS` filters out runtime/async scaffolding.
- `already_seen` deduplicates repeated frames (e.g. recursion tail entries).
- Returns an empty `Vec` if the START_PATTERN is never found.

Both `Profile::new` variants (`full_profiling` and `time_profiling`-only) guard
against an empty callstack with a `warn_once!` that prints to stderr, then return
`None`. The `time_profiling`-only variant previously **panicked** at `cleaned_stack[0]`
on an empty stack — this was the original bug symptom before the mangling fix.

## `safe_alloc!` and the `full_profiling` allocator

`full_profiling` installs `MultiAllocator` as the global allocator. `safe_alloc!`
switches the per-thread `USING_SYSTEM_ALLOCATOR` TLS flag to `true` for its body,
making all allocations within use the system allocator (so they are not double-counted
by the memory tracker).

**Important**: `return` inside a `safe_alloc!` body exits the enclosing *function*,
not just the block. The `safe_alloc!`'s cleanup (`set_using_system(false)`) is only
skipped if `was_already_using_sys` was `true` (meaning the outer context had already
switched), so state is correctly maintained. But if you add a bare `return` inside
a `safe_alloc!` when `was_already_using_sys = false`, the TLS flag is leaked as
`true` until the next `safe_alloc!` call resets it.

## `finalize_profiling` idempotency

A `PROFILING_FINALIZED: AtomicBool` prevents double-processing if both the explicit
call (from `#[enable_profiling]` generated code) and the atexit handler fire.
`enable_profiling(true, ...)` resets it to `false` so a new profiling session (e.g.
in tests) can finalize again.

## atexit handler

Registered in `init_profiling` via raw `extern "C" { fn atexit(...) }`. Intended
to catch `std::process::exit()` from GUI frameworks (e.g. eframe/winit on macOS
with Cmd-Q) that bypass the normal Rust call stack.

**Limitation with `full_profiling`**: the atexit handler calls `finalize_profiling`
which uses `safe_alloc!`. During `atexit`, TLS may be in an indeterminate state on
macOS (thread-local destructors may have already run), causing the allocator switching
to fail silently. The result: `.profraw` survives (written live with `flush()`), but
the `.folded` files are not created. The atexit handler is reliable for
`time_profiling`-only builds (no custom allocator, no TLS switching).

**Workaround for `full_profiling` GUI apps**: add a Quit button (or Cmd-W shortcut)
that sends `egui::ViewportCommand::Close`, causing `eframe::run_native` to return
normally through the Rust call stack, running `finalize_profiling` explicitly.

## eframe / winit on macOS

Cmd-Q on macOS goes through the Cocoa app delegate (`applicationShouldTerminate:`).
In some winit versions this causes `process::exit()` rather than a clean return from
the event loop. Cmd-W and in-app Quit buttons that use `ViewportCommand::Close` cause
a clean event-loop exit and are preferred for profiled GUI apps.

## Test suite requirements

All `thag_profiler` tests **must** run with `--test-threads=1`. The tests share
global singletons (`PROFILING_MUTEX`, `PROFILING_STATE`, `GLOBAL_PROFILE_TYPE`,
`PROFILE_CONFIG_CACHE`, `ProfilePaths`, etc.) that are not safe for concurrent test
execution.

CI command:
```
cargo test -p thag_profiler --features full_profiling -- --test-threads=1
```

## `warn_once!` behaviour

The three-argument form `warn_once!(condition, warning_fn, return_expr)` expands to:
```rust
if warn_once!(condition, warning_fn) { return_expr }
```
The warning is printed only on the **first** call where the condition is true
(guarded by a `static AtomicBool`). But `return_expr` executes on **every** call
where the condition is true, not just the first. So a `warn_once!(!is_enabled, ...,
return None)` silently returns `None` on every subsequent call without re-printing
the warning.

## `ProfrawProfileFile` lazy open

`write_profile_event` opens the BufWriter on the **first call** (lazily):
```rust
if guard.is_none() {
    *guard = Some(BufWriter::new(OpenOptions::new().create(true).append(true).open(path)?));
}
```
It then calls `writer.flush()` after every write. Errors are silently discarded
(`let _ = self.write_profraw_event(...)`). If the path is wrong or the file can't
be opened, no data is written and no error is surfaced.

## `ProfileConfiguration` output_dir parsing

`TryFrom<&[&str]>` parses `THAG_PROFILER=type,dir,debug_level[,detailed_memory]`.
The `dir` field is canonicalized to an absolute path at parse time. If the directory
doesn't exist yet, it falls back to `current_dir().join(raw_dir)`. The `Default`
impl also uses `current_dir()`. Always absolute after construction.
