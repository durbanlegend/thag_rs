

I'm iterating through a backtrace::Backtrace and want to extract the symbols from each frame, but only from the frame where the symbol name contains "thag_rs::profiling::Profile::new", and only until the symbol name contains "std::sys::backtrace::__rust_begin_short_backtrace".
Currently this is printing all symbols, so how can I modify it as above?:
```
let mut current_backtrace = Backtrace::new_unresolved();
current_backtrace.resolve();
Backtrace::frames(&current_backtrace)
    .into_iter()
    .for_each(|frame| {
        for symbol in frame.symbols() {
            eprintln!("name={:?}", symbol.name());
        }
    });
```


I'll help you modify your code to only print symbols between the specified boundary conditions. Here's how to change it:

```rust
let mut current_backtrace = Backtrace::new_unresolved();
current_backtrace.resolve();
let mut is_within_target_range = false;

Backtrace::frames(&current_backtrace)
    .into_iter()
    .for_each(|frame| {
        for symbol in frame.symbols() {
            if let Some(name) = symbol.name() {
                let name_str = name.to_string();

                // Check if we've reached the start condition
                if !is_within_target_range && name_str.contains("thag_rs::profiling::Profile::new") {
                    is_within_target_range = true;
                }

                // Print only if we're within our target range
                if is_within_target_range {
                    eprintln!("name={:?}", name);
                }

                // Check if we've reached the end condition
                if is_within_target_range && name_str.contains("std::sys::backtrace::__rust_begin_short_backtrace") {
                    is_within_target_range = false;
                    break; // Exit this frame's symbols loop
                }
            }
        }
    });
```

This code:
1. Introduces a flag `is_within_target_range` to track when we're between the start and end conditions
2. Sets the flag to `true` when we find the starting symbol
3. Only prints symbols when the flag is `true`
4. Sets the flag to `false` and breaks out of the inner loop when we find the ending symbol

This will give you all symbols starting from the frame containing "thag_rs::profiling::Profile::new" up to and including the frame with "std::sys::backtrace::__rust_begin_short_backtrace".

Thanks!
The names are all in the format:
```
name=thag_config_builder_proto_profile::prompt_config::{{closure}}::h888992f266362a53
name=thag_config_builder_proto_profile::main::{{closure}}::{{closure}}::h1edb8870173c691d
name=tokio::runtime::park::CachedParkThread::block_on::{{closure}}::h973947f73b43550c
```
What's the simplest way to extract the function name from these names?

The names you're seeing are Rust's mangled function names, which include the module path, function name, and a unique hash. To extract just the function name (without the hash and closure details), you can use a regular expression or string manipulation.

Here's a simple approach:

```rust
let mut current_backtrace = Backtrace::new_unresolved();
current_backtrace.resolve();
let mut is_within_target_range = false;

Backtrace::frames(&current_backtrace)
    .into_iter()
    .for_each(|frame| {
        for symbol in frame.symbols() {
            if let Some(name) = symbol.name() {
                let name_str = name.to_string();

                // Check if we've reached the start condition
                if !is_within_target_range && name_str.contains("thag_rs::profiling::Profile::new") {
                    is_within_target_range = true;
                }

                // Print only if we're within our target range
                if is_within_target_range {
                    // Extract the function name
                    let function_name = extract_function_name(&name_str);
                    eprintln!("function={}", function_name);
                }

                // Check if we've reached the end condition
                if is_within_target_range && name_str.contains("std::sys::backtrace::__rust_begin_short_backtrace") {
                    is_within_target_range = false;
                    break;
                }
            }
        }
    });

// Helper function to extract the function name
fn extract_function_name(full_name: &str) -> String {
    // Split by :: to get module segments
    let parts: Vec<&str> = full_name.split("::").collect();

    // Find the function name (typically the last part before any {{closure}} or hash)
    for (i, part) in parts.iter().enumerate() {
        if part.starts_with("{{closure}}") || part.starts_with('h') && part.len() > 10 && part[1..].chars().all(|c| c.is_digit(16)) {
            if i > 0 {
                return parts[i-1].to_string();
            }
        }
    }

    // If we can't identify a clear function name, use a simpler approach
    // Just take the last part that's not a hash or closure
    for part in parts.iter().rev() {
        if !part.starts_with("{{closure}}") && !(part.starts_with('h') && part.len() > 10 && part[1..].chars().all(|c| c.is_digit(16))) {
            return part.to_string();
        }
    }

    // Fallback to the original
    full_name.to_string()
}
```

For a more accurate solution using regular expressions, you could add the `regex` crate to your dependencies and use:

```rust
use regex::Regex;

fn extract_function_name(full_name: &str) -> String {
    // This regex extracts the module path and function name without the hash
    let re = Regex::new(r"^(.*?)::(?:{{closure}}::)*h[0-9a-f]+$").unwrap();

    if let Some(caps) = re.captures(full_name) {
        let path = caps.get(1).unwrap().as_str();
        let parts: Vec<&str> = path.split("::").collect();
        if !parts.is_empty() {
            return parts.last().unwrap().to_string();
        }
    }

    // Fallback: just return anything before the hash
    full_name.split("::h").next().unwrap_or(full_name).to_string()
}
```

This should extract just the function name from the mangled names in most cases. For more complex scenarios, you might want to look into using the `rustc-demangle` crate for proper demangling of Rust symbols.

Thanks, can you show me how to use rustc-demangle, and also how to exclude all functions beginning with "tokio::runtime"?

# Using `rustc-demangle` and Filtering Tokio Runtime Functions

Here's how to use the `rustc-demangle` crate to properly demangle Rust symbols and exclude functions beginning with "tokio::runtime":

First, add `rustc-demangle` to your dependencies in `Cargo.toml`:

```toml
[dependencies]
backtrace = "0.3"
rustc-demangle = "0.1"
```

Now, here's the updated code:

```rust
use backtrace::Backtrace;
use rustc_demangle::demangle;

fn main() {
    let mut current_backtrace = Backtrace::new_unresolved();
    current_backtrace.resolve();
    let mut is_within_target_range = false;

    Backtrace::frames(&current_backtrace)
        .into_iter()
        .for_each(|frame| {
            for symbol in frame.symbols() {
                if let Some(name) = symbol.name() {
                    let name_str = name.to_string();

                    // Check if we've reached the start condition
                    if !is_within_target_range && name_str.contains("thag_rs::profiling::Profile::new") {
                        is_within_target_range = true;
                    }

                    // Skip if we're not in the target range or it's a tokio::runtime function
                    if !is_within_target_range || name_str.starts_with("tokio::runtime") {
                        continue;
                    }

                    // Demangle the symbol
                    let demangled = demangle(&name_str).to_string();

                    // Print the demangled symbol
                    eprintln!("function={}", demangled);

                    // Check if we've reached the end condition
                    if name_str.contains("std::sys::backtrace::__rust_begin_short_backtrace") {
                        is_within_target_range = false;
                        break;
                    }
                }
            }
        });
}
```

## Extracting Just the Function Name

If you want to extract just the function name from the demangled symbol, you can add a helper function:

```rust
fn extract_function_name(demangled: &str) -> String {
    // Split by :: to get module segments
    let parts: Vec<&str> = demangled.split("::").collect();

    // Find the last part that's not a closure or hash
    for part in parts.iter().rev() {
        if !part.contains("closure") && !part.starts_with('h') {
            return part.to_string();
        }
    }

    // Fallback
    demangled.to_string()
}

// Then use it like this:
let demangled = demangle(&name_str).to_string();
let function_name = extract_function_name(&demangled);
eprintln!("function={}", function_name);
```

## Complete Example

Here's a complete example that:
1. Uses `rustc-demangle` to properly demangle symbols
2. Skips any functions that start with "tokio::runtime"
3. Extracts clean function names from the demangled symbols

```rust
use backtrace::Backtrace;
use rustc_demangle::demangle;

fn main() {
    let mut current_backtrace = Backtrace::new_unresolved();
    current_backtrace.resolve();
    let mut is_within_target_range = false;

    Backtrace::frames(&current_backtrace)
        .into_iter()
        .for_each(|frame| {
            for symbol in frame.symbols() {
                if let Some(name) = symbol.name() {
                    let name_str = name.to_string();

                    // Check if we've reached the start condition
                    if !is_within_target_range && name_str.contains("thag_rs::profiling::Profile::new") {
                        is_within_target_range = true;
                    }

                    // Process symbols within our target range
                    if is_within_target_range {
                        // Skip tokio::runtime functions
                        if name_str.starts_with("tokio::runtime") {
                            continue;
                        }

                        // Demangle the symbol
                        let demangled = demangle(&name_str).to_string();

                        // Extract just the function name
                        let function_name = extract_function_name(&demangled);

                        // Print it
                        eprintln!("function={}", function_name);

                        // Check if we've reached the end condition
                        if name_str.contains("std::sys::backtrace::__rust_begin_short_backtrace") {
                            is_within_target_range = false;
                            break;
                        }
                    }
                }
            }
        });
}

fn extract_function_name(demangled: &str) -> String {
    // Split the demangled name by "::"
    let parts: Vec<&str> = demangled.split("::").collect();

    // Find the last meaningful part (not a closure or hash)
    for part in parts.iter().rev() {
        if !part.contains("closure") && !part.contains("{closure}") &&
           !(part.starts_with('h') && part.chars().skip(1).all(|c| c.is_digit(16))) {
            return part.to_string();
        }
    }

    // If we couldn't find a good function name, return the full demangled string
    demangled.to_string()
}
```

This will give you readable function names while skipping any tokio::runtime functions, making your output much cleaner.

I'm looking for a way to collapse this pattern, which is due to the generate_async_wrapper function of the profile proc macro implementation:
```
name=thag_config_builder_proto_profile::prompt_config::{{closure}}::{{closure}}::hc58248661e144efb
name=<thag_config_builder_proto_profile::prompt_config::{{closure}}::ProfiledFuture<F> as core::future::future::Future>::poll::ha46568dfcb2f94bb
name=thag_config_builder_proto_profile::prompt_config::{{closure}}::h888992f266362a53
```
I think that the bottom one of these (the first chronologically) is the one we want. It then creates a ProfiledFuture (the middle entry) with the name as a generic type parameter, which invokes Profile::new. I'm not sure exactly where the top entry with the double closure comes in - is it from the execution of the future or the poll? - but I'd like a clean way to fold these 3 entries into one, since AFAIK the top 2 are just scaffolding for the bottom one. Obviously this must work irrespective of module name and function name.

# Collapsing ProfiledFuture Stack Frames

You're right that this pattern comes from your `generate_async_wrapper` macro, with the bottom entry being the original function call, the middle entry being the `Future::poll` implementation, and the top entry likely being an additional closure wrapper for the async machinery.

Here's how to collapse these related entries into one meaningful entry:

```rust
use backtrace::Backtrace;
use rustc_demangle::demangle;
use std::collections::VecDeque;

fn main() {
    let mut current_backtrace = Backtrace::new_unresolved();
    current_backtrace.resolve();
    let mut is_within_target_range = false;

    // Buffer to hold recent frames for collapsing related ProfiledFuture entries
    let mut recent_frames: VecDeque<String> = VecDeque::with_capacity(3);

    Backtrace::frames(&current_backtrace)
        .into_iter()
        .for_each(|frame| {
            for symbol in frame.symbols() {
                if let Some(name) = symbol.name() {
                    let name_str = name.to_string();

                    // Check if we've reached the start condition
                    if !is_within_target_range && name_str.contains("thag_rs::profiling::Profile::new") {
                        is_within_target_range = true;
                    }

                    // Process symbols within our target range
                    if is_within_target_range {
                        // Skip tokio::runtime functions
                        if name_str.starts_with("tokio::runtime") {
                            continue;
                        }

                        // Demangle the symbol
                        let demangled = demangle(&name_str).to_string();

                        // Add to our recent frames buffer
                        recent_frames.push_back(demangled.clone());
                        if recent_frames.len() > 3 {
                            // Process the oldest frame
                            let oldest = recent_frames.pop_front().unwrap();

                            // Check if we need to collapse a ProfiledFuture pattern
                            if should_print_frame(&recent_frames, &oldest) {
                                let clean_name = extract_function_name(&oldest);
                                eprintln!("function={}", clean_name);
                            }
                        }

                        // Check if we've reached the end condition
                        if name_str.contains("std::sys::backtrace::__rust_begin_short_backtrace") {
                            is_within_target_range = false;
                            break;
                        }
                    }
                }
            }
        });

    // Process any remaining frames
    while !recent_frames.is_empty() {
        let frame = recent_frames.pop_front().unwrap();
        if should_print_frame(&recent_frames, &frame) {
            let clean_name = extract_function_name(&frame);
            eprintln!("function={}", clean_name);
        }
    }
}

// Determines if we should print a frame or if it's part of a pattern to collapse
fn should_print_frame(recent_frames: &VecDeque<String>, current_frame: &str) -> bool {
    // Don't print if this is part of a ProfiledFuture sequence
    if current_frame.contains("ProfiledFuture") && current_frame.contains("::poll::") {
        return false;
    }

    // Don't print double-closure wrappers if they're followed by a ProfiledFuture
    if current_frame.contains("{{closure}}::{{closure}}") &&
       !recent_frames.is_empty() &&
       recent_frames[0].contains("ProfiledFuture") {
        return false;
    }

    // Don't print the closure that's part of the ProfiledFuture pattern
    // We only want to print the original function
    if recent_frames.len() >= 2 &&
       current_frame.contains("{{closure}}") &&
       recent_frames[0].contains("{{closure}}::{{closure}}") &&
       recent_frames[1].contains("ProfiledFuture") {
        return false;
    }

    // Print all other frames
    true
}

fn extract_function_name(demangled: &str) -> String {
    // Split the demangled name by "::"
    let parts: Vec<&str> = demangled.split("::").collect();

    // Find the module path (everything before the last part)
    let module_path = parts.iter()
        .take(parts.len().saturating_sub(1))
        .fold(String::new(), |acc, part| {
            if acc.is_empty() { part.to_string() } else { format!("{}::{}", acc, part) }
        });

    // Find the function name (last part without closure/hash noise)
    let mut function_name = parts.last().unwrap_or(&"").to_string();

    // Clean up function name - remove hash if present
    if let Some(idx) = function_name.find("::h") {
        function_name = function_name[..idx].to_string();
    }

    // Remove closure markers
    function_name = function_name.replace("{{closure}}", "");

    // Format the final clean name
    if !module_path.is_empty() {
        format!("{}::{}", module_path, function_name)
    } else {
        function_name
    }
}
```

## A More Elegant Approach

If you want a cleaner solution, here's an alternative that uses pattern matching to detect and collapse these sequences:

```rust
use backtrace::Backtrace;
use rustc_demangle::demangle;
use std::collections::VecDeque;

fn main() {
    let mut current_backtrace = Backtrace::new_unresolved();
    current_backtrace.resolve();
    let mut is_within_target_range = false;

    // Store the processed stack trace
    let mut processed_stack: Vec<String> = Vec::new();

    // First, collect all relevant frames
    let mut raw_frames: Vec<String> = Vec::new();

    Backtrace::frames(&current_backtrace)
        .into_iter()
        .for_each(|frame| {
            for symbol in frame.symbols() {
                if let Some(name) = symbol.name() {
                    let name_str = name.to_string();

                    // Check if we've reached the start condition
                    if !is_within_target_range && name_str.contains("thag_rs::profiling::Profile::new") {
                        is_within_target_range = true;
                    }

                    // Collect frames within our target range
                    if is_within_target_range {
                        // Skip tokio::runtime functions
                        if name_str.starts_with("tokio::runtime") {
                            continue;
                        }

                        // Demangle the symbol
                        let demangled = demangle(&name_str).to_string();
                        raw_frames.push(demangled);

                        // Check if we've reached the end condition
                        if name_str.contains("std::sys::backtrace::__rust_begin_short_backtrace") {
                            is_within_target_range = false;
                            break;
                        }
                    }
                }
            }
        });

    // Process the collected frames to collapse patterns
    let mut i = 0;
    while i < raw_frames.len() {
        // Look for ProfiledFuture pattern (3 consecutive frames)
        if i + 2 < raw_frames.len() &&
           raw_frames[i].contains("{{closure}}::h") &&
           raw_frames[i+1].contains("ProfiledFuture") && raw_frames[i+1].contains("::poll::") &&
           raw_frames[i+2].contains("{{closure}}::{{closure}}") {

            // Extract the original function name from the first frame
            let clean_name = extract_function_name(&raw_frames[i]);
            processed_stack.push(clean_name);

            // Skip the next 2 frames
            i += 3;
        } else {
            // Regular frame - extract clean function name
            let clean_name = extract_function_name(&raw_frames[i]);
            processed_stack.push(clean_name);
            i += 1;
        }
    }

    // Print the processed stack trace
    for func_name in processed_stack {
        eprintln!("function={}", func_name);
    }
}

fn extract_function_name(demangled: &str) -> String {
    // Same function as before
    // ...
}
```

This approach:

1. First collects all relevant frames into a vector
2. Then processes this vector, looking for the specific pattern of 3 consecutive frames that match your ProfiledFuture pattern
3. When it finds this pattern, it collapses it into a single entry using the function name from the original function call
4. For all other frames, it processes them normally

This should give you a much cleaner stack trace that shows the actual function calls without the async machinery noise.


I've tried the second of these and it works to the extent of removing the pattern, but the `extract_function_name` does not work, nor does the old regex version. So the stack looks like this:
```
function=thag_rs::profiling::Profile::new::h3a13ddff9cb8ccd8
function=thag_config_builder_proto_profile::prompt_config::{{closure}}::h954ab6e59b8573d4
function=thag_config_builder_proto_profile::main::{{closure}}::{{closure}}::h2311dbcd3264cebe
function=thag_config_builder_proto_profile::main::{{closure}}::hde7be20a08e1931e
function=thag_config_builder_proto_profile::main::hc6650931edd254f2
function=core::ops::function::FnOnce::call_once::hb1eb7da98a867d4b
function=std::sys::backtrace::__rust_begin_short_backtrace::h561e8abad309410c
```
It also highlights a similar pattern with `async fn main`. which does not have or need a #[profile] attribute, so doesn't get the ProfiledFuture scaffolding. I don't need or want the `main` entries with closures, only the last of the 3. I also don't want the FnOnce::call_once or the backtrace::__rust_begin_short_backtrace, which for me is just a marker for the start of the backtrace.

# Improved Stack Frame Processing

I understand the issues you're encountering. Let's fix the `extract_function_name` function and improve the pattern matching to handle both the ProfiledFuture scaffolding and the async main pattern.

Here's a completely revised implementation:

```rust
use backtrace::Backtrace;
use rustc_demangle::demangle;

fn main() {
    let mut current_backtrace = Backtrace::new_unresolved();
    current_backtrace.resolve();
    let mut is_within_target_range = false;

    // Store the processed stack trace
    let mut raw_frames: Vec<String> = Vec::new();

    // First, collect all relevant frames
    Backtrace::frames(&current_backtrace)
        .into_iter()
        .for_each(|frame| {
            for symbol in frame.symbols() {
                if let Some(name) = symbol.name() {
                    let name_str = name.to_string();

                    // Check if we've reached the start condition
                    if !is_within_target_range && name_str.contains("thag_rs::profiling::Profile::new") {
                        is_within_target_range = true;
                    }

                    // Collect frames within our target range
                    if is_within_target_range {
                        // Skip tokio::runtime functions
                        if name_str.starts_with("tokio::runtime") {
                            continue;
                        }

                        // Demangle the symbol
                        let demangled = demangle(&name_str).to_string();
                        raw_frames.push(demangled);

                        // Check if we've reached the end condition
                        if name_str.contains("std::sys::backtrace::__rust_begin_short_backtrace") {
                            is_within_target_range = false;
                            break;
                        }
                    }
                }
            }
        });

    // Process the collected frames to collapse patterns and clean up
    let cleaned_stack = clean_stack_trace(raw_frames);

    // Print the processed stack trace
    for func_name in cleaned_stack {
        eprintln!("function={}", func_name);
    }
}

fn clean_stack_trace(raw_frames: Vec<String>) -> Vec<String> {
    let mut cleaned_frames = Vec::new();
    let mut skip_until_index = 0;

    // Remove the standard library functions we don't care about
    let filtered_frames: Vec<String> = raw_frames.into_iter()
        .filter(|frame|
            !frame.contains("core::ops::function::FnOnce::call_once") &&
            !frame.contains("std::sys::backtrace::__rust_begin_short_backtrace")
        )
        .collect();

    // Now process the filtered frames
    let mut i = 0;
    while i < filtered_frames.len() {
        if i < skip_until_index {
            i += 1;
            continue;
        }

        // Extract clean name from the current frame
        let clean_name = clean_function_name(&filtered_frames[i]);

        // Look for ProfiledFuture pattern (sequence of 3 frames)
        if i + 2 < filtered_frames.len() {
            let curr = &filtered_frames[i];
            let next1 = &filtered_frames[i+1];
            let next2 = &filtered_frames[i+2];

            if curr.contains("{{closure}}::h") &&
               next1.contains("ProfiledFuture") && next1.contains("::poll::") &&
               next2.contains("{{closure}}::{{closure}}") {

                // Add only the clean name from the first frame
                cleaned_frames.push(clean_name);
                skip_until_index = i + 3;
                i += 1;
                continue;
            }
        }

        // Look for async main pattern (sequence of closures)
        if filtered_frames[i].contains("main::{{closure}}") {
            // Find the last main closure in sequence
            let mut last_main_idx = i;
            for j in i..filtered_frames.len() {
                if filtered_frames[j].contains("main::") &&
                   (filtered_frames[j].contains("{{closure}}") ||
                    !filtered_frames[j].contains("::h")) {
                    last_main_idx = j;
                } else {
                    break;
                }
            }

            // Only add the main function without closures
            cleaned_frames.push("main".to_string());
            skip_until_index = last_main_idx + 1;
            i += 1;
            continue;
        }

        // Regular frame
        cleaned_frames.push(clean_name);
        i += 1;
    }

    cleaned_frames
}

fn clean_function_name(demangled: &str) -> String {
    // Split by module path and function name
    let parts: Vec<&str> = demangled.split("::").collect();

    // Extract module path (everything before function name)
    let mut module_parts = Vec::new();
    let mut function_name = String::new();

    for (i, part) in parts.iter().enumerate() {
        // Skip empty parts
        if part.is_empty() {
            continue;
        }

        // Check if this is a function name (contains hash marker or is last part)
        if part.contains('h') && part.len() > 10 && i == parts.len() - 1 {
            // This is a hash suffix, get the function name from previous part
            if i > 0 {
                function_name = parts[i-1].to_string();
                // Remove the last module part as it's actually the function name
                if !module_parts.is_empty() {
                    module_parts.pop();
                }
            }
            break;
        } else if i == parts.len() - 1 {
            // Last part with no hash is the function name
            function_name = part.to_string();
        } else {
            // This is part of the module path
            module_parts.push(*part);
        }
    }

    // Clean up function name - remove closure markers and hash
    function_name = function_name.replace("{{closure}}", "");

    if let Some(idx) = function_name.find("::h") {
        function_name = function_name[..idx].to_string();
    }

    // Join module path with function name
    let module_path = module_parts.join("::");

    if module_path.is_empty() {
        function_name
    } else if function_name.is_empty() {
        module_path
    } else {
        format!("{}::{}", module_path, function_name)
    }
}
```

This implementation:

1. Filters out unwanted standard library functions (`FnOnce::call_once` and `__rust_begin_short_backtrace`)
2. Handles the ProfiledFuture scaffolding pattern as before
3. Detects sequences of `main::{{closure}}` patterns and collapses them into just `main`
4. Uses a completely rewritten `clean_function_name` function that should better handle the hash extraction

For example, this should turn:
```
function=thag_rs::profiling::Profile::new::h3a13ddff9cb8ccd8
function=thag_config_builder_proto_profile::prompt_config::{{closure}}::h954ab6e59b8573d4
function=thag_config_builder_proto_profile::main::{{closure}}::{{closure}}::h2311dbcd3264cebe
function=thag_config_builder_proto_profile::main::{{closure}}::hde7be20a08e1931e
function=thag_config_builder_proto_profile::main::hc6650931edd254f2
function=core::ops::function::FnOnce::call_once::hb1eb7da98a867d4b
function=std::sys::backtrace::__rust_begin_short_backtrace::h561e8abad309410c
```

Into something like:
```
function=thag_rs::profiling::Profile::new
function=thag_config_builder_proto_profile::prompt_config
function=main
```

If you're still having trouble with the function name extraction, let me know, and I can provide a simpler but more robust version.

Thanks, it's partly successful. Unfortunately, the ProfiledFuture scaffolding pattern is back and fn `main`. appears in 2 guises:
```
function=thag_rs::profiling::Profile::new
function=thag_config_builder_proto_profile::prompt_logging_config_with_escape
function=thag_config_builder_proto_profile::prompt_config::{{closure}}
function=<thag_config_builder_proto_profile::prompt_config::{{closure}}::ProfiledFuture<F> as core::future::future::Future>::poll
function=thag_config_builder_proto_profile::prompt_config
function=main
function=thag_config_builder_proto_profile::main
```
The plot seems to be thickening. Here's what displays as I work through the profiled code:
```
Welcome to thag config generator!
function=thag_rs::profiling::Profile::new
function=thag_config_builder_proto_profile::prompt_config
function=main
function=thag_config_builder_proto_profile::main
> Configure: Logging
function=thag_rs::profiling::Profile::new
function=thag_config_builder_proto_profile::prompt_logging_config_with_escape
function=thag_config_builder_proto_profile::prompt_config::{{closure}}
function=<thag_config_builder_proto_profile::prompt_config::{{closure}}::ProfiledFuture<F> as core::future::future::Future>::poll
function=thag_config_builder_proto_profile::prompt_config
function=main
function=thag_config_builder_proto_profile::main
> Default verbosity level: error
> Configure: Dependencies
function=thag_rs::profiling::Profile::new
function=thag_config_builder_proto_profile::prompt_dependency_config
function=thag_config_builder_proto_profile::prompt_config::{{closure}}
function=<thag_config_builder_proto_profile::prompt_config::{{closure}}::ProfiledFuture<F> as core::future::future::Future>::poll
function=thag_config_builder_proto_profile::prompt_config
function=main
function=thag_config_builder_proto_profile::main
> Exclude unstable features? Yes
> Configure always-included features? No
> Configure globally excluded features? No
> Add crate-specific feature overrides? No
> Configure: Colors
function=thag_rs::profiling::Profile::new
function=thag_config_builder_proto_profile::prompt_colors_config_with_escape
function=thag_config_builder_proto_profile::prompt_config::{{closure}}
function=<thag_config_builder_proto_profile::prompt_config::{{closure}}::ProfiledFuture<F> as core::future::future::Future>::poll
function=thag_config_builder_proto_profile::prompt_config
function=main
function=thag_config_builder_proto_profile::main
> Color support: auto
...
```
The async functions here are: main, prompt_config and prompt_dependency_config. The sync functions are prompt_logging_config_with_escape and prompt_colors_config_with_escape.
So I can't understand what I'm seeing here. I've since added displays that confirm that sync functions like prompt_logging_config_with_escape and prompt_colors_config_with_escape are correctly invoked from generate_sync_wrapper, while async functions like main, prompt_config and prompt_dependency_config are correctly invoked from generate_async_wrapper. So why doesn't the first appearance of prompt_config have the ProfiledFuture scaffolding, and why do subsequent appearances of prompt_config have it?

# Debugging the Backtrace Pattern Issue

Based on the output you're seeing, there are some complex interactions happening in your code's execution paths. Let me help you understand what might be going on and provide a more robust solution.

## Understanding the Pattern

From your output, I notice:

1. The first call to `prompt_config` doesn't show the ProfiledFuture pattern, but subsequent calls do
2. We have both `main` and `thag_config_builder_proto_profile::main` appearing
3. There's a mix of sync and async functions being profiled

This suggests:

- The first call to `prompt_config` might be through a different execution path than subsequent calls
- The backtrace structure might be different depending on where/how the function is called
- We need a more robust approach to filter out the scaffolding regardless of the pattern

## Enhanced Solution

Here's a more comprehensive solution that should handle the various patterns better:

```rust
use backtrace::Backtrace;
use rustc_demangle::demangle;
use std::collections::HashSet;

fn main() {
    let mut current_backtrace = Backtrace::new_unresolved();
    current_backtrace.resolve();
    let mut is_within_target_range = false;

    // Store the processed stack trace
    let mut raw_frames: Vec<String> = Vec::new();

    // First, collect all relevant frames
    Backtrace::frames(&current_backtrace)
        .into_iter()
        .for_each(|frame| {
            for symbol in frame.symbols() {
                if let Some(name) = symbol.name() {
                    let name_str = name.to_string();

                    // Check if we've reached the start condition
                    if !is_within_target_range && name_str.contains("thag_rs::profiling::Profile::new") {
                        is_within_target_range = true;
                    }

                    // Collect frames within our target range
                    if is_within_target_range {
                        // Skip tokio::runtime functions
                        if name_str.starts_with("tokio::runtime") {
                            continue;
                        }

                        // Demangle the symbol
                        let demangled = demangle(&name_str).to_string();
                        raw_frames.push(demangled);

                        // Check if we've reached the end condition
                        if name_str.contains("std::sys::backtrace::__rust_begin_short_backtrace") {
                            is_within_target_range = false;
                            break;
                        }
                    }
                }
            }
        });

    // Clean up the stack trace
    let cleaned_frames = clean_stack_trace(raw_frames);

    // Print the processed stack trace
    for func_name in cleaned_frames {
        eprintln!("function={}", func_name);
    }
}

fn clean_stack_trace(raw_frames: Vec<String>) -> Vec<String> {
    // First, filter out standard library infrastructure we don't care about
    let filtered_frames: Vec<String> = raw_frames.into_iter()
        .filter(|frame|
            !frame.contains("core::ops::function::FnOnce::call_once") &&
            !frame.contains("std::sys::backtrace::__rust_begin_short_backtrace") &&
            !frame.contains("std::rt::lang_start") &&
            !frame.contains("std::panicking")
        )
        .collect();

    // These are patterns we want to remove from the stack
    let scaffolding_patterns: Vec<&str> = vec![
        "ProfiledFuture",
        "{{closure}}::{{closure}}",
        "<F as core::future::future::Future>::poll",
        "::poll::",
    ];

    // Create a new cleaned stack, filtering out scaffolding
    let mut cleaned_frames = Vec::new();
    let mut i = 0;
    let mut already_seen = HashSet::new();
    let mut seen_main = false;

    while i < filtered_frames.len() {
        let current_frame = &filtered_frames[i];

        // Check if this is scaffolding we want to skip
        let is_scaffolding = scaffolding_patterns.iter()
            .any(|pattern| current_frame.contains(pattern));

        if is_scaffolding {
            i += 1;
            continue;
        }

        // Clean the function name
        let mut clean_name = clean_function_name(current_frame);

        // Handle main function special case
        if clean_name.ends_with("::main") || clean_name == "main" {
            if !seen_main {
                cleaned_frames.push("main".to_string());
                seen_main = true;
            }
            i += 1;
            continue;
        }

        // Skip duplicate function calls (helps with the {{closure}} pattern)
        if already_seen.contains(&clean_name) {
            i += 1;
            continue;
        }

        already_seen.insert(clean_name.clone());
        cleaned_frames.push(clean_name);
        i += 1;
    }

    cleaned_frames
}

fn clean_function_name(demangled: &str) -> String {
    // Remove hash suffixes and closure markers
    let mut clean_name = demangled.to_string();

    // Find and remove hash suffixes (::h followed by hex digits)
    if let Some(hash_pos) = clean_name.find("::h") {
        if clean_name[hash_pos+3..].chars().all(|c| c.is_digit(16)) {
            clean_name = clean_name[..hash_pos].to_string();
        }
    }

    // Remove closure markers
    clean_name = clean_name.replace("{{closure}}", "");

    // Clean up any double colons that might be left
    while clean_name.contains("::::") {
        clean_name = clean_name.replace("::::", "::");
    }
    if clean_name.ends_with("::") {
        clean_name = clean_name[..clean_name.len()-2].to_string();
    }

    clean_name
}
```

## What This Approach Does:

1. **Broader Filtering**: Removes all forms of scaffolding based on specific patterns rather than sequence recognition
2. **Duplicate Removal**: Keeps track of functions we've already seen to avoid duplicates
3. **Main Simplification**: Simplifies all variations of "main" function to just "main"
4. **Better Name Cleaning**: Improved function name cleaning that handles hash suffixes and closure markers more robustly

This approach should produce a much cleaner stack trace regardless of the execution path. The expected output would be something like:

```
function=thag_rs::profiling::Profile::new
function=thag_config_builder_proto_profile::prompt_logging_config_with_escape
function=thag_config_builder_proto_profile::prompt_config
function=main
```

If you're still seeing scaffolding frames, you might need to add more specific patterns to the `scaffolding_patterns` list.


Thanks, all going well. My next challenge is to see how we might profile sections of code. We have declarative macro profile_section! and fn end_profile_section, but the latter uses the redundant PROFILE_STACK which I'm getting rid of. Can an attribute macro or a proc macro in general be used to replace it? One issue is that I don't want to have to enclose the section in a block because it's often not possible or desirable to isolate it from the surrounding code.


# Profiling Code Sections without Using Blocks

You're right that replacing `profile_section!` and `end_profile_section` with a proc macro is challenging if you don't want to enclose the section in a block. Let me explain some approaches and their tradeoffs.

## Option 1: Improved Declarative Macro

The simplest solution would be to keep using a declarative macro approach but improve it to avoid the global `PROFILE_STACK`:

```rust
#[macro_export]
macro_rules! profile_section {
    ($name:expr) => {
        let _section_profiler = $crate::profiling::SectionProfiler::new($name);
    };
}
```

And define `SectionProfiler` as:

```rust
pub struct SectionProfiler {
    name: String,
    start_time: Instant,
}

impl SectionProfiler {
    pub fn new(name: &str) -> Self {
        eprintln!("Starting section: {}", name);
        Self {
            name: name.to_string(),
            start_time: Instant::now(),
        }
    }
}

impl Drop for SectionProfiler {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        eprintln!("Section {} took {:?}", self.name, duration);
    }
}
```

This uses RAII (Resource Acquisition Is Initialization) to automatically clean up when the variable goes out of scope, which would happen at the end of the current block.

## Option 2: Proc Macro with RAII-based API

You could create a procedural macro that transforms code like:

```rust
#[profile_section("my section")]
let result = expensive_operation();
```

Into:

```rust
let _section_profiler = SectionProfiler::new("my section");
let result = expensive_operation();
```

The key is that the `_section_profiler` variable will be dropped at the end of the current scope.

## Option 3: Span-based Profiling

For more flexibility, you could implement a span-based profiling approach:

```rust
pub fn start_section(name: &str) -> SectionId {
    let id = generate_unique_id();
    eprintln!("Starting section {} with id {}", name, id);
    // Record start time for id
    id
}

pub fn end_section(id: SectionId) {
    // Calculate elapsed time using stored start time
    eprintln!("Section with id {} ended", id);
}

// Usage:
let section_id = start_section("my section");
// Code to profile
end_section(section_id);
```

## Option 4: Context Manager Approach

For the cleanest API, you could adopt a context manager pattern similar to Python's `with` statement:

```rust
fn my_function() {
    // Normal code

    profile_section!("expensive part").run(|| {
        // Code to profile
        expensive_operation();
    });

    // More normal code
}
```

Implementation:

```rust
#[macro_export]
macro_rules! profile_section {
    ($name:expr) => {
        $crate::profiling::SectionProfiler::new($name)
    };
}

pub struct SectionProfiler {
    name: String,
}

impl SectionProfiler {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub fn run<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        eprintln!("Starting section: {}", self.name);

        let result = f();

        let duration = start.elapsed();
        eprintln!("Section {} took {:?}", self.name, duration);

        result
    }
}
```

## Best Approach: Combine Options 1 and 3

For your specific use case, I'd recommend a combination of Options 1 and 3:

```rust
#[macro_export]
macro_rules! profile_section {
    ($name:expr) => {
        let _section_profiler = $crate::profiling::SectionProfiler::begin($name);
    };
}

thread_local! {
    static CURRENT_SECTION: RefCell<Option<SectionProfiler>> = RefCell::new(None);
}

pub struct SectionProfiler {
    name: String,
    start_time: Instant,
    parent: Option<Box<SectionProfiler>>,
}

impl SectionProfiler {
    pub fn begin(name: &str) -> SectionGuard {
        let profiler = Self {
            name: name.to_string(),
            start_time: Instant::now(),
            parent: CURRENT_SECTION.with(|current| current.borrow_mut().take().map(Box::new)),
        };

        let guard = SectionGuard;

        CURRENT_SECTION.with(|current| {
            *current.borrow_mut() = Some(profiler);
        });

        guard
    }

    fn end(self) {
        let duration = self.start_time.elapsed();
        eprintln!("Section {} took {:?}", self.name, duration);

        // Restore parent section if any
        if let Some(parent) = self.parent {
            CURRENT_SECTION.with(|current| {
                *current.borrow_mut() = Some(*parent);
            });
        }
    }
}

pub struct SectionGuard;

impl Drop for SectionGuard {
    fn drop(&mut self) {
        CURRENT_SECTION.with(|current| {
            if let Some(profiler) = current.borrow_mut().take() {
                profiler.end();
            }
        });
    }
}
```

This approach allows you to:

1. Simply write `profile_section!("my section")` at the start of a section
2. Let the scope handle the end of the section automatically
3. Support nested sections with proper parent restoration
4. Avoid global mutable state with thread-local storage

Is this the type of solution you're looking for, or would you prefer a different approach? I can refine any of these options based on your specific needs.


Thanks for the ideas. Note that AFAIK the existing `profile_section` macro will still profile just fine from the point of invocation until the Profile variable it creates goes out of scope at the end of the function or block:
```
#[macro_export]
macro_rules! profile_section {
    ($name:expr) => {
        let _profile = $crate::profiling::Profile::new(
            $name,
            $crate::profiling::get_global_profile_type(),
            false,
            false,
        );
    };
}
```
What I really want is a way to end the profiling at the end of the section without waiting for the Profile to go out of scope. Otherwise each section encompasses all the following sections. After the discussion, it strikes me that surely all I need to do is get this macro to pass back a reference to the Profile instance so that I can manually drop the profile in some manner by calling the existing end_profile_section function as I've been doing using the redundant PROFILE_STACK. I just need new logic in fn end_profile_section to accepr the reference instead of the section name literal and to drop the profile.


# Using References to Profile Instances

You're absolutely right! The simplest approach is to have the `profile_section!` macro return a reference to the `Profile` instance, allowing you to manually end it whenever needed. Here's how you could implement this:

## 1. Updated `profile_section!` Macro

```rust
#[macro_export]
macro_rules! profile_section {
    ($name:expr) => {
        let profile = $crate::profiling::Profile::new(
            $name,
            $crate::profiling::get_global_profile_type(),
            false,
            false,
        );
        let profile_ref = &profile; // Create a reference
        profile_ref // Return the reference
    };
}
```

## 2. Updated `end_profile_section` Function

```rust
pub fn end_profile_section(profile: &Profile) {
    // Access the profile directly and end it
    unsafe {
        std::ptr::drop_in_place(profile as *const Profile as *mut Profile);
    }
}
```

## 3. Usage

```rust
fn my_function() {
    // Start a section
    let section_profile = profile_section!("expensive part");

    // Do some work
    expensive_operation();

    // Explicitly end the section
    end_profile_section(section_profile);

    // More code here that won't be included in the profile
    other_operations();
}
```

## Important Safety Considerations

The approach above uses `drop_in_place`, which is unsafe. There are several issues to be aware of:

1. **Double Drop**: After calling `end_profile_section`, the variable is still in scope, so Rust will try to drop it again when it goes out of scope.

2. **Use After Free**: Any further access to the profile would be a use-after-free bug.

## Safer Alternatives

### Option 1: Using ManuallyDrop

A safer approach is to use `ManuallyDrop`:

```rust
use std::mem::ManuallyDrop;

#[macro_export]
macro_rules! profile_section {
    ($name:expr) => {
        let profile = ManuallyDrop::new($crate::profiling::Profile::new(
            $name,
            $crate::profiling::get_global_profile_type(),
            false,
            false,
        ));
        &profile
    };
}

pub fn end_profile_section(profile: &ManuallyDrop<Profile>) {
    // Cast to mutable reference (safe because we're the sole controller of this instance)
    let profile_mut = unsafe { &mut *(profile as *const ManuallyDrop<Profile> as *mut ManuallyDrop<Profile>) };
    // Drop the profile safely
    unsafe { ManuallyDrop::drop(profile_mut) };
}
```

### Option 2: Using a Closure-Based API

An even safer approach is to redesign the API to use a closure:

```rust
#[macro_export]
macro_rules! profile_section {
    ($name:expr) => {
        $crate::profiling::ProfileSection::new($name)
    };
}

pub struct ProfileSection {
    profile: Profile,
}

impl ProfileSection {
    pub fn new(name: &str) -> Self {
        Self {
            profile: Profile::new(name, get_global_profile_type(), false, false),
        }
    }

    pub fn end(self) {
        // Profile is dropped here
    }
}

// Usage:
fn my_function() {
    let section = profile_section!("expensive part");

    // Do some work
    expensive_operation();

    // End the section
    section.end();

    // More code that won't be profiled
}
```

### Option 3: Using an Explicit State Flag

Add a state flag to your `Profile` struct:

```rust
pub struct Profile {
    // existing fields
    active: bool,
}

impl Profile {
    pub fn new(...) -> Self {
        Self {
            // existing initialization
            active: true,
        }
    }

    pub fn end(&mut self) {
        if self.active {
            // End profiling logic
            self.active = false;
        }
    }
}

impl Drop for Profile {
    fn drop(&mut self) {
        if self.active {
            // End profiling logic
        }
    }
}

#[macro_export]
macro_rules! profile_section {
    ($name:expr) => {
        let mut profile = $crate::profiling::Profile::new(
            $name,
            $crate::profiling::get_global_profile_type(),
            false,
            false,
        );
        &mut profile
    };
}

pub fn end_profile_section(profile: &mut Profile) {
    profile.end();
}
```

## Recommendation

I recommend Option 3 as the safest and most idiomatic approach. It:

1. Doesn't require unsafe code in the public API
2. Prevents double-accounting by using the `active` flag
3. Maintains the current API shape with minimal changes
4. Is clear and easy to understand

This approach allows you to profile sections of code without forcing blocks and without using thread-local storage or other complex patterns.


Would the closure-based API in option 2 use drop_in_place or ManuallyDrop? And would the closure limit the scope of the profile section? I don't want the profiling logic to interfere with the logic of the code being profiled. I'm definitely open to option 3 but I just want to understand the implications of each option.


# Clarifying Option 2: Closure-Based API

Let me clarify how Option 2 (the closure-based API) works:

## Option 2 Details

The closure-based API in Option 2 **doesn't use `drop_in_place` or `ManuallyDrop`** at all. It relies on Rust's ownership system in a fully safe way.

Here's how it works:

```rust
pub struct ProfileSection {
    profile: Profile,
}

impl ProfileSection {
    pub fn new(name: &str) -> Self {
        Self {
            profile: Profile::new(name, get_global_profile_type(), false, false),
        }
    }

    pub fn end(self) {
        // When self is consumed, profile is dropped naturally here
        // No unsafe code needed
    }
}
```

When you call `section.end()`, the `ProfileSection` is consumed (note the `self` rather than `&self` parameter), which means:

1. The `ProfileSection` instance is moved into the `end` method
2. At the end of the `end` method, the `ProfileSection` goes out of scope
3. Rust automatically drops the `profile` field, ending the profiling
4. No manual dropping or unsafe code is required

### Usage Example

```rust
fn my_function() {
    let section = profile_section!("expensive part");

    // Do expensive work
    expensive_operation();

    // End the section - this consumes the section variable
    section.end();

    // More code that won't be profiled
    more_operations();

    // You can't accidentally use section again - it's been consumed
    // This would be a compile error:
    // section.end();  // Error: use of moved value
}
```

### Implications

1. **Safety**: This approach is 100% safe - no unsafe code required
2. **Usability**: Clear API that prevents accidental re-use
3. **Scope Independence**: The profiling doesn't depend on code blocks or scopes - you explicitly control when it starts and ends
4. **Non-Interference**: The profiling logic doesn't interfere with the code being profiled
5. **Compile-Time Checks**: Rust's ownership system ensures you can't accidentally end the same section twice

## Comparing with Option 3

Option 3 (using an explicit state flag) achieves similar goals but works differently:

```rust
pub fn end_profile_section(profile: &mut Profile) {
    profile.end();
}
```

### Key Differences

1. **Mutable References**: Option 3 uses `&mut` references instead of consuming the value
2. **Multiple Calls**: Option 3 requires a state flag to prevent logic from running twice if `end` is called multiple times
3. **Safety Level**: Both are safe, but Option 2 leverages Rust's type system more strongly

## Recommendation

I still think Option 3 is probably the most practical for your specific case since:

1. It maintains compatibility with your existing API shape
2. It clearly separates the "start" and "end" concepts
3. It's intuitive for users of your library

However, Option 2 is slightly more "Rusty" and provides stronger compile-time guarantees. Either would be a good choice.

Which approach would you prefer to implement? I can provide more detailed code for either one.

I'm not yet convinced by the reasoning behind preferring Option 3 over Option 2. I don't think you've shown a usage example of option 3 to compare, could you provide one?

# Comparing Option 2 vs Option 3: Usage Examples

You're absolutely right that I should provide complete usage examples for a better comparison. Let's look at both approaches side by side:

## Option 2: Closure-Based API (Consuming the value)

### Implementation

```rust
pub struct ProfileSection {
    profile: Profile,
}

impl ProfileSection {
    pub fn new(name: &str) -> Self {
        Self {
            profile: Profile::new(name, get_global_profile_type(), false, false),
        }
    }

    pub fn end(self) {
        // Profile is dropped here automatically
    }
}

#[macro_export]
macro_rules! profile_section {
    ($name:expr) => {
        $crate::profiling::ProfileSection::new($name)
    };
}
```

### Usage Example

```rust
fn complicated_function() {
    // Some initial code
    prepare_data();

    // Start profiling a section
    let section = profile_section!("expensive part");

    // Do the expensive work we want to profile
    expensive_operation_1();
    expensive_operation_2();

    // End the section explicitly - consumes the section variable
    section.end();

    // More code that won't be profiled
    finalize_data();

    // If we tried to use section again, we'd get a compile error:
    // section.end();  // Error: use of moved value
}
```

## Option 3: State Flag API (Mutable reference)

### Implementation

```rust
pub struct Profile {
    // existing fields
    name: String,
    // ... other fields
    active: bool,
}

impl Profile {
    pub fn new(name: &str, profile_type: ProfileType, log_to_file: bool, log_to_stdout: bool) -> Self {
        Self {
            name: name.to_string(),
            // ... initialize other fields
            active: true,
        }
    }

    pub fn end(&mut self) {
        if self.active {
            // End profiling logic here
            println!("Ending profile section: {}", self.name);
            self.active = false;
        }
    }
}

impl Drop for Profile {
    fn drop(&mut self) {
        if self.active {
            // Same end profiling logic
            println!("Dropping profile section: {}", self.name);
        }
    }
}

#[macro_export]
macro_rules! profile_section {
    ($name:expr) => {
        let mut profile = $crate::profiling::Profile::new(
            $name,
            $crate::profiling::get_global_profile_type(),
            false,
            false,
        );
        &mut profile
    };
}

pub fn end_profile_section(profile: &mut Profile) {
    profile.end();
}
```

### Usage Example

```rust
fn complicated_function() {
    // Some initial code
    prepare_data();

    // Start profiling a section
    let mut section = profile_section!("expensive part");

    // Do the expensive work we want to profile
    expensive_operation_1();
    expensive_operation_2();

    // End the section explicitly
    end_profile_section(&mut section);

    // More code that won't be profiled
    finalize_data();

    // If we accidentally try to end it again, it's safe (no-op):
    end_profile_section(&mut section);  // Won't do anything because active is false
}
```

## Direct Comparison

Now that we have both usage examples, let's compare them directly:

### Option 2 (Consuming the value)
- **Pros**:
  - Uses Rust's ownership system to prevent accidental double-ending
  - Cleaner API with method call syntax (`section.end()`)
  - No need for mutable references
  - No state flag needed internally
- **Cons**:
  - Introduces a new wrapper type
  - Slightly deviates from your current API pattern

### Option 3 (Mutable reference)
- **Pros**:
  - More similar to your existing API
  - Flexible - can be ended multiple times safely
  - Maintains the existing function-based approach
- **Cons**:
  - Requires mutable references
  - Needs internal state tracking
  - Doesn't leverage Rust's ownership system for safety

## Recommendation Revisited

Given this direct comparison, I now think **Option 2** is actually the better choice for most scenarios because:

1. It provides stronger compile-time guarantees
2. It has a cleaner, more Rust-idiomatic API
3. It doesn't require internal state tracking
4. The method call syntax is more discoverable

However, if maintaining close compatibility with your existing API is a priority, or if you need the flexibility to call `end_profile_section` multiple times, Option 3 would be better.

Which aspects are most important for your use case?


Yes, my gut instinct also favours Option 2. What would happen in option 2 if we accidentally try to end it again?

# What Happens If You Try to End a Section Twice in Option 2

In Option 2 (the consuming value approach), if you try to end the same profile section twice, you'll get a **compile-time error**, which is one of its biggest advantages!

## Example Showing the Compile Error

```rust
fn complicated_function() {
    // Start profiling a section
    let section = profile_section!("expensive part");

    // Do the expensive work we want to profile
    expensive_operation_1();

    // End the section - this consumes the section variable
    section.end();

    // If we try to use section again:
    section.end();  // COMPILE ERROR: use of moved value: `section`

    // The rest of the function
    finalize_data();
}
```

## Why This Happens

This happens because:

1. The `end()` method takes `self` (not `&self` or `&mut self`), which means it takes ownership of the `ProfileSection` instance
2. After calling `section.end()`, the variable `section` is no longer valid because its value has been moved
3. Rust's ownership system detects this at compile time and prevents you from using `section` again

## Benefits of This Approach

This is a significant advantage of Option 2 because:

1. **Compile-Time Safety**: Errors are caught during compilation, not at runtime
2. **No Runtime Overhead**: No need for runtime checks since impossible states are eliminated by the type system
3. **Self-Documenting Code**: Makes it clear that once a section is ended, it cannot be used again
4. **Prevention of Logic Bugs**: Eliminates an entire class of potential bugs before they can happen

## What If You Need Multiple Ends?

If for some reason you actually need the ability to potentially end a section multiple times (perhaps in different code paths), you could adapt Option 2 to use an `Option`:

```rust
pub struct ProfileSection {
    profile: Option<Profile>,
}

impl ProfileSection {
    pub fn new(name: &str) -> Self {
        Self {
            profile: Some(Profile::new(name, get_global_profile_type(), false, false)),
        }
    }

    pub fn end(&mut self) -> bool {
        if let Some(profile) = self.profile.take() {
            // Profile is dropped here
            true
        } else {
            // Already ended
            false
        }
    }

    // You could also add a convenience method to check if it's still active
    pub fn is_active(&self) -> bool {
        self.profile.is_some()
    }
}
```

This would allow multiple calls to `end()` without errors, but would only actually end the profiling on the first call. It would still be safe and idiomatic, while giving you the flexibility of Option 3.

Given these details, does Option 2 still align with your needs?

Yes it does. Let's go with Option 2 then.

# Implementing Option 2: Profile Section with Ownership-Based End

Great choice! Here's a complete implementation of Option 2, which uses Rust's ownership system to provide a clean and safe API for profiling sections:

```rust
// In your profiling.rs file:

// The ProfileSection struct that wraps Profile
pub struct ProfileSection {
    profile: Profile,
}

impl ProfileSection {
    // Create a new profile section
    pub fn new(name: &str) -> Self {
        Self {
            profile: Profile::new(
                name,
                get_global_profile_type(),
                false, // log_to_file
                false, // log_to_stdout
            ),
        }
    }

    // End the profile section by consuming self
    pub fn end(self) {
        // Profile is automatically dropped here
        // No additional code needed - we're relying on RAII
    }
}

// The macro that creates a ProfileSection
#[macro_export]
macro_rules! profile_section {
    ($name:expr) => {
        $crate::profiling::ProfileSection::new($name)
    };
}
```

## Usage Examples

Here are some usage examples to document in your code:

### Basic Usage

```rust
fn my_function() {
    // Regular code here

    // Start a profiled section
    let section = profile_section!("expensive part");

    // Code that will be profiled
    expensive_operation();

    // End the profiled section
    section.end();

    // More code that won't be profiled
    cleanup();
}
```

### With Conditional Logic

```rust
fn process_data(data: &[u32]) {
    let section = profile_section!("data processing");

    if data.is_empty() {
        // End early if there's no data
        section.end();
        return;
    }

    // Process the data
    for item in data {
        process_item(item);
    }

    // End the profiling
    section.end();
}
```

### Error Handling Example

```rust
fn fallible_operation() -> Result<(), Error> {
    let section = profile_section!("fallible op");

    // Try the operation
    let result = match do_something_risky() {
        Ok(value) => value,
        Err(e) => {
            // End profiling before returning the error
            section.end();
            return Err(e);
        }
    };

    // Continue with more operations
    do_more_stuff(result)?;

    // End profiling before returning success
    section.end();
    Ok(())
}
```

## Implementation Notes

1. This approach doesn't interfere with the code being profiled - it simply measures the time between creation and ending.

2. The `end()` method consumes `self`, ensuring that a profile section can only be ended once.

3. You don't need to add anything to your `Drop` implementation for `Profile` - it will work as is.

4. If you want to add more functionality, like pausing/resuming profiling, you can add more methods to `ProfileSection`.

Let me know if you need any adjustments to this implementation!


Thanks, there's an issue in that ProfileSection::new expects Profile::new to return a Profile instance, but it actually returns it wrapped in an Option, and returns None if profiling is not enabled. Perhaps ProfileSection should do something similar?


# Handling Optional Profiling in ProfileSection

You're absolutely right! If `Profile::new` returns an `Option<Profile>` to handle cases where profiling is disabled, we should adapt `ProfileSection` to handle this as well. Here's how to modify the implementation:

```rust
// In your profiling.rs file:

// The ProfileSection struct that wraps an optional Profile
pub struct ProfileSection {
    profile: Option<Profile>,
}

impl ProfileSection {
    // Create a new profile section, handling the case where profiling is disabled
    pub fn new(name: &str) -> Self {
        Self {
            profile: Profile::new(
                name,
                get_global_profile_type(),
                false, // log_to_file
                false, // log_to_stdout
            ),
        }
    }

    // End the profile section by consuming self
    pub fn end(self) {
        // The Option<Profile> will be dropped here, which will drop
        // the Profile if it exists
    }

    // Optional: Add a method to check if profiling is active
    pub fn is_active(&self) -> bool {
        self.profile.is_some()
    }
}

// The macro that creates a ProfileSection
#[macro_export]
macro_rules! profile_section {
    ($name:expr) => {
        $crate::profiling::ProfileSection::new($name)
    };
}
```

## Usage Examples with Optional Profiling

Here's how this would work with the possibility of profiling being disabled:

### Basic Usage (Unchanged)

```rust
fn my_function() {
    // Start a profiled section
    let section = profile_section!("expensive part");

    // Code that will be profiled (if profiling is enabled)
    expensive_operation();

    // End the profiled section
    section.end();
}
```

### With Active Checking

```rust
fn process_data(data: &[u32]) {
    let section = profile_section!("data processing");

    // Optional: You can check if profiling is active
    if section.is_active() {
        println!("Profiling is enabled for this section");
    }

    // Process the data
    for item in data {
        process_item(item);
    }

    // End the profiling
    section.end();
}
```

## Advantages of This Approach

1. **Consistent API**: The API remains the same whether profiling is enabled or not

2. **Zero Overhead When Disabled**: If profiling is disabled, there's virtually no overhead since `profile` will be `None`

3. **Type Safety**: Still leverages Rust's ownership system for safety

4. **Optional Checking**: Provides the ability to check if profiling is active if needed

This approach gracefully handles both the profiling-enabled and profiling-disabled cases while maintaining the clean API we discussed earlier.

Does this adaptation address your concerns about handling the `Option<Profile>` return value?



error[E0521]: borrowed data escapes outside of associated function
   --> src/profiling.rs:609:22
    |
607 |       pub fn new(name: &str) -> Self {
    |                  ----  - let's call the lifetime of this reference `'1`
    |                  |
    |                  `name` is a reference that is only valid in the associated function body
608 |           Self {
609 |               profile: Profile::new(
    |  ______________________^
610 | |                 name,
611 | |                 get_global_profile_type(),
612 | |                 false, // log_to_file
613 | |                 false, // log_to_stdout
614 | |             ),
    | |             ^
    | |             |
    | |_____________`name` escapes the associated function body here
    |               argument requires that `'1` must outlive `'static`

# Fixing the Reference Lifetime Issue

You're encountering a lifetime issue because `Profile::new` expects a reference with a longer lifetime than what's being provided. Let's fix this by ensuring we're properly handling the string lifetime:

```rust
// In your profiling.rs file:

// The ProfileSection struct that wraps an optional Profile
pub struct ProfileSection {
    profile: Option<Profile>,
}

impl ProfileSection {
    // Create a new profile section, handling the case where profiling is disabled
    pub fn new(name: &str) -> Self {
        // Create a owned String from the reference to avoid lifetime issues
        let name_owned = name.to_string();

        Self {
            profile: Profile::new(
                &name_owned, // Use the owned string's reference
                get_global_profile_type(),
                false, // log_to_file
                false, // log_to_stdout
            ),
        }
    }

    // End the profile section by consuming self
    pub fn end(self) {
        // The Option<Profile> will be dropped here
    }

    // Optional: Add a method to check if profiling is active
    pub fn is_active(&self) -> bool {
        self.profile.is_some()
    }
}
```

This isn't quite right yet because we'd create an owned string but then the reference to it would be invalid when it goes out of scope. The proper solution depends on how `Profile::new` is implemented.

## Solution 1: If Profile::new Makes Its Own Copy of the String

If `Profile::new` copies the string internally (which it likely does), modify the method to directly pass the owned string:

```rust
pub fn new(name: &str) -> Self {
    Self {
        profile: Profile::new(
            &name.to_string(), // Create and pass an owned copy
            get_global_profile_type(),
            false,
            false,
        ),
    }
}
```

## Solution 2: If Profile Needs to Clone the Name Itself

If `Profile::new` needs a reference that outlives the function call, modify the method:

```rust
pub fn new<S: AsRef<str>>(name: S) -> Self {
    let name_ref = name.as_ref();
    Self {
        profile: Profile::new(
            name_ref,
            get_global_profile_type(),
            false,
            false,
        ),
    }
}
```

## Solution 3: Looking at Profile::new's Implementation

The best solution depends on how `Profile::new` is implemented. Let's assume it looks something like this:

```rust
impl Profile {
    pub fn new(name: &str, profile_type: ProfileType, log_to_file: bool, log_to_stdout: bool) -> Option<Self> {
        if !is_profiling_enabled() {
            return None;
        }

        Some(Self {
            name: name.to_string(), // It stores its own copy
            // other fields...
        })
    }
}
```

If this is the case, we can solve the issue by modifying `ProfileSection::new`:

```rust
pub fn new<S: Into<String>>(name: S) -> Self {
    let name_string = name.into();
    Self {
        profile: Profile::new(
            &name_string,
            get_global_profile_type(),
            false,
            false,
        ),
    }
}
```

## Best Solution

The most robust solution is to change how `Profile::new` accepts the name parameter. If you can modify `Profile::new`, consider changing it to:

```rust
impl Profile {
    pub fn new<S: Into<String>>(name: S, profile_type: ProfileType, log_to_file: bool, log_to_stdout: bool) -> Option<Self> {
        if !is_profiling_enabled() {
            return None;
        }

        Some(Self {
            name: name.into(), // Convert to String
            // other fields...
        })
    }
}
```

Then `ProfileSection::new` becomes simpler:

```rust
pub fn new<S: Into<String>>(name: S) -> Self {
    Self {
        profile: Profile::new(
            name,
            get_global_profile_type(),
            false,
            false,
        ),
    }
}
```

This approach uses generics to accept any type that can be converted into a `String`, which allows for more flexible usage while avoiding lifetime issues.

Which of these solutions would work best for your codebase?

A couple of issues:

```
warning: unused return value of `profiling::ProfileSection::new` that must be used
   --> src/profiling.rs:943:9
    |
943 |         $crate::profiling::ProfileSection::new($name)
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
   ::: src/styling.rs:787:13
    |
787 |             profile_section!("INSTANCE::get_or_init");
    |             ----------------------------------------- in this macro invocation
    |
    = note: `#[warn(unused_must_use)]` on by default
    = note: this warning originates in the macro `profile_section` (in Nightly builds, run with -Z macro-backtrace for more info)
help: use `let _ = ...` to ignore the resulting value
    |
943 |         let _ = $crate::profiling::ProfileSection::new($name)
    |         +++++++
```
I'm getting a lot of these warnings, suggesting that it doesn't understand that we want to return this value.
Secondly, I had to change the generic S to implement Display, and add `let name = name.to_string();` to deal with the fallout:
```
    pub fn new<S: Into<String> + Display>(
        name: S,
        requested_type: ProfileType,
        is_async: bool,
        is_method: bool,
    ) -> Option<Self> {
        if !is_profiling_enabled() {
            return None;
        }

        let name = name.to_string();
```
Now the compiler is suggesting I take a reference instead: `name: &S`, but when I do all hell breaks loose relating to Sized and size not being known at compile time. So why is Rust suggesting this and is all this worth it?


# Simplifying Your Profiling Implementation

You're hitting several common Rust issues related to generics, string handling, and the `#[must_use]` attribute. Let's solve them one by one:

## 1. Fixing the "unused must-use value" warnings

This warning happens because your `ProfileSection` struct is marked with `#[must_use]` (either explicitly or implicitly through a derive). The compiler is correctly telling you that you're creating a value but not using it. Since we specifically want people to save and use the `ProfileSection` object, we should address this in the macro:

```rust
#[macro_export]
macro_rules! profile_section {
    ($name:expr) => {
        let _section = $crate::profiling::ProfileSection::new($name);
        _section
    };
}
```

This way, when someone uses just `profile_section!("name")` without assigning it, the value is stored in `_section` and returned.

## 2. Simplifying String Handling

The generic trait bounds are getting overly complex. Let's simplify by going back to a straightforward approach:

```rust
impl ProfileSection {
    // Create a new profile section
    pub fn new(name: &str) -> Self {
        Self {
            profile: Profile::new(
                name.to_string(), // Pass an owned String directly
                get_global_profile_type(),
                false,
                false,
            ),
        }
    }
}
```

And in your `Profile` implementation:

```rust
impl Profile {
    pub fn new(
        name: String,  // Take an owned String directly
        requested_type: ProfileType,
        is_async: bool,
        is_method: bool,
    ) -> Option<Self> {
        if !is_profiling_enabled() {
            return None;
        }

        // Now you can use name directly
        Some(Self {
            name,
            // other fields...
        })
    }
}
```

This approach is much simpler:
- `ProfileSection::new` takes a `&str` and creates a `String`
- `Profile::new` takes ownership of the `String` directly
- No complex generics or trait bounds needed

## 3. If You Really Need Generics

If you still want to keep the generic parameter for flexibility, here's a cleaner approach:

```rust
impl Profile {
    pub fn new<S: Into<String>>(
        name: S,
        requested_type: ProfileType,
        is_async: bool,
        is_method: bool,
    ) -> Option<Self> {
        if !is_profiling_enabled() {
            return None;
        }

        Some(Self {
            name: name.into(),  // Convert directly to String
            // other fields...
        })
    }
}

impl ProfileSection {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            profile: Profile::new(
                name,  // Pass through the generic parameter
                get_global_profile_type(),
                false,
                false,
            ),
        }
    }
}
```

This approach:
- Uses `Into<String>` to allow any type that can be converted to a String
- Doesn't need additional trait bounds like `Display`
- Passes the generic parameter directly from `ProfileSection::new` to `Profile::new`

## 4. Addressing the "Is It Worth It" Question

Given the issues you're encountering, it's worth considering if the generic approach is justified. Here's my recommendation:

1. **For the simplest solution**: Use the direct `String` approach (option 2)
2. **If you need flexibility**: Use `Into<String>` (option 3)
3. **If neither works well**: Consider using a string slice (`&str`) for `Profile::new` and have it do the string allocation internally

The simplest solution is likely to be the best one here. Over-generalization can lead to exactly the kind of issues you're experiencing.

Would you like me to provide a complete implementation of the simplified approach?


I'm getting a lot of errors like this:
```
warning: path statement drops value
   --> src/profiling.rs:936:9
    |
936 |         _section
    |         ^^^^^^^^ help: use `drop` to clarify the intent: `drop(_section);`
    |
   ::: src/cmd_args.rs:311:9
    |
311 |         profile_section!("config_loop_assert");
    |         -------------------------------------- in this macro invocation
    |
```
One thing after another...

# Fixing the "path statement drops value" warning

You're encountering a common warning in Rust macros. When a macro expands to a value expression at the end (like `_section`), Rust interprets it as a statement that doesn't do anything with the value. Let's fix this by making our macro more robust:

```rust
#[macro_export]
macro_rules! profile_section {
    ($name:expr) => {{
        let section = $crate::profiling::ProfileSection::new($name);
        section  // Return the section
    }};
}
```

The double curly braces `{{` and `}}` create a block expression that returns the value of the last expression.

## Complete Revised Implementation

Here's a complete implementation that should resolve all these issues:

```rust
// In profiling.rs:

pub struct ProfileSection {
    profile: Option<Profile>,
}

impl ProfileSection {
    pub fn new(name: &str) -> Self {
        Self {
            profile: Profile::new(
                name.to_string(),
                get_global_profile_type(),
                false, // log_to_file
                false, // log_to_stdout
            ),
        }
    }

    pub fn end(self) {
        // Profile (if any) will be dropped here
    }

    pub fn is_active(&self) -> bool {
        self.profile.is_some()
    }
}

// Update Profile::new to accept a String
impl Profile {
    pub fn new(
        name: String,
        requested_type: ProfileType,
        is_async: bool,
        is_method: bool,
    ) -> Option<Self> {
        if !is_profiling_enabled() {
            return None;
        }

        Some(Self {
            name,
            // other fields...
        })
    }
}

// The improved macro
#[macro_export]
macro_rules! profile_section {
    ($name:expr) => {{
        let section = $crate::profiling::ProfileSection::new($name);
        section
    }};
}
```

## Usage Examples

Now you can use the macro in several ways:

### Basic - Assign to Variable and End Explicitly

```rust
fn my_function() {
    let section = profile_section!("expensive part");
    expensive_operation();
    section.end();
}
```

### Fire-and-Forget (Auto-ends at End of Scope)

```rust
fn my_function() {
    let _section = profile_section!("whole function");
    // Underscore prefix to avoid unused variable warning
    // This will end when it goes out of scope at the end of the function

    expensive_operation();
}
```

### Using Just as a Statement

```rust
fn my_function() {
    // This works but will end immediately at the semicolon
    // Not very useful for profiling
    profile_section!("pointless");

    expensive_operation();
}
```

## Explanation

1. We've simplified the implementation to use plain Strings
2. The macro now returns the section in a block expression
3. This approach lets users explicitly control when profiling ends
4. No more "path statement drops value" warnings

This approach should give you a clean, ergonomic API without the warnings.

Thank you, I'm very happy with it. Now I want to think about ways to switch instrumentation on and off. Currently we have two alternative ways to enable profiling: via the `profiling` feature or via the #[enable_profiling] attribute. What I want to do would be straightforward if we were only using the feature, namely to turn off instrumentation when profiling is disabled. We could just test for the feature and disable the generation of the sync and async wrappers if it's not enabled. But as things are, we have the enable_profiling attribute, which is useful in certain contexts. Do we need another default feature, such as `instrument`, to control instrumentation when we're not using the feature? Or is there a way we could combine the two attributes, e.g. if #[profile] is on main, it could function as a toggle for profiling and instrumentation? I.e. replace #[enable_profiling] with #[profile] deployed on main. Unfortunately I can't see my way forward with this, but you quite often have ideas I haven't thought of.

# Refining Profiling and Instrumentation Control

You raise an excellent point about having more flexible control over profiling and instrumentation. Let me suggest a few approaches that could streamline your current system.

## Option 1: Hierarchical Control System

This approach uses a hierarchy of controls, from compile-time features down to runtime toggles:

```
1. Compile-time feature flag `profiling`: Controls if profiling code is included
2. Compile-time feature flag `instrument`: Controls if instrumentation is generated
3. Attribute #[profile] on functions: Marks functions for profiling
4. Attribute #[profile] on main: Acts as a global toggle
```

### Implementation

```rust
// In your proc macro crate:

#[proc_macro_attribute]
pub fn profile(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let name = &input.sig.ident;

    // Check if this is the main function
    let is_main = name.to_string() == "main";

    // If this is main, set a global flag
    if is_main {
        if cfg!(feature = "profiling") {
            // Generate code to enable profiling at runtime
            quote! {
                #[allow(non_upper_case_globals)]
                static #PROFILING_ENABLED: std::sync::atomic::AtomicBool =
                    std::sync::atomic::AtomicBool::new(true);

                #input
            }
        } else {
            // Without the profiling feature, do nothing special
            quote! { #input }
        }
    } else {
        // For regular functions, apply profiling if enabled
        if cfg!(feature = "profiling") {
            // Generate profiling wrapper
            generate_profiling_wrapper(attr, input)
        } else if cfg!(feature = "instrument") {
            // With instrument but not profiling, generate just the instrumentation
            generate_instrumentation_only(attr, input)
        } else {
            // If neither feature is enabled, pass through unchanged
            quote! { #input }
        }
    }
}

// Add helper function to check if profiling is enabled
pub fn is_profiling_enabled() -> bool {
    if cfg!(feature = "profiling") {
        // Check the global flag
        unsafe { PROFILING_ENABLED.load(std::sync::atomic::Ordering::Relaxed) }
    } else {
        false
    }
}

// Helper function to check if we should instrument
pub fn should_instrument() -> bool {
    cfg!(any(feature = "profiling", feature = "instrument"))
}
```

## Option 2: Simplified Combined Approach

This approach simplifies by combining the two control mechanisms:

```rust
#[proc_macro_attribute]
pub fn profile(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let name = &input.sig.ident;

    // Always check for main function
    if name.to_string() == "main" {
        // Main with #[profile] enables profiling globally
        quote! {
            #[allow(non_upper_case_globals)]
            static PROFILING_ENABLED: std::sync::atomic::AtomicBool =
                std::sync::atomic::AtomicBool::new(true);

            #input
        }
    } else {
        // For regular functions
        if should_profile_function(&input) {
            generate_profiling_wrapper(attr, input)
        } else {
            quote! { #input }
        }
    }
}

// Helper function to determine if we should profile a function
fn should_profile_function(input: &ItemFn) -> bool {
    // Check feature flags
    if !cfg!(any(feature = "profiling", feature = "instrument")) {
        return false;
    }

    // Additional checks can go here
    true
}
```

## Option 3: Profile Registry Approach

This approach introduces a "profile registry" that's populated at compile time and can be controlled at runtime:

```rust
// In your macro crate
#[proc_macro_attribute]
pub fn profile(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let name = &input.sig.ident;
    let fn_path = get_full_path(name); // Helper to get fully qualified name

    if cfg!(feature = "profiling") {
        // Register this function for profiling
        quote! {
            const _: () = {
                #[link_section = ".profile_registry"]
                static REGISTER: &'static [u8] = #fn_path.as_bytes();
            };

            // Generate profiling wrapper
            #generate_profiling_wrapper(attr, input)
        }
    } else if cfg!(feature = "instrument") {
        // Just instrument without profiling
        generate_instrumentation_only(attr, input)
    } else {
        // No features enabled, return unchanged
        quote! { #input }
    }
}
```

## Recommendation: Combined Hierarchical Approach

I recommend a combination of Options 1 and 2, which provides flexibility while remaining straightforward:

1. Use `profiling` feature to control if profiling code is included
2. Use `instrument` feature to control if instrumentation is generated without profiling
3. Use `#[profile]` on main to act as a runtime toggle
4. Use `#[profile]` on functions to mark them for profiling when enabled

This gives you a clean hierarchy:
- Compile-time features control if profiling/instrumentation is available
- Putting `#[profile]` on main controls if it's enabled at runtime
- Individual functions marked with `#[profile]` are profiled when enabled

### Complete Implementation

```rust
// In your proc macro crate:

#[proc_macro_attribute]
pub fn profile(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let name = &input.sig.ident;

    // Check if this is main
    let is_main = name.to_string() == "main";

    if is_main {
        // Putting #[profile] on main enables profiling globally
        if cfg!(feature = "profiling") {
            let result = quote! {
                #[allow(non_upper_case_globals)]
                pub static PROFILING_ENABLED: std::sync::atomic::AtomicBool =
                    std::sync::atomic::AtomicBool::new(true);

                // Original function
                #input
            };
            return result.into();
        } else {
            // Without profiling feature, just return main unchanged
            return quote! { #input }.into();
        }
    }

    // For regular functions
    if cfg!(feature = "profiling") {
        // With profiling feature, generate full profiling wrapper
        generate_profiling_wrapper(attr, input).into()
    } else if cfg!(feature = "instrument") {
        // With instrument feature, generate just instrumentation
        generate_instrumentation_only(attr, input).into()
    } else {
        // Neither feature enabled, return unchanged
        quote! { #input }.into()
    }
}

// Helper function in your runtime library:
pub fn is_profiling_enabled() -> bool {
    #[cfg(feature = "profiling")]
    {
        // Check the global toggle
        use std::sync::atomic::Ordering;
        static CHECKED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        static mut ENABLED: bool = false;

        if !CHECKED.load(Ordering::Relaxed) {
            // First time check - look for PROFILING_ENABLED
            if let Ok(val) = std::env::var("THAG_PROFILE") {
                let enabled = val.to_lowercase() != "false" && val != "0";
                unsafe { ENABLED = enabled; }
            } else {
                // Default to enabled if the feature is on
                unsafe { ENABLED = true; }
            }
            CHECKED.store(true, Ordering::Relaxed);
        }

        unsafe { ENABLED }
    }

    #[cfg(not(feature = "profiling"))]
    {
        false
    }
}
```

This approach gives you:
1. Clean compile-time control via features
2. Runtime control via `#[profile]` on main
3. The ability to profile specific functions
4. Environmental overrides via `THAG_PROFILE` environment variable

Does this approach align with what you're looking for? I can modify it if you have specific requirements I've missed.


I have some doubts. Firstly I should mention that I'm not sure we mean the same thing by instrumentation. I mean simply to instrument for profiling, i.e. generate the wrappers, so that the profiling code is included in the final binary. So profiling would happen either if 1. feature `profiling` was enabled to turn on wrapper generation and profiling or 2. feature `instrument` was enabled to turn on wrapper generation and attribute #[enable_profiling] was enabled to turn on profiling.
But mainly I'm looking for a way for #[enable_profiling] on the main function - or some equivalent proc macro - to control the compile-time behaviour in such a way as to switch the wrapper generation on or off across the board, so that we don't need an `instrument` feature. Which strikes me as a tall order. There's always build.rs as an alternative I guess. Or do we set up two alternative versions of the library, one with profiling and one without that we can switch between? I want the user to be able to instrument their code with #[profile] attributes without having to worry about the details of the profiling implementation.


# Clarifying Instrumentation, Profiling, and Control

Thanks for clarifying what you mean by instrumentation. I'll address your concerns and suggest an approach that might work for your needs.

## Understanding Your Requirements

1. **Instrumentation** = Generating profiling wrappers (the code structure needed for profiling)
2. **Profiling** = Actually collecting and reporting timing data
3. You want `#[profile]` attributes to work regardless of whether profiling is active
4. You want to control compile-time behavior with a proc macro attribute like `#[enable_profiling]` on main

## The Challenge

The fundamental challenge is that you want a proc macro attribute to control code generation in *other* proc macros. This is tricky because:

1. Proc macros run independently during compilation
2. There's no built-in way for one proc macro to communicate with another
3. Proc macros can't directly affect how other proc macros generate code

## Potential Solutions

### Option 1: Using Environment Variables During Build

One approach is to use a combination of `build.rs` and environment variables:

```rust
// In build.rs
fn main() {
    // Check if we should enable profiling wrappers
    let enable_profiling = std::env::var("ENABLE_PROFILING").unwrap_or_else(|_| "false".to_string());

    // Pass this to the compiler so proc macros can see it
    println!("cargo:rustc-env=ENABLE_PROFILING={}", enable_profiling);
}

// In your proc macro crate
#[proc_macro_attribute]
pub fn enable_profiling(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // Set the environment variable for the current compilation
    std::env::set_var("ENABLE_PROFILING", "true");

    quote! { #input }.into()
}

#[proc_macro_attribute]
pub fn profile(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // Check if profiling wrappers are enabled
    let enable_wrappers = std::env::var("ENABLE_PROFILING").unwrap_or_else(|_| "false".to_string()) == "true"
        || cfg!(feature = "profiling");

    if enable_wrappers {
        generate_profiling_wrapper(attr, input).into()
    } else {
        quote! { #input }.into()
    }
}
```

However, this approach has limitations, as environment variables set during macro expansion might not affect other macro expansions reliably.

### Option 2: Compile-Time Feature Detection + Runtime Toggle

This is a more reliable approach:

```rust
// In your proc macro crate
#[proc_macro_attribute]
pub fn enable_profiling(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    quote! {
        #[allow(non_upper_case_globals)]
        pub static PROFILING_ENABLED: std::sync::atomic::AtomicBool =
            std::sync::atomic::AtomicBool::new(true);

        #input
    }.into()
}

#[proc_macro_attribute]
pub fn profile(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // Always generate the wrappers when requested via attribute
    // But make the profiling conditional on PROFILING_ENABLED
    generate_conditional_profiling_wrapper(attr, input).into()
}

// Helper function to generate wrapper with runtime check
fn generate_conditional_profiling_wrapper(attr: TokenStream, input: ItemFn) -> TokenStream {
    let name = &input.sig.ident;

    quote! {
        // Original function
        #[inline(always)]
        #input

        // Wrapper that checks if profiling is enabled at runtime
        #[no_mangle]
        pub fn #name(...) -> ... {
            if is_profiling_enabled() {
                // Call with profiling
                Profile::new(...);
                original_fn(...)
            } else {
                // Call without profiling overhead
                original_fn(...)
            }
        }
    }
}

// Runtime function to check if profiling is enabled
pub fn is_profiling_enabled() -> bool {
    // Try to access PROFILING_ENABLED static
    static CHECKED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    static mut ENABLED: bool = false;

    if !CHECKED.load(std::sync::atomic::Ordering::Relaxed) {
        // First time check
        if let Some(_) = std::env::var_os("PROFILING_ENABLED") {
            // We found the static, profiling is available
            unsafe { ENABLED = true; }
        }
        CHECKED.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    unsafe { ENABLED }
}
```

### Option 3: Split Library Approach

Another option is to have two versions of your library:

1. A base version with no profiling
2. A profiling version that wraps the base version and adds profiling

Users would choose which one to import based on their needs. This is a common pattern in mature libraries.

### Best Solution: Combined Feature + Conditional Generation

Given your requirements, I think the best approach is:

```rust
// In your proc macro crate

// Attribute for main function to enable profiling at runtime
#[proc_macro_attribute]
pub fn enable_profiling(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    quote! {
        #[allow(non_upper_case_globals)]
        pub static PROFILING_ENABLED: std::sync::atomic::AtomicBool =
            std::sync::atomic::AtomicBool::new(true);

        #input
    }.into()
}

// Attribute for functions to profile
#[proc_macro_attribute]
pub fn profile(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // Always generate instrumentation (wrappers)
    // The actual profiling will be controlled at runtime
    generate_zero_overhead_wrapper(attr, input).into()
}

// Helper to generate wrapper with minimal overhead when profiling is disabled
fn generate_zero_overhead_wrapper(attr: TokenStream, input: ItemFn) -> TokenStream {
    // Implementation that checks PROFILING_ENABLED at runtime
    // with near-zero overhead when disabled
}
```

This approach:
1. Always instruments functions with `#[profile]`
2. Uses `#[enable_profiling]` on main to control profiling at runtime
3. Has minimal overhead when profiling is disabled
4. Doesn't require additional features beyond the existing `profiling` feature

## Implementation Recommendation

Based on your needs, I recommend:

1. Keep the `profiling` feature for backward compatibility
2. Always generate instrumentation with `#[profile]`
3. Add a runtime check in the instrumentation to skip profiling when disabled
4. Use `#[enable_profiling]` on main to control the runtime flag

This provides users with a clean, single interface (`#[profile]`) without requiring them to understand the implementation details.

Does this align with what you're looking for?


It does something valuable I hadn't thought of by opening up a zero-cost runtime code path, which is very attractive. However, the holy grail remains to avoid generating the wrappers altogether if we're not going to use them. But maybe I've got confused by using thag to profile itself. For user code, all we need to do is include the `profiling` feature on the import of the `thag_rs` crate (or ``thag_profiling` crate if I decide to split it out).

# Finding the Holy Grail: Zero-Cost Profiling Attributes

You've identified an important distinction - there are two different use cases:

1. **Internal profiling**: Using profiling within your own crate (thag profiling itself)
2. **User profiling**: Users adding profiling to their code via your crate

Let me address both scenarios and propose a solution that achieves the holy grail: completely avoiding wrapper generation when profiling is disabled.

## The Holy Grail Solution

### For User Code

For user code, you're right that it's straightforward - they either include the `profiling` feature or they don't:

```toml
# User's Cargo.toml
[dependencies]
thag_rs = { version = "0.1", features = ["profiling"] }
```

This is clean and follows Rust conventions.

### For Internal Use (The Tricky Part)

For internal use (thag profiling itself), we can leverage conditional compilation at the attribute level:

```rust
// In your public-facing macro crate
#[proc_macro_attribute]
pub fn profile(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // Check if profiling feature is enabled
    if cfg!(feature = "profiling") {
        // Generate profiling wrapper
        generate_profiling_wrapper(attr, input).into()
    } else {
        // No wrapper generation - pass through unchanged
        quote! { #input }.into()
    }
}
```

This ensures that when the `profiling` feature is disabled, the `#[profile]` attribute becomes a no-op, and the original function is used directly without any wrapper generation.

### The Best of Both Worlds

To get the best of both worlds (zero wrapper generation when disabled, but still allowing runtime toggling), you can combine approaches:

```rust
#[proc_macro_attribute]
pub fn profile(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // Only generate wrappers when profiling feature is enabled
    if cfg!(feature = "profiling") {
        // Generate wrapper with runtime check
        generate_conditional_profiling_wrapper(attr, input).into()
    } else {
        // No wrapper, return original function unchanged
        quote! { #input }.into()
    }
}

// Runtime control via attribute on main
#[proc_macro_attribute]
pub fn enable_profiling(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // Only add the runtime toggle when profiling is enabled
    if cfg!(feature = "profiling") {
        quote! {
            #[allow(non_upper_case_globals)]
            pub static PROFILING_ENABLED: std::sync::atomic::AtomicBool =
                std::sync::atomic::AtomicBool::new(true);

            #input
        }.into()
    } else {
        // Without profiling feature, do nothing special
        quote! { #input }.into()
    }
}
```

## Implementation Details

To make this work optimally:

1. **Generate profiling wrappers conditionally** based on the feature flag
2. **Within those wrappers, include a runtime check** for further control
3. **Make the runtime check as efficient as possible** (inlinable, branch prediction hints)

Here's the detailed implementation:

```rust
fn generate_conditional_profiling_wrapper(attr: TokenStream, input: ItemFn) -> TokenStream {
    let name = &input.sig.ident;
    let original_name = format_ident!("__original_{}", name);

    // Create the wrapper with runtime check
    quote! {
        // Rename the original function
        #[inline(always)]
        fn #original_name #input.sig.generics (#input.sig.inputs) #input.sig.output {
            #input.block
        }

        // Create the wrapper
        #input.vis fn #name #input.sig.generics (#input.sig.inputs) #input.sig.output {
            // Check if profiling is enabled at runtime
            // Using likely/unlikely for branch prediction
            if likely(is_profiling_enabled()) {
                let _profile = Profile::new(
                    stringify!(#name),
                    get_global_profile_type(),
                    false,
                    false
                );
                #original_name(#(#input.sig.inputs),*)
            } else {
                // Direct call with no profiling overhead
                #original_name(#(#input.sig.inputs),*)
            }
        }
    }
}

// Helper function for branch prediction
#[inline(always)]
fn likely(b: bool) -> bool {
    #[cfg(feature = "profiling")]
    {
        std::intrinsics::likely(b)
    }
    #[cfg(not(feature = "profiling"))]
    {
        b
    }
}
```

## Resolving Your Confusion

For thag profiling itself, you have two options:

1. **Accept that internal profiling has a cost**: Keep the `profiling` feature enabled in debug builds but disabled in release builds
2. **Split the library**: Create a separate `thag_profiling` crate that depends on the core `thag` crate

## Recommendation

I recommend:

1. Keep the simple conditional compilation in the `#[profile]` attribute
2. Add the runtime toggle via `#[enable_profiling]` for fine-grained control
3. Document clearly that users should enable the `profiling` feature when they want profiling

This achieves the holy grail - zero wrapper generation when profiling is disabled - while maintaining the flexibility of runtime control when needed.

Does this address your concerns about avoiding wrapper generation entirely?



Thanks, I'm still digesting it but it makes sense. Perhaps we need to talk about whether to split out profiling into a separate crate (`thag_profiling`), which is something I've been considering on its own merits but which might help here because then the same rules would apply to `thag_rs` as to any other user code. And I think all we would have to do is make the `profiling` feature an attribute of the `thag_profiling` crate rather than of the `thag` crate. The user code just needs to include the `thag_profiling` crate to allow them to instrument their code with `#[profile]` attributes and the various `profile_*` declarative macros, and enable the `profiling` feature if they want to generate and use the profiling wrappers. We might need to work on the `profile_*` declarative macros to do nothing if profiling is disabled.


# Splitting Profiling into a Separate Crate

I think splitting profiling into a separate `thag_profiling` crate is an excellent idea. This approach offers several benefits:

## Advantages of a Separate Profiling Crate

1. **Cleaner Dependency Structure**: Users who don't need profiling don't pay for it at all
2. **Consistent Rules**: As you noted, the same rules would apply to `thag_rs` as to any user code
3. **Better Modularity**: Keeps the core functionality separate from diagnostic tools
4. **Reduced Compile Time**: For projects not using profiling, compilation will be faster
5. **Easier Maintenance**: Clearer separation of concerns in your codebase

## Implementation Strategy

Here's how you could structure this:

### 1. Create `thag_profiling` Crate

```toml
# thag_profiling/Cargo.toml
[package]
name = "thag_profiling"
version = "0.1.0"
edition = "2021"

[features]
default = []
profiling = []  # Main feature to enable actual profiling

[dependencies]
thag_rs = { path = "../thag_rs" }  # Core dependency
proc-macro2 = "1.0"
quote = "1.0"
syn = { version = "1.0", features = ["full"] }

[lib]
proc-macro = true
```

### 2. Define the Attribute Macro

```rust
// thag_profiling/src/lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn profile(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // Check if profiling feature is enabled
    if cfg!(feature = "profiling") {
        // Generate profiling wrapper
        generate_profiling_wrapper(attr, input).into()
    } else {
        // No wrapper generation - pass through unchanged
        quote! { #input }.into()
    }
}

// Also implement the enable_profiling attribute
#[proc_macro_attribute]
pub fn enable_profiling(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    if cfg!(feature = "profiling") {
        quote! {
            #[allow(non_upper_case_globals)]
            pub static PROFILING_ENABLED: std::sync::atomic::AtomicBool =
                std::sync::atomic::AtomicBool::new(true);

            #input
        }.into()
    } else {
        quote! { #input }.into()
    }
}
```

### 3. Implement the Declarative Macros

```rust
// thag_profiling/src/lib.rs (continued)

// Export the declarative macros
#[macro_export]
macro_rules! profile_section {
    ($name:expr) => {
        #[cfg(feature = "profiling")]
        {
            let section = $crate::ProfileSection::new($name);
            section
        }

        #[cfg(not(feature = "profiling"))]
        {
            // Return a dummy value that does nothing
            $crate::DummyProfileSection {}
        }
    };
}

// Implement the runtime structures
#[cfg(feature = "profiling")]
pub struct ProfileSection {
    profile: Option<thag_rs::Profile>,  // Use the core Profile type
}

#[cfg(feature = "profiling")]
impl ProfileSection {
    pub fn new(name: &str) -> Self {
        Self {
            profile: thag_rs::Profile::new(name.to_string(), thag_rs::ProfileType::User, false, false),
        }
    }

    pub fn end(self) {
        // Profile will be dropped here
    }

    pub fn is_active(&self) -> bool {
        self.profile.is_some()
    }
}

// Dummy implementation when profiling is disabled
#[cfg(not(feature = "profiling"))]
pub struct DummyProfileSection {}

#[cfg(not(feature = "profiling"))]
impl DummyProfileSection {
    pub fn end(self) {}
    pub fn is_active(&self) -> bool { false }
}
```

### 4. User Code Example

```rust
// In user crate
use thag_rs::SomeCore;
use thag_profiling::profile;

#[profile]  // This does nothing unless thag_profiling/profiling feature is enabled
fn my_function() {
    // Function code here

    // This also does nothing unless profiling is enabled
    let section = profile_section!("expensive part");

    // Expensive code

    section.end();
}
```

## In Their Cargo.toml:

```toml
[dependencies]
thag_rs = "0.1"
thag_profiling = { version = "0.1", features = ["profiling"] }  # Enable profiling
```

## Key Benefits of This Approach

1. **Zero Overhead Without Profiling**: When the feature is off, macros expand to the original code or no-ops
2. **Clear Opt-In**: Users explicitly add the crate and feature when they want profiling
3. **Self-Contained**: All profiling logic is in one dedicated crate
4. **API Consistency**: The profiling API remains the same regardless of whether profiling is enabled

## Required Changes to thag_rs

1. Move the profiling-specific code to the new crate
2. Expose any necessary types (like `Profile`) that the profiling crate needs
3. Remove the `profiling` feature from the core crate

## Runtime Toggle Still Works

You can still have the runtime toggle (`#[enable_profiling]`) in the profiling crate, which will only generate the toggle code when the `profiling` feature is enabled.

This approach gives you the best of both worlds:
- Zero cost when profiling is disabled
- Consistent rules for all users (including thag itself)
- Clean separation of concerns

What do you think of this approach? Is this the kind of separation you were envisioning?


Yes I like it. A couple of points:
1. The proc macros will need to remain in the `thag_proc_macros` crate, and the `thag_profiling` crate will need to have a dependency on this crate. This is because proc macros must be in a separate crate, and I don't think a separate proc macro crate is warranted.
2. I think I would like to keep `thag_profiling` in the same workspace as `thag_rs`, just as we did with `thag_proc_macros`.
3. Here are the `thag_rs` dependencies in the `profiling` module:
```
use crate::{lazy_static_var, static_lazy, ThagError, ThagResult, Verbosity};
```
`Verbosity` and `lazy_static_var` only used by a dead-code logging function that we can dispense with. I think we can replace ThagError and ThagResult with a custom error type and a custom result type. It only currently uses the ThagError::Profiling variant, so we can modify the latter to be constructed from a `thag_profiling` `ProfileError`. That leaves `static_lazy`, which is a declarative macro from the `thag_rs` `shared` module. It isn't specific to profiling, so I don't know what the best solution is. Perhaps just include it in `thag_profiling::profiling` without exporting it?


# Refining the thag_profiling Implementation Plan

You've raised excellent points about the practical implementation details. Let me address each one and provide a more refined plan:

## 1. Keeping Proc Macros in thag_proc_macros

You're absolutely right that proc macros need to be in a separate crate. This is a better approach:

```
thag_rs/
├── thag_proc_macros/      (existing proc macro crate)
│   └── src/
│       ├── lib.rs         (contains #[profile] attribute macro)
│       └── ...
├── thag_profiling/        (new profiling utilities crate)
│   └── src/
│       ├── lib.rs         (profiling API, declarative macros)
│       └── ...
└── thag_rs/               (core crate)
    └── src/
        ├── lib.rs
        └── ...
```

The `thag_proc_macros` crate would continue to contain the `#[profile]` attribute, but would conditionally forward to the implementation based on whether `thag_profiling` is enabled with the `profiling` feature.

## 2. Keeping thag_profiling in the Same Workspace

This is the ideal approach for maintainability - keeping all related crates in a single workspace:

```toml
# Root Cargo.toml
[workspace]
members = [
    "thag_rs",
    "thag_proc_macros",
    "thag_profiling",
]
```

## 3. Handling Dependencies

Let's address each dependency issue:

### Custom Error Type for thag_profiling

```rust
// thag_profiling/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    #[error("Profiling error: {0}")]
    General(String),

    #[error("Invalid profile section: {0}")]
    InvalidSection(String),

    // Add more variants as needed
}

pub type ProfileResult<T> = Result<T, ProfileError>;

// In thag_rs/src/error.rs, modify ThagError:
#[derive(Debug, thiserror::Error)]
pub enum ThagError {
    // ... existing variants

    #[error("Profiling error: {0}")]
    Profiling(#[from] thag_profiling::ProfileError),
}
```

### Handling static_lazy Macro

For the `static_lazy` macro, you have a few options:

1. **Copy the Implementation**: Just duplicate the macro in `thag_profiling` (simplest but not DRY)
2. **Create a Small Shared Utils Crate**: Extract common utilities to a new crate
3. **Keep It Internal**: Include it in `thag_profiling` only for internal use

I'd recommend either option 1 (for simplicity) or option 3 (for cleanliness):

```rust
// In thag_profiling/src/utils.rs
#[macro_export]
#[doc(hidden)] // Makes it not appear in documentation
macro_rules! static_lazy {
    ($name:ident: $type:ty = $expr:expr) => {
        // Copy implementation from thag_rs
    };
}
```

## Complete Implementation Plan

### 1. thag_proc_macros Changes

```rust
// thag_proc_macros/src/lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn profile(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // We need to check if the thag_profiling feature is enabled
    if cfg!(feature = "profiling") {
        generate_profiling_wrapper(attr, input).into()
    } else {
        // No wrapper, return original function
        quote! { #input }.into()
    }
}

// Now thag_proc_macros has a feature flag
```

### 2. thag_profiling Crate

```toml
# thag_profiling/Cargo.toml
[package]
name = "thag_profiling"
version = "0.1.0"
edition = "2021"

[features]
default = []
profiling = ["thag_proc_macros/profiling"]  # Enable profiling in proc macros too

[dependencies]
thag_proc_macros = { path = "../thag_proc_macros" }
```

```rust
// thag_profiling/src/lib.rs
mod error;

pub use error::{ProfileError, ProfileResult};

// Re-export the profile attribute from thag_proc_macros
pub use thag_proc_macros::profile;

// Internal static_lazy macro
#[macro_export]
#[doc(hidden)]
macro_rules! static_lazy {
    // Implementation here
}

// Profiling section implementation
#[cfg(feature = "profiling")]
pub struct ProfileSection {
    profile: Option<Profile>,
}

#[cfg(feature = "profiling")]
impl ProfileSection {
    pub fn new(name: &str) -> Self {
        Self {
            profile: Profile::new(name.to_string(), ProfileType::User, false, false),
        }
    }

    pub fn end(self) {
        // Profile will be dropped here
    }

    pub fn is_active(&self) -> bool {
        self.profile.is_some()
    }
}

// Profile implementation
#[cfg(feature = "profiling")]
pub struct Profile {
    // Fields here
}

#[cfg(feature = "profiling")]
impl Profile {
    // Implementation here
}

// Dummy implementation when profiling is disabled
#[cfg(not(feature = "profiling"))]
pub struct ProfileSection;

#[cfg(not(feature = "profiling"))]
impl ProfileSection {
    pub fn new(_name: &str) -> Self { Self }
    pub fn end(self) {}
    pub fn is_active(&self) -> bool { false }
}

// Declarative macros
#[macro_export]
macro_rules! profile_section {
    ($name:expr) => {{
        let section = $crate::ProfileSection::new($name);
        section
    }};
}
```

### 3. thag_rs Changes

```rust
// thag_rs/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum ThagError {
    // ... existing variants

    #[error("Profiling error: {0}")]
    Profiling(String),
}

// Convert from ProfileError to ThagError
#[cfg(feature = "profiling")]
impl From<thag_profiling::ProfileError> for ThagError {
    fn from(err: thag_profiling::ProfileError) -> Self {
        ThagError::Profiling(err.to_string())
    }
}
```

## Final Structure

With this approach:

1. Users add `thag_profiling` to use profiling
2. They enable the `profiling` feature when they want to actually profile
3. `thag_proc_macros` conditionally generates profiling code based on the feature
4. `thag_profiling` provides no-op implementations when profiling is disabled
5. The core `thag_rs` crate isn't affected by profiling changes

This should provide a clean separation while addressing all the dependency issues you mentioned.

Does this revised plan work for your needs?



Yes. Now currently for some reason we have:
```
thag_rs/
├── src/
│   └── proc_macros/       (existing proc macro crate)
│       ├── lib.rs         (contains #[profile] attribute macro)
│       └── ...
```

Will there be fallout from reversing the src and proc_macros directories and renaming the latter, to change the structure to what you suggest, namely:
```
thag_rs/
├── thag_proc_macros/      (existing proc macro crate)
│   └── src/
│       ├── lib.rs         (contains #[profile] attribute macro)
│       └── ...
```
?


# Managing the Directory Structure Change

Moving from `thag_rs/src/proc_macros/` to `thag_rs/thag_proc_macros/src/` is a significant structural change that will have several implications. Let me walk you through the potential fallout and how to manage it:

## Potential Issues

1. **Cargo.toml References**: Any references to the proc macro crate in Cargo.toml files will need to be updated
2. **Import Paths**: Code that imports from the proc macro crate will need updated paths
3. **Git History**: The file movement might disrupt Git history tracking
4. **Published Crate**: If the proc macro crate has been published separately, this is essentially a new crate
5. **User Dependencies**: Any external projects depending on the proc macro crate will need updates

## Step-by-Step Plan

Here's how to manage this transition with minimal disruption:

### 1. Create the New Directory Structure

```bash
# Create the new directory structure
mkdir -p thag_rs/thag_proc_macros/src

# Copy files over (don't move yet to preserve original)
cp -R thag_rs/src/proc_macros/* thag_rs/thag_proc_macros/src/
```

### 2. Update Cargo.toml Files

#### Root Workspace Cargo.toml (if it exists)

```toml
[workspace]
members = [
    "thag_rs",
    "thag_proc_macros",
    # Other members
]
```

#### New thag_proc_macros/Cargo.toml

Create this file with appropriate contents:

```toml
[package]
name = "thag_proc_macros"
version = "0.1.0"  # Match your current version
edition = "2021"
description = "Procedural macros for the thag library"
license = "MIT OR Apache-2.0"  # Match your current license

[lib]
proc-macro = true

[dependencies]
proc-macro2 = "1.0"
quote = "1.0"
syn = { version = "1.0", features = ["full"] }
# Any other dependencies

[features]
profiling = []  # New feature for conditional compilation
```

#### Update thag_rs/Cargo.toml

Update reference to proc macros:

```toml
[dependencies]
# Change this from
# proc_macros = { path = "src/proc_macros" }
# to
thag_proc_macros = { path = "../thag_proc_macros" }
```

### 3. Update Import Paths in thag_rs

Find all instances where the proc macros are imported:

```bash
grep -r "use proc_macros" thag_rs/src/
```

And update them to use the new path:

```rust
// Change from
use proc_macros::profile;

// To
use thag_proc_macros::profile;
```

### 4. Test the New Structure

Run tests to ensure everything works with the new structure:

```bash
cd thag_rs
cargo test

cd ../thag_proc_macros
cargo test
```

### 5. Remove the Old Directory (Once Confirmed Working)

After confirming the new structure works:

```bash
# Backup just in case
cp -R thag_rs/src/proc_macros thag_rs/src/proc_macros_backup

# Remove old directory
rm -rf thag_rs/src/proc_macros
```

### 6. Update Git Repository

If you're using Git:

```bash
git add thag_rs/thag_proc_macros
git rm -r thag_rs/src/proc_macros
git commit -m "Restructure: Move proc_macros to standalone crate"
```

## Migration Strategy for Users

If external projects depend on your proc macros, you have a few options:

1. **Version Bump with Breaking Change**: Clearly document the change in your changelog
2. **Feature Flag the New Structure**: Keep both old and new for a transition period
3. **Reexport from Old Location**: Temporarily maintain reexports from the old location

The simplest is to treat this as a breaking change and update your version accordingly.

## Minimizing Fallout

To minimize disruption:

1. **Make this change in isolation**: Don't combine it with other changes
2. **Update all examples and documentation**: Ensure consistent usage examples
3. **Run thorough tests**: Test all features that use proc macros
4. **Consider a transitional approach**: If you have many users, consider a gradual migration

With careful planning, you can make this structural change with minimal fallout. The main complexity will be updating import paths throughout your codebase.

Would you like me to elaborate on any specific aspect of this migration?


Is it worth making this change at all? The hardest part for me is potential disruption to my admittedly small user base. `thag_proc_macros` is currently published separately on crates.io as it has to be, although right now it's only got a single quite trivial proc macro that is used by `thag_rs`. The crates.io entry doesn't even mention the rep location. Do you really mean to say I can't publish a new version of this crate from the new location?

# Evaluating the Need for Directory Structure Change

You raise an excellent point about the potential disruption relative to the benefits. Let me reconsider the proposed change.

## Current Situation
- `thag_proc_macros` is already published on crates.io
- It's located at `thag_rs/src/proc_macros/`
- It has a small user base
- It's already working and separate as required for proc macros

## Key Question: Is the Change Worth It?

Given these facts, I think the directory structure change is **probably not worth the disruption**. Here's why:

1. **Publishing Continuity**: You can absolutely publish new versions from the current location without issue. The path within your repository doesn't affect the published crate.

2. **No Functional Benefit**: The proposed change offers no functional improvements - it's purely organizational.

3. **Potential User Confusion**: Any change creates potential confusion, even for a small user base.

4. **More Pressing Need**: Your main goal is to split out profiling functionality, not reorganize existing code.

## Better Alternative: Leave the Structure As-Is

I recommend:

1. **Keep `thag_proc_macros` where it is**: At `thag_rs/src/proc_macros/`
2. **Focus on the profiling split**: Create a new `thag_profiling` crate alongside
3. **Make `thag_profiling` depend on existing `thag_proc_macros`**: No structure changes needed

```
thag_rs/
├── src/
│   └── proc_macros/       (existing proc macro crate, unchanged)
│       ├── lib.rs         (contains #[profile] attribute macro)
│       └── ...
├── thag_profiling/        (new profiling utilities crate)
│   └── src/
│       ├── lib.rs         (profiling API)
│       └── ...
└── src/                   (core crate)
    ├── lib.rs
    └── ...
```

## Adding Features to thag_proc_macros

You can still add the `profiling` feature to `thag_proc_macros` without changing its location:

```toml
# thag_rs/src/proc_macros/Cargo.toml
[features]
profiling = []  # New feature
```

And then make `thag_profiling` depend on it with features:

```toml
# thag_profiling/Cargo.toml
[dependencies]
thag_proc_macros = { version = "0.x", features = ["profiling"] }
```

## Conclusion

Structural changes to published crates should be approached with caution. In this case, the potential benefits don't outweigh the disruption. You can accomplish your goal of separating profiling functionality without changing the existing directory structure.

Unless there's a compelling reason I've missed, I recommend keeping `thag_proc_macros` where it is and focusing on the functional improvements with `thag_profiling`.

I double-checked with ChatGPT and it said I could in fact keep publishing `thag_proc_macros` to crates.io from the new location. Perhaps I haven't been clear, but my gut says it should be possible and if I can do that I'm on board to make the directory structure change.



# Publishing from a New Directory Location

You're absolutely right - you can indeed continue to publish `thag_proc_macros` to crates.io from a new directory location without breaking compatibility for users. Let me clarify:

## Publishing From the New Location

When you publish a crate to crates.io, what matters is:

1. The crate **name** in Cargo.toml
2. The crate **version**
3. The crate **contents**

The physical location within your repo is irrelevant to crates.io and users of your published crate.

## How Publishing Works

When you run `cargo publish`:

1. Cargo packages up the crate based on its Cargo.toml
2. It uploads this package to crates.io
3. Users download it based on its name and version, not its source location

## Example Process

```bash
# Move to new directory structure
mv thag_rs/src/proc_macros thag_rs/thag_proc_macros

# Navigate to the new location
cd thag_rs/thag_proc_macros

# Publish as usual
cargo publish
```

Users who depend on `thag_proc_macros = "0.x"` will get the new version without any changes to their code. The repository location is completely transparent to them.

## Making the Change

Given this understanding, if you prefer the new directory structure for organizational clarity, go ahead with the change! It's a good practice to have proc macro crates at the workspace level rather than nested under another crate's src directory.

Follow the step-by-step plan I outlined earlier, and you'll be able to publish from the new location without disrupting your users.

The main precautions are:

1. Keep the crate name the same in Cargo.toml
2. Follow proper semantic versioning
3. Ensure the crate's public API remains compatible

With these in place, the transition should be smooth for your users.

Does this address your concerns about publishing from the new location?


Yes, thank you. Now to go back to an earlier recommendation you made which I was happy with, the `profiling` feature on the `thag_profiling` crate will be required to enable profiling. What I misremembered is that the profiling feature alone doesn't enable profiling, and we still need to specify the profile type/s we want. In the absence of the #[enable_profiling] attribute this currently requires something like this:
```
    if cfg!(feature = "profiling") {
        profiling::enable_profiling(true, profiling::ProfileType::Both)?;
    }
```
So I think everything fits together nicely. The role of #[enable_profiling] will be diminished in that it will no longer be able to override the absence of the `profiling` feature (now that it's moving to the `thag_profiling` dependency), but either it, or a direct function call as above, will still be required to enable profiling, and the `$[enable_profiling]` attribute will still be the more convenient way to do so.
Related, how would the `if cfg!(feature = "profiling")` change given that the feature is now going to be in the `thag_profiling` dependency?

# Using the `profiling` Feature from `thag_profiling`

You're absolutely right about the distinction between enabling the feature and setting the profile type. Let me address how this all fits together with the new structure and how to access the feature flag from the dependency.

## Checking for the Feature from a Dependency

When you want to check if a feature is enabled in a dependency, you need to use a slightly different syntax:

```rust
// Instead of:
if cfg!(feature = "profiling") {
    // ...
}

// Use this:
#[cfg(feature = "thag_profiling/profiling")]
{
    profiling::enable_profiling(true, profiling::ProfileType::Both)?;
}
```

The syntax `feature = "crate/feature"` is used to check if a feature from a specific dependency is enabled.

## Alternative Approach: Re-export the Feature

A cleaner approach is to have `thag_profiling` re-export its feature status through a constant or function:

```rust
// In thag_profiling/src/lib.rs
#[cfg(feature = "profiling")]
pub const PROFILING_ENABLED: bool = true;

#[cfg(not(feature = "profiling"))]
pub const PROFILING_ENABLED: bool = false;
```

Then in your code:

```rust
if thag_profiling::PROFILING_ENABLED {
    profiling::enable_profiling(true, profiling::ProfileType::Both)?;
}
```

## Full Implementation Example

Let's put all of this together:

### In thag_profiling/src/lib.rs:

```rust
// Flag to indicate if profiling is enabled at compile time
#[cfg(feature = "profiling")]
pub const PROFILING_ENABLED: bool = true;

#[cfg(not(feature = "profiling"))]
pub const PROFILING_ENABLED: bool = false;

// Profile types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileType {
    User,
    System,
    Both,
    None,
}

// Enable profiling function
pub fn enable_profiling(enabled: bool, profile_type: ProfileType) -> Result<(), ProfileError> {
    #[cfg(feature = "profiling")]
    {
        // Actual implementation
        set_global_profiling_status(enabled, profile_type);
        Ok(())
    }

    #[cfg(not(feature = "profiling"))]
    {
        // No-op when profiling is disabled
        Ok(())
    }
}

// The enable_profiling attribute macro
// (This will be in thag_proc_macros, but shown here for completeness)
#[proc_macro_attribute]
pub fn enable_profiling(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    if cfg!(feature = "profiling") {
        // Set up the profiling global flag
        quote! {
            const _: () = {
                // This runs when the binary starts
                thag_profiling::enable_profiling(true, thag_profiling::ProfileType::Both)
                    .expect("Failed to enable profiling");
            };

            #input
        }.into()
    } else {
        // Without the feature, do nothing
        quote! { #input }.into()
    }
}
```

### In user code:

```rust
use thag_profiling::{profile, enable_profiling, ProfileType};

// Option 1: Use the attribute
#[enable_profiling]
fn main() {
    // This will enable profiling if the feature is on
    // ...
}

// Option 2: Manual control
fn main() {
    // Check if profiling is available at compile time
    if thag_profiling::PROFILING_ENABLED {
        // Enable it with specific settings
        thag_profiling::enable_profiling(true, ProfileType::User)
            .expect("Failed to enable profiling");
    }

    // Rest of code
}
```

## How it All Fits Together

With this restructuring:

1. The `profiling` feature in `thag_profiling` controls whether profiling is available
2. The `#[enable_profiling]` attribute provides a convenient way to enable it when available
3. The `enable_profiling()` function gives more control over profile types
4. The `PROFILING_ENABLED` constant lets you easily check if profiling is available

This structure maintains the separation of concerns while providing a clean API that's easy to use. The attribute still has value as a more convenient way to enable profiling with default settings.

Does this implementation plan make sense for your needs?
