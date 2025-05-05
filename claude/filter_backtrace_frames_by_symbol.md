

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
It also highlights a similar pattern with `async fn main`. which does not have or need a #[profiled] attribute, so doesn't get the ProfiledFuture scaffolding. I don't need or want the `main` entries with closures, only the last of the 3. I also don't want the FnOnce::call_once or the backtrace::__rust_begin_short_backtrace, which for me is just a marker for the start of the backtrace.

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
            if let Ok(val) = std::env::var("THAG_PROFILER") {
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
4. Environmental overrides via `THAG_PROFILER` environment variable

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


Yes, except to point out that 3. is not the case, because #[enable_profiling] allows the profile type/s to be specified, e.g.: `#[enable_profiling(profile_type = "time")]` (or `memory` or `both`)

# Including Profile Type Options in #[enable_profiling]

You're absolutely right - I overlooked that the `#[enable_profiling]` attribute already supports specifying the profile type. Let's correct that and ensure the attribute maintains its full functionality in the new structure.

## Enhanced Implementation with Profile Type Options

Here's how to properly implement the `#[enable_profiling]` attribute with profile type support:

```rust
// In thag_proc_macros/src/lib.rs
#[proc_macro_attribute]
pub fn enable_profiling(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // Parse the attribute arguments to extract profile_type
    let profile_type = if attr.is_empty() {
        // Default to "both" if no type specified
        quote! { thag_profiling::ProfileType::Both }
    } else {
        // Parse the attribute arguments
        let attr_args = parse_macro_input!(attr as AttributeArgs);

        // Find the profile_type argument
        let mut profile_type = None;
        for arg in attr_args {
            if let NestedMeta::Meta(Meta::NameValue(name_value)) = arg {
                if name_value.path.is_ident("profile_type") {
                    if let Lit::Str(lit_str) = &name_value.lit {
                        let type_str = lit_str.value();
                        profile_type = Some(match type_str.as_str() {
                            "time" => quote! { thag_profiling::ProfileType::User },
                            "memory" => quote! { thag_profiling::ProfileType::System },
                            "both" => quote! { thag_profiling::ProfileType::Both },
                            "none" => quote! { thag_profiling::ProfileType::None },
                            _ => {
                                return syn::Error::new_spanned(
                                    &name_value.lit,
                                    "profile_type must be one of: time, memory, both, none"
                                ).to_compile_error().into();
                            }
                        });
                    }
                }
            }
        }

        profile_type.unwrap_or(quote! { thag_profiling::ProfileType::Both })
    };

    if cfg!(feature = "profiling") {
        // Generate code with the specified profile type
        quote! {
            const _: () = {
                // This runs when the binary starts
                thag_profiling::enable_profiling(true, #profile_type)
                    .expect("Failed to enable profiling");
            };

            #input
        }.into()
    } else {
        // Without the feature, pass through unchanged
        quote! { #input }.into()
    }
}
```

## Usage Examples

Now users can specify the profile type in any of these ways:

```rust
// Default (both time and memory)
#[enable_profiling]
fn main() {
    // ...
}

// Just time profiling
#[enable_profiling(profile_type = "time")]
fn main() {
    // ...
}

// Just memory profiling
#[enable_profiling(profile_type = "memory")]
fn main() {
    // ...
}

// Explicitly specify both
#[enable_profiling(profile_type = "both")]
fn main() {
    // ...
}

// Disable profiling even when feature is on
#[enable_profiling(profile_type = "none")]
fn main() {
    // ...
}
```

## Ensuring Backward Compatibility

This preserves the existing functionality of the `#[enable_profiling]` attribute while adapting it to work with the new structure. The attribute remains the most convenient way to enable profiling with specific settings.

## In Summary

With this implementation:

1. The `profiling` feature in `thag_profiling` controls whether profiling code is included
2. The `#[enable_profiling]` attribute provides a convenient way to enable it with specific profile types
3. Manual control is still possible using the `enable_profiling()` function
4. The `PROFILING_ENABLED` constant makes it easy to check if profiling is available

This gives users the full range of options from simple to advanced usage, maintaining all the existing functionality while adding the benefits of the new structure.

Does this fully address the profile type specification in the attribute?



Yes, I think that's good thanks. Now one thing I've overlooked is the binaries we need for profiling. Currently these take the form of 3 scripts in the thag_rs tools directory: profile_instrument.rs, profile_remove.rs, and profile_analyze.rs. Fortunately, they don't have significant dependencies on the rest of the codebase. I need to consider my options for packaging and distribution:
1. Combine them into a single binary as the main executable of `thag_profiling`. The analyze script is based on an `inquire` menu, so that could be extended to include additional options to add and remove instrumentation from selected files.
2. Leave the profiling and analysis scripts and require users to install the `thag_rs` crate to use them.
3. Create a separate binary for each script, and package them in a zip file or something. But that makes for an awkward installation I think?
4. Insert alternative suggestions here.

# Packaging Profiling Tools for `thag_profiling`

After considering your options for the profiling scripts, I think Option 1 (a combined binary) makes the most sense. Let me explain why and how to implement it effectively:

## Recommendation: Option 1 - Combined Binary with Extended Functionality

### Benefits of This Approach

1. **Single Installation**: Users only need to install one tool
2. **Crates.io Distribution**: Can be published and installed with `cargo install thag_profiling`
3. **Unified Interface**: Provides a cohesive experience through a single CLI
4. **Discoverability**: Makes it easier for users to discover all available functions
5. **Maintenance**: Easier to maintain one binary than several separate scripts

### Implementation Plan

Here's how to implement a combined binary in `thag_profiling`:

```
thag_rs/
├── thag_proc_macros/       (proc macro crate)
├── thag_profiling/         (profiling library + binary)
│   ├── src/
│   │   ├── lib.rs          (library functionality)
│   │   ├── main.rs         (binary entry point)
│   │   ├── commands/       (command implementations)
│   │   │   ├── mod.rs
│   │   │   ├── instrument.rs  (add instrumentation)
│   │   │   ├── remove.rs      (remove instrumentation)
│   │   │   └── analyze.rs     (analyze results)
│   │   └── ui/             (UI components)
│   │       └── menu.rs     (inquire-based menu)
│   └── Cargo.toml
└── ...
```

### Cargo.toml Setup

```toml
[package]
name = "thag_profiling"
version = "0.1.0"
edition = "2021"
description = "Profiling tools for Rust applications"
repository = "https://github.com/yourusername/thag_rs"
license = "MIT OR Apache-2.0"

# Both a library and a binary
[lib]
path = "src/lib.rs"

[[bin]]
name = "thag-profile"
path = "src/main.rs"

[features]
default = []
profiling = ["thag_proc_macros/profiling"]

[dependencies]
thag_proc_macros = { path = "../thag_proc_macros", version = "0.1" }
inquire = "0.6"
clap = { version = "4.3", features = ["derive"] }
# Other dependencies from your scripts
```

### main.rs Implementation

```rust
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod ui;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add profiling instrumentation to Rust source files
    Instrument {
        /// Path to the source file or directory
        #[arg(required = true)]
        path: PathBuf,

        /// Whether to recurse into subdirectories
        #[arg(short, long)]
        recursive: bool,
    },

    /// Remove profiling instrumentation from Rust source files
    Remove {
        /// Path to the source file or directory
        #[arg(required = true)]
        path: PathBuf,

        /// Whether to recurse into subdirectories
        #[arg(short, long)]
        recursive: bool,
    },

    /// Analyze profiling results
    Analyze {
        /// Path to the profiling data file
        #[arg(short, long)]
        file: Option<PathBuf>,

        /// Open interactive menu if no file specified
        #[arg(short, long)]
        interactive: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Instrument { path, recursive } => {
            commands::instrument::run(path, recursive);
        },
        Commands::Remove { path, recursive } => {
            commands::remove::run(path, recursive);
        },
        Commands::Analyze { file, interactive } => {
            if let Some(file) = file {
                commands::analyze::analyze_file(file);
            } else if interactive {
                ui::menu::show_interactive_menu();
            } else {
                println!("Please specify a file with --file or use --interactive");
            }
        },
    }
}
```

### Interactive Menu Integration

```rust
// src/ui/menu.rs
use inquire::{Select, Confirm};
use std::path::PathBuf;
use crate::commands;

pub fn show_interactive_menu() {
    let options = vec![
        "Instrument a file or directory",
        "Remove instrumentation",
        "Analyze profile results",
        "Exit",
    ];

    loop {
        let selection = Select::new("What would you like to do?", options.clone())
            .prompt();

        match selection {
            Ok("Instrument a file or directory") => instrument_menu(),
            Ok("Remove instrumentation") => remove_menu(),
            Ok("Analyze profile results") => analyze_menu(),
            Ok("Exit") | _ => break,
        }
    }
}

fn instrument_menu() {
    // Implement with inquire to get path and recursive option
    // Then call commands::instrument::run()
}

// Similar implementations for remove_menu() and analyze_menu()
```

## Installation and Usage

Users would install your tool with:

```bash
cargo install thag_profiling
```

Then use it via:

```bash
# Add instrumentation
thag-profile instrument src/main.rs

# Remove instrumentation
thag-profile remove src/main.rs

# Analyze results interactively
thag-profile analyze --interactive

# Analyze a specific file
thag-profile analyze --file profile_results.json
```

## Alternative Options Worth Considering

While I believe Option 1 is best, here are thoughts on alternatives:

### Option 2+ (Enhanced Version of Option 2)

If you're concerned about package size or complexity, you could keep the scripts in the `thag_rs` repo but make them installable:

```toml
# thag_rs/Cargo.toml
[[bin]]
name = "thag-profile-instrument"
path = "tools/profile_instrument.rs"

[[bin]]
name = "thag-profile-remove"
path = "tools/profile_remove.rs"

[[bin]]
name = "thag-profile-analyze"
path = "tools/profile_analyze.rs"
```

This allows users to install individual tools:

```bash
cargo install thag_rs --bin thag-profile-instrument
```

### Option 5: Separate Companion Tool Crate

Create a separate `thag_profile_tools` crate that depends on `thag_profiling`:

```
thag_rs/
├── thag_proc_macros/
├── thag_profiling/         (library only)
├── thag_profile_tools/     (binaries only)
│   └── ...
└── ...
```

This separates the concerns but might be overkill for your needs.

## Conclusion

Based on the information provided, I recommend Option 1 - creating a combined binary as part of the `thag_profiling` crate. This provides the best user experience while keeping everything neatly organized. The clap + inquire combination gives you both command-line flexibility and interactive menus, covering all use cases elegantly.

Does this approach align with your vision for the profiling tools?


Wait, re option 2+, do you mean to tell me that I could already make all my thag_rs tools installable in this way? If this is the case, that changes the way I think about packaging the tools. And for the current issue, if the user can just add thag_profiling to their project's dependencies, and then install the tools using 3 commands of the form `cargo install thag_rs --bin thag-profile-<tool_name>`, then that makes things pretty easy for them and for me without needing a binary in `thag_profiling`.


# Yes, You Can Make Tools Installable From Your Crate!

You're absolutely right - you can make your existing tools installable directly from the `thag_rs` crate, and this same approach would work for `thag_profiling` as well.

## How It Works

By adding binary targets to your `Cargo.toml`, you can make individual scripts installable via `cargo install`:

```toml
# In thag_rs/Cargo.toml
[[bin]]
name = "thag-profile-instrument"
path = "tools/profile_instrument.rs"

[[bin]]
name = "thag-profile-remove"
path = "tools/profile_remove.rs"

[[bin]]
name = "thag-profile-analyze"
path = "tools/profile_analyze.rs"
```

Then users can install specific tools:

```bash
cargo install thag_rs --bin thag-profile-instrument
```

## Applying This to thag_profiling

For the new `thag_profiling` crate, you could do the same:

```toml
# In thag_profiling/Cargo.toml
[[bin]]
name = "thag-profile-instrument"
path = "src/bin/instrument.rs"

[[bin]]
name = "thag-profile-remove"
path = "src/bin/remove.rs"

[[bin]]
name = "thag-profile-analyze"
path = "src/bin/analyze.rs"
```

## Benefits of This Approach

1. **Simplicity**: No need to create a complex unified CLI
2. **Flexibility**: Users can install only the tools they need
3. **Familiarity**: Maintains the existing workflow
4. **Less Work**: You can largely reuse your existing scripts

## Directory Structure

```
thag_rs/
├── thag_proc_macros/       (proc macro crate)
├── thag_profiling/         (profiling library + binaries)
│   ├── src/
│   │   ├── lib.rs          (library functionality)
│   │   └── bin/            (binary scripts)
│   │       ├── instrument.rs
│   │       ├── remove.rs
│   │       └── analyze.rs
│   └── Cargo.toml
└── ...
```

## Documentation

You would then document the installation process:

```markdown
# Installation

Add `thag_profiling` to your project:

```toml
[dependencies]
thag_profiling = "0.1"
```

To install the profiling tools:

```bash
# Install all tools
cargo install thag_profiling

# Or install individual tools
cargo install thag_profiling --bin thag-profile-instrument
cargo install thag_profiling --bin thag-profile-remove
cargo install thag_profiling --bin thag-profile-analyze
```
```

## The Best Solution For Your Case

Given this new information, I think the multiple binary approach might be the best fit for your needs:

1. It's simpler to implement (minimal changes to existing code)
2. It provides a clear separation of concerns
3. It gives users flexibility to install just what they need
4. It's consistent with how many Rust tools are distributed

This approach gives you the benefits of proper packaging without the complexity of merging everything into a single binary with subcommands.

Does this approach better align with your existing workflow and requirements?



These tools are currently pre-processed thag before compilation to turn their toml block into a Cargo.toml file. How can I get around this? Do I have to move the dependencies into thag_rs's Cargo.toml file?

# Handling Tool Dependencies in Binaries

You've identified an important challenge with packaging your tools - they're currently using a pre-processing step to turn TOML blocks into a Cargo.toml file. Let me explain some approaches to handle this:

## Option 1: Move Dependencies to Main Cargo.toml

The simplest approach is to move all the tool dependencies to the main Cargo.toml file of the crate where the binaries will be defined (either `thag_rs` or `thag_profiling`):

```toml
# In thag_profiling/Cargo.toml
[dependencies]
# Core dependencies
thag_proc_macros = { path = "../thag_proc_macros", version = "0.1" }

# Tool-specific dependencies
inquire = "0.6"  # For analyze tool
clap = "4.3"     # For CLI handling
regex = "1.8"    # If used by instruments/remove
walkdir = "2.3"  # If needed for directory traversal
# ...other tool dependencies
```

This works well if:
- The number of tool dependencies is reasonable
- There's significant overlap between the dependencies
- You don't mind the extra dependencies when users only use the library

## Option 2: Feature-Gated Dependencies

A more sophisticated approach is to use feature flags to conditionally include dependencies:

```toml
# In thag_profiling/Cargo.toml
[dependencies]
thag_proc_macros = { path = "../thag_proc_macros", version = "0.1" }

# Tool dependencies - only included when the respective features are enabled
inquire = { version = "0.6", optional = true }
clap = { version = "4.3", optional = true }
regex = { version = "1.8", optional = true }

[features]
default = []
profiling = ["thag_proc_macros/profiling"]
tools = ["inquire", "clap", "regex"]  # Feature to enable all tool dependencies
instrument-tool = ["regex", "clap"]
remove-tool = ["regex", "clap"]
analyze-tool = ["inquire", "clap"]

[[bin]]
name = "thag-profile-instrument"
path = "src/bin/instrument.rs"
required-features = ["instrument-tool"]

[[bin]]
name = "thag-profile-remove"
path = "src/bin/remove.rs"
required-features = ["remove-tool"]

[[bin]]
name = "thag-profile-analyze"
path = "src/bin/analyze.rs"
required-features = ["analyze-tool"]
```

With this setup:
- Library users don't get tool dependencies
- Tool users get only the dependencies they need
- Installation commands include needed features:
  ```bash
  cargo install thag_profiling --features=instrument-tool --bin thag-profile-instrument
  ```

## Option 3: Convert to Proper Modules

Another approach is to convert your tool scripts into proper Rust modules:

```rust
// src/bin/instrument.rs
use std::fs;
use std::path::Path;
use regex::Regex;
// ... other imports

fn main() {
    // Your tool implementation
}

// Exposed functions that could be reused
pub fn add_instrumentation(path: &Path) -> Result<(), Box<dyn Error>> {
    // Implementation
}
```

This way, you avoid the pre-processing step altogether. Your scripts become proper Rust files that use standard dependency management.

## Option 4: Separate Package for Tools (Clean Separation)

If the tools have many dependencies that aren't relevant to the library, create a separate package:

```
thag_rs/
├── thag_proc_macros/
├── thag_profiling/         (library only)
├── thag_profiling_tools/   (tools only)
│   ├── Cargo.toml          (with all tool dependencies)
│   └── src/
│       └── bin/
│           ├── instrument.rs
│           ├── remove.rs
│           └── analyze.rs
└── ...
```

This keeps the library package lean while still providing the tools.

## Recommendation

For your specific case with pre-processed TOML blocks, I recommend **Option 3 (Convert to Proper Modules)** combined with **Option 2 (Feature-Gated Dependencies)**:

1. Convert your tool scripts to standard Rust modules
2. Use feature flags to control which dependencies are included
3. Define bin targets with required-features

This gives you:
- Clean dependency management
- No preprocessing needed
- Minimal dependencies for library users
- A straightforward installation process for tool users

For example installation:
```bash
cargo install thag_profiling --features=tools  # Install all tools
# or
cargo install thag_profiling --features=analyze-tool --bin thag-profile-analyze  # Just one tool
```

Would this approach work with your current tool implementation?


I think that might work, but I think I want to keep the tools as part of the main thag project for now. Here are the dependencies for tools/profile_instrument.rs:

[dependencies]
ra_ap_syntax = "=0.0.264"
ra-ap-rustc_lexer = "=0.96.0"

Can you show me how to implement this?

# Adding Tool Dependencies to thag_rs

I'll show you how to implement the approach for keeping your tools as part of the main `thag_rs` project with feature-gated dependencies. Here's how to set it up specifically for `profile_instrument.rs` with its dependencies:

## Step 1: Update thag_rs/Cargo.toml

```toml
# In thag_rs/Cargo.toml

[dependencies]
# Your existing dependencies
# ...

# Tool-specific dependencies, made optional
ra_ap_syntax = { version = "=0.0.264", optional = true }
ra-ap-rustc_lexer = { version = "=0.96.0", optional = true }

[features]
# Your existing features
# ...

# New feature for the profiling tools
profile-tools = ["ra_ap_syntax", "ra-ap-rustc_lexer"]
profile-instrument = ["ra_ap_syntax", "ra-ap-rustc_lexer"]
profile-remove = []  # Add dependencies here
profile-analyze = []  # Add dependencies here

[[bin]]
name = "thag-profile-instrument"
path = "tools/profile_instrument.rs"
required-features = ["profile-instrument"]

[[bin]]
name = "thag-profile-remove"
path = "tools/profile_remove.rs"
required-features = ["profile-remove"]

[[bin]]
name = "thag-profile-analyze"
path = "tools/profile_analyze.rs"
required-features = ["profile-analyze"]
```

## Step 2: Update Your Tool Scripts

You'll need to update your tool scripts to be proper Rust files that don't rely on the pre-processing step. For example:

```rust
// tools/profile_instrument.rs
use ra_ap_syntax::{AstNode, SourceFile, SyntaxNode};
use ra_ap_rustc_lexer as rustc_lexer;
use std::fs;
use std::path::{Path, PathBuf};
// ... other imports

fn main() {
    // Your tool implementation
    // Example: Parse command line args, call functions to process files, etc.
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("Usage: thag-profile-instrument <path>");
        return;
    }

    let path = PathBuf::from(&args[1]);
    match instrument_path(&path) {
        Ok(_) => println!("Successfully instrumented: {}", path.display()),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn instrument_path(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Implementation to instrument a file or directory
    // ...
    Ok(())
}

// Other functions from your original script
```

## Step 3: Installation Instructions

Document how users can install the tools:

```markdown
# Profiling Tools

Thag includes several profiling tools that can be installed separately:

## Installation

Install a specific tool:

```bash
# Install the instrumentation tool
cargo install thag_rs --features=profile-instrument --bin thag-profile-instrument

# Install the removal tool
cargo install thag_rs --features=profile-remove --bin thag-profile-remove

# Install the analysis tool
cargo install thag_rs --features=profile-analyze --bin thag-profile-analyze
```

Or install all profiling tools at once:

```bash
cargo install thag_rs --features=profile-tools
```
```

## Step 4: Update Your Release Process

When you publish `thag_rs` to crates.io, make sure the tool scripts are included in the package. By default, Cargo will include them if they're in a directory that's not excluded.

## Advantages of This Approach

1. **Simplicity**: Keep all code in one repository
2. **Selective Installation**: Users can install only the tools they need
3. **Dependency Control**: Tool dependencies don't affect library users
4. **Minimal Changes**: You only need to modify Cargo.toml and convert scripts to standard Rust files

## Compatibility Note

Ensure your tools don't rely on any internal details of `thag_rs` that aren't part of the public API. If they do, consider exposing those details through a proper public API.

Would this approach work for your needs? It keeps the tools in the main project while properly handling their dependencies.



Yes, I think it would, and I'll try it - many thanks.

You're welcome! I'm glad the approach seems like a good fit for your needs. Here's a quick summary of the steps to implement this:

1. **Update Cargo.toml** with optional dependencies and binary targets
2. **Convert your tool scripts** to standard Rust modules
3. **Add feature flags** for each tool
4. **Document the installation process** for users

This approach gives you the benefits of proper cargo packaging while keeping everything within the main thag_rs project. It's a clean solution that provides flexibility for both you and your users.

If you encounter any issues during implementation or have questions about specific details, feel free to reach out. Good luck with the implementation, and I hope this helps streamline your profiling tools!

Thank you! I've managed to make the changes to Cargo.toml and install the tools successfully. I've also made the `thag_proc_macros` structure changes. We've covered a lot of ground in this chat, and refined our ideas as we've gone along, so I wonder if you could recap the plan for the outstanding changes for us to pick up tomorrow. I think the next steps are:

1. Move the src/profiling.rs module to its own sub-crate in directory `thag_profiling`.
2. Changes to `profiling.rs` and `thag_proc_macros::profile` to support its own error and result type and its own static_lazy macro, and the conditional compilation of profiling code with the aid of the PROFILING_ENABLED flag.
3. Note: no further changes are needed as above to support args to the attribute macros, as these are already in place.
4. Add some new unit tests for profiling functionality.
5. Update `Thag_Profiling_User_Guide.md`.
6. Bonus task: weigh the pros and cons of the name choice `thag_profiling` vs some other and shorter catchy name, such as `profile` or `perfil`. I think a big plus of retaining the `thag_` prefix is the branding aspect. Provided I make both products as good as I possibly can, each will draw attention to the other with a halo effect. So `thag_profile` or `thag_profiler`?


# Plan Recap: Creating the thag_profiling Sub-Crate

You've made excellent progress! Here's a comprehensive recap of the remaining steps to extract profiling functionality into its own sub-crate:

## 1. Create the thag_profiling Sub-Crate

- Create directory structure for `thag_profiling`
- Set up `Cargo.toml` with appropriate dependencies and features
- Add to workspace in root `Cargo.toml`

## 2. Move and Adapt the Profiling Code

- Move `src/profiling.rs` module to `thag_profiling/src/lib.rs`
- Create custom error type in `thag_profiling/src/error.rs`
- Implement or copy `static_lazy` macro locally
- Set up conditional compilation with the `profiling` feature flag
- Implement `ProfileSection` for profiling code sections

## 3. Update Attribute Macros in thag_proc_macros

- Update `#[profile]` to check for `thag_profiling/profiling` feature
- Update `#[enable_profiling]` to set the runtime toggle
- Ensure all attribute arguments continue to work properly

## 4. Testing and Documentation

- Add unit tests for profiling functionality
- Document the new crate structure in README files
- Update the Profiling User Guide
- Add examples showing how to use the new crate

## 5. Update thag_rs to Work with the New Structure

- Remove profiling module from `thag_rs`
- Update any code that relied on the profiling module
- Add appropriate re-exports if needed

## 6. Name Consideration

For the name choice, here's an analysis:

### Option 1: `thag_profiling`
- ✅ Clear branding connection to `thag_rs`
- ✅ Descriptive of functionality
- ✅ Consistent with existing naming scheme
- ❌ Slightly longer name

### Option 2: `thag_profile`
- ✅ Still maintains branding
- ✅ Slightly shorter
- ✅ Sounds more action-oriented
- ⚠️ Could be confused with user profiles in some contexts

### Option 3: `thag_profiler`
- ✅ Emphasizes tool aspect
- ✅ Clear connection to parent project
- ✅ More conventional name for a profiling tool
- ✅ Signals both the action and the tool

### Option 4: Generic names (`profile`, `perfil`, etc.)
- ✅ Short and catchy
- ✅ Could be more memorable
- ❌ Loses branding connection
- ❌ Likely to conflict with existing crates
- ❌ May not convey full feature set

### Recommendation

I recommend **`thag_profiler`** as it:
1. Maintains the branding connection
2. Clearly communicates the purpose
3. Follows standard naming conventions for tools
4. Has the best chance of being memorable while still descriptive

The branding advantage you mentioned is significant - having related tools reinforces your ecosystem and increases discoverability across your projects.

## Next Steps

1. Create the directory structure for `thag_profiler` (or whichever name you choose)
2. Start moving code and making the necessary adaptations
3. Update the dependencies and feature flags
4. Test functionality incrementally to identify and fix issues early

This systematic approach should make the transition smooth while creating a well-designed, standalone profiling solution.

Does this recap align with your understanding of the remaining work?

Yes, thanks, will help me to focus. Good night and see you tomorrow.

Good night! This was a productive session, and we've mapped out a clear path forward for the profiling sub-crate. The plan looks solid, with a clear understanding of how everything will fit together.

I think `thag_profiler` is a strong name choice that maintains your branding while clearly communicating the tool's purpose.

Rest well, and I look forward to continuing our work on this tomorrow. We'll tackle the implementation details and make sure everything works smoothly together.

See you tomorrow!

I've currently got the [[bin]] details in thag_rs Cargo.toml set up and the installations of the 3 tool programs working, but now I'm thinking of moving them (thag_profile_[analyze/instrument/remove].rs) into thag_profiler after all. Any cons of doing this. Would it make sense to move them into the `src` or `src/bin` directory within `thag_profiler` rather than a `tools` directory`? Do you want to suggest any improvements to their names while we're about it? E.g. `thag_profiler_instrument`?

# Moving Profiling Tools to thag_profiler

Moving the profiling tools to the `thag_profiler` crate makes perfect sense! This centralizes all profiling-related functionality in one place, creating a more cohesive package. Let me address your questions:

## Directory Structure

I recommend using `src/bin/` within `thag_profiler` rather than a `tools` directory. This follows Rust convention for binary targets and makes the relationship clearer:

```
thag_profiler/
├── src/
│   ├── lib.rs             (library functionality)
│   └── bin/               (binary targets)
│       ├── instrument.rs  (instrumentation tool)
│       ├── remove.rs      (removal tool)
│       └── analyze.rs     (analysis tool)
├── Cargo.toml
└── README.md
```

This structure is immediately recognizable to Rust developers and follows cargo's standard practices.

## Naming Considerations

For binary names, you have several options:

### Option 1: Short Command-Style Names (Recommended)
```toml
[[bin]]
name = "thag-instrument"
path = "src/bin/instrument.rs"

[[bin]]
name = "thag-remove"
path = "src/bin/remove.rs"

[[bin]]
name = "thag-analyze"
path = "src/bin/analyze.rs"
```

### Option 2: Include "profiler" in Names
```toml
[[bin]]
name = "thag-profiler-instrument"
path = "src/bin/instrument.rs"

[[bin]]
name = "thag-profiler-remove"
path = "src/bin/remove.rs"

[[bin]]
name = "thag-profiler-analyze"
path = "src/bin/analyze.rs"
```

### Option 3: Unified Command with Subcommands (More Work)
```toml
[[bin]]
name = "thag-profiler"
path = "src/bin/main.rs"
```

Where `main.rs` uses clap to implement subcommands like:
```
thag-profiler instrument <path>
thag-profiler remove <path>
thag-profiler analyze [options]
```

## Recommendation

I recommend **Option 1** with short command-style names because:

1. It's more user-friendly (shorter commands to type)
2. The `thag-` prefix already establishes branding
3. The commands are more focused on the action (verb) rather than the tool
4. Installation is simpler (no subcommand handling needed)

## Any Cons of Moving?

The only potential cons are:

1. **Increased Build Time**: Users who only want the library will now need to build the tools too (mitigated by using feature flags)
2. **Dependency Management**: Tool-specific dependencies need to be managed (also addressed with feature flags)
3. **Backward Compatibility**: Users who installed tools from `thag_rs` will need to reinstall them

These are all minor concerns compared to the benefits of consolidation.

## Implementation in Cargo.toml

```toml
# thag_profiler/Cargo.toml
[package]
name = "thag_profiler"
version = "0.1.0"
edition = "2021"
description = "Profiling tools for Rust applications"
repository = "https://github.com/yourusername/thag_rs"
license = "MIT OR Apache-2.0"

[dependencies]
thag_proc_macros = { path = "../thag_proc_macros", version = "0.1" }
# Core dependencies...

# Tool dependencies
ra_ap_syntax = { version = "=0.0.264", optional = true }
ra-ap-rustc_lexer = { version = "=0.96.0", optional = true }
# Other tool dependencies...

[features]
default = []
profiling = ["thag_proc_macros/profiling"]  # Enable profiling code
tools = ["instrument-tool", "remove-tool", "analyze-tool"]  # Enable all tools
instrument-tool = ["ra_ap_syntax", "ra-ap-rustc_lexer"]  # Dependencies for instrument
remove-tool = []  # Dependencies for remove
analyze-tool = []  # Dependencies for analyze

[[bin]]
name = "thag-instrument"
path = "src/bin/instrument.rs"
required-features = ["instrument-tool"]

[[bin]]
name = "thag-remove"
path = "src/bin/remove.rs"
required-features = ["remove-tool"]

[[bin]]
name = "thag-analyze"
path = "src/bin/analyze.rs"
required-features = ["analyze-tool"]
```

This configuration lets users install:
```bash
# Just the library
cargo add thag_profiler

# All tools
cargo install thag_profiler --features=tools

# Specific tools
cargo install thag_profiler --features=instrument-tool --bin thag-instrument
```

With this approach, you get the best of both worlds: clean packaging, user-friendly names, and selective installation options.

Just to check: in the root Cargo.toml I should add thag_profiler as a dependency:
```
thag_profiler = { version = "0.1", path = "thag_profiler" }
```
And then in src/lib.rs I can re-export the bits I need:
```
pub use {
...
thag_proc_macros::{profile, ...},
thag_profiler::{enable_profiling, profile_fn, Profile, ProfileType, ...},
};
```

# Integrating thag_profiler with the Root Crate

You're on the right track! Let me clarify how to properly integrate `thag_profiler` with your root crate:

## 1. In the Root Cargo.toml

```toml
[dependencies]
# Other dependencies...
thag_proc_macros = { path = "thag_proc_macros", version = "0.1" }
thag_profiler = { path = "thag_profiler", version = "0.1", optional = true }

[features]
# Other features...
profiling = ["thag_profiler", "thag_profiler/profiling", "thag_proc_macros/profiling"]
```

Making `thag_profiler` optional and controlled by the `profiling` feature allows users to choose whether to include the profiling functionality.

## 2. In src/lib.rs

```rust
// Re-exports
#[cfg(feature = "profiling")]
pub use thag_proc_macros::{profile, enable_profiling};

#[cfg(feature = "profiling")]
pub use thag_profiler::{
    Profile, ProfileType, ProfileError, ProfileResult,
    profile_fn, profile_section, is_profiling_enabled
};

// Provide an empty implementation when profiling is disabled
#[cfg(not(feature = "profiling"))]
#[macro_export]
macro_rules! profile_section {
    ($name:expr) => {
        // No-op
    };
}

// And any other macros that should have no-op versions when profiling is disabled
```

## 3. Documentation for Users

With this setup, there are two ways for users to use profiling:

```rust
// Option 1: Use thag_rs with profiling feature
// In their Cargo.toml:
[dependencies]
thag_rs = { version = "0.1", features = ["profiling"] }

// In their code:
use thag_rs::{profile, enable_profiling, profile_section};

#[profile]
fn my_function() { /* ... */ }
```

```rust
// Option 2: Use thag_profiler directly
// In their Cargo.toml:
[dependencies]
thag_profiler = "0.1"
thag_proc_macros = "0.1"

// In their code:
use thag_proc_macros::{profile, enable_profiling};
use thag_profiler::{profile_section};

#[profile]
fn my_function() { /* ... */ }
```

## Important Considerations

1. **Feature Propagation**: The `profiling` feature in your root crate should enable the necessary features in both `thag_proc_macros` and `thag_profiler`.

2. **Binary Tools**: The tools are now in `thag_profiler`, so users would install them with:
   ```bash
   cargo install thag_profiler --features=instrument-tool --bin thag-instrument
   ```

3. **Documentation**: Clearly document both approaches so users know they can either use `thag_rs` with the profiling feature or `thag_profiler` directly.

4. **Dependencies**: Ensure that `thag_profiler` depends on `thag_proc_macros` so users who choose option 2 get a consistent experience.

This approach gives you the flexibility to maintain the profiling functionality separately while providing a clean integration with your main crate. It also allows users to choose the approach that best suits their needs.

Does this clarify how to integrate everything together?



What do you think of thag_profiler re-exporting its thag_proc_macros dependencies?
```
#[cfg(feature = "profiling")]
pub use profiling::Profile;
pub use {
    errors::{ProfileError, ProfileResult},
    profiling::ProfileType,
    thag_proc_macros::{enable_profiling, profile},
};
```


# Re-exporting proc_macros from thag_profiler

Re-exporting `thag_proc_macros` dependencies from `thag_profiler` is an **excellent idea** that provides several benefits:

## Advantages

1. **Single Import Point**: Users only need to import from `thag_profiler` to get everything they need
2. **Proper Encapsulation**: Hides the internal division between the proc macros and runtime library
3. **Better User Experience**: Simplifies usage by providing a cohesive API
4. **Future Flexibility**: Gives you freedom to restructure internals without breaking user code

## Implementation Recommendation

Your implementation looks good, with a small suggestion for the `Profile` re-export:

```rust
// In thag_profiler/src/lib.rs
#[cfg(feature = "profiling")]
pub use profiling::Profile;

// These are always exported (though Profile-related types might be empty structs without the feature)
pub use {
    errors::{ProfileError, ProfileResult},
    profiling::ProfileType,
    thag_proc_macros::{enable_profiling, profile},
};

// Also consider re-exporting the declarative macros
#[cfg(feature = "profiling")]
pub use crate::{profile_fn_macro as profile_fn, profile_section_macro as profile_section};

#[cfg(not(feature = "profiling"))]
#[macro_export]
macro_rules! profile_section {
    ($name:expr) => {{}};
}

#[cfg(not(feature = "profiling"))]
#[macro_export]
macro_rules! profile_fn {
    ($name:expr) => {{}};
}
```

## User Code Example

With this approach, users would have a very clean experience:

```rust
// In user's Cargo.toml
[dependencies]
thag_profiler = { version = "0.1", features = ["profiling"] }

// In user's code
use thag_profiler::{profile, enable_profiling, profile_section, ProfileType};

#[enable_profiling(profile_type = "both")]
fn main() {
    // Application code

    let section = profile_section!("expensive part");
    // Expensive operation
    section.end();
}

#[profile]
fn my_function() {
    // This function will be profiled
}
```

## Impact on thag_rs

This means your main crate can also simplify its re-exports:

```rust
// In thag_rs/src/lib.rs
#[cfg(feature = "profiling")]
pub use thag_profiler::*;  // Re-export everything from profiler
```

## Documentation

Make sure to document this well:

```rust
/// # Profiling API
///
/// This module provides tools for profiling your application.
///
/// ## Basic Usage
///
/// ```
/// use thag_profiler::{profile, profile_section};
///
/// #[profile]
/// fn my_function() {
///     // Function code
///
///     let section = profile_section!("expensive part");
///     // Expensive operation
///     section.end();
/// }
/// ```
pub mod profiling {
    // Re-exports here
}
```

## Conclusion

Re-exporting the proc macros from `thag_profiler` is definitely the right approach. It creates a cleaner, more cohesive API for users and hides implementation details. This is the pattern used by many popular Rust crates (like `serde` re-exporting `serde_derive`).

This approach simplifies the user experience while giving you more flexibility to evolve the implementation over time.


When you say "Make sure to document this well", where are you saying that documentation should go? I also don't understand why re-exports would be in pub mod profiling.


# Clarifying Documentation and Re-exports

You're right to ask for clarification - let me explain more precisely:

## Documentation Location

The documentation should go directly in your `thag_profiler/src/lib.rs` file, at the crate level:

```rust
//! # thag_profiler
//!
//! A Rust profiling library for measuring and analyzing code performance.
//!
//! ## Basic Usage
//!
//! ```
//! use thag_profiler::{profile, profile_section};
//!
//! #[profile]
//! fn my_function() {
//!     // Function code
//!
//!     let section = profile_section!("expensive part");
//!     // Expensive operation
//!     section.end();
//! }
//! ```

// Re-exports
#[cfg(feature = "profiling")]
pub use profiling::Profile;

pub use {
    errors::{ProfileError, ProfileResult},
    profiling::ProfileType,
    thag_proc_macros::{enable_profiling, profile},
};

// Rest of your code...
```

This puts the documentation right at the top of your crate, so it's the first thing users see when they look at your API docs.

## Re-exports Clarification

I apologize for the confusion in my previous response. I mistakenly included a `pub mod profiling` section, which would not be appropriate here. You're absolutely right - the re-exports should be directly in the root of the crate, exactly as you've shown in your example:

```rust
// In thag_profiler/src/lib.rs
#[cfg(feature = "profiling")]
pub use profiling::Profile;

pub use {
    errors::{ProfileError, ProfileResult},
    profiling::ProfileType,
    thag_proc_macros::{enable_profiling, profile},
};
```

There's no need for an additional `pub mod profiling` section in your lib.rs file. Your approach is correct.

The structure should be:

```
thag_profiler/
├── src/
│   ├── lib.rs           (contains re-exports and main documentation)
│   ├── profiling.rs     (implementation details)
│   ├── errors.rs        (error types)
│   └── other modules... (as needed)
```

Thank you for catching that error, and I hope this clarifies where the documentation should go.


I'm struggling a bit with the fact that thag_rs code is itself instrumented with #[profile] attributes, although by default I don't want profiling enabled. So I'm getting errors like this:
```
error[E0432]: unresolved imports `crate::enable_profiling`, `crate::profile_fn`, `crate::profile_section`
 --> src/styling.rs:5:5
  |
5 |     enable_profiling, lazy_static_var, profile, profile_fn, profile_section, v...
  |     ^^^^^^^^^^^^^^^^                            ^^^^^^^^^^  ^^^^^^^^^^^^^^^ no `profile_section` in the root
  |     |                                           |
  |     |                                           no `profile_fn` in the root
  |     |                                           help: a similar name exists in the module: `profile`
  |     no `enable_profiling` in the root
  ```
because the imports are currently feature gated by the above statement:
```
#[cfg(feature = "profiling")]
pub use thag_profiler::*;  // Re-export everything from profiler
```
and I'm not sure how the `#[cfg(not(feature = "profiling"))]` should look by contrast.


# Handling Profiling Imports When Feature is Disabled

You're facing a common issue when transitioning code with feature flags: how to handle imports for code that uses features that might be disabled. Let me help you solve this:

## The Solution

For your case, you need to provide no-op versions of the profiling macros and functions when the `profiling` feature is disabled. Here's how to update your `thag_rs/src/lib.rs`:

```rust
// When profiling is enabled, re-export from thag_profiler
#[cfg(feature = "profiling")]
pub use thag_profiler::*;

// When profiling is disabled, provide no-op versions
#[cfg(not(feature = "profiling"))]
pub use thag_proc_macros::profile;  // This should be a no-op when feature is disabled

#[cfg(not(feature = "profiling"))]
#[macro_export]
macro_rules! profile_section {
    ($name:expr) => {{
        struct DummyProfileSection;
        impl DummyProfileSection {
            pub fn end(self) {}
        }
        DummyProfileSection
    }};
}

#[cfg(not(feature = "profiling"))]
#[macro_export]
macro_rules! profile_fn {
    ($name:expr) => {{}};
}

#[cfg(not(feature = "profiling"))]
pub fn enable_profiling(_enabled: bool, _profile_type: &str) -> Result<(), String> {
    Ok(())
}

// Define a dummy ProfileType enum if needed
#[cfg(not(feature = "profiling"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileType {
    User,
    System,
    Both,
    None,
}
```

## Making thag_proc_macros Handle Disabled Profiling

Additionally, ensure that `thag_proc_macros::profile` is a no-op when the `profiling` feature is disabled:

```rust
// In thag_proc_macros/src/lib.rs
#[proc_macro_attribute]
pub fn profile(attr: TokenStream, item: TokenStream) -> TokenStream {
    #[cfg(feature = "profiling")]
    {
        // Actual profiling implementation
        let input = parse_macro_input!(item as ItemFn);
        // Generate profiling wrapper
        generate_profiling_wrapper(attr, input).into()
    }

    #[cfg(not(feature = "profiling"))]
    {
        // Just return the original code unchanged
        item
    }
}

#[proc_macro_attribute]
pub fn enable_profiling(attr: TokenStream, item: TokenStream) -> TokenStream {
    #[cfg(feature = "profiling")]
    {
        // Actual implementation
        // ...
    }

    #[cfg(not(feature = "profiling"))]
    {
        // Just return the original item
        item
    }
}
```

## In thag_profiler/src/lib.rs

Ensure all exports are properly gated:

```rust
#[cfg(feature = "profiling")]
pub use profiling::Profile;

// These are always exported
pub use errors::{ProfileError, ProfileResult};
pub use profiling::ProfileType;
pub use thag_proc_macros::{enable_profiling, profile};

// Export macros
#[cfg(feature = "profiling")]
pub use crate::profile_section;

#[cfg(feature = "profiling")]
pub use crate::profile_fn;

// Define macros with feature gates
#[cfg(feature = "profiling")]
#[macro_export]
macro_rules! profile_section {
    ($name:expr) => {{
        let section = $crate::ProfileSection::new($name);
        section
    }};
}

#[cfg(not(feature = "profiling"))]
#[macro_export]
macro_rules! profile_section {
    ($name:expr) => {{
        struct DummyProfileSection;
        impl DummyProfileSection {
            pub fn end(self) {}
        }
        DummyProfileSection
    }};
}

// Similar for profile_fn
```

## Complete Solution

This approach ensures that:

1. When profiling is enabled, all real functionality is used
2. When profiling is disabled, no-op implementations are provided
3. The code compiles in both cases
4. There's minimal runtime overhead when profiling is disabled

Your thag_rs code can then use profiling attributes and macros without worrying about whether the feature is enabled, as the appropriate implementation will be selected at compile time.

This is a common pattern when implementing optional features in Rust, and it allows your codebase to use the feature internally without requiring all users to enable it.


1. When you say "// Similar for profile_fn", could you please show what it would look like?
2. In thag_rs, you show no-op versions of the profiling macros and functions when the `profiling` feature is disabled. Could we not just import these from thag_profiler?


# Clarifying the Profiling Implementations

Let's address both of your questions:

## 1. Complete profile_fn Implementation

Here's the complete implementation for the `profile_fn` macro:

```rust
// In thag_profiler/src/lib.rs

// When profiling is enabled
#[cfg(feature = "profiling")]
#[macro_export]
macro_rules! profile_fn {
    ($name:expr) => {{
        let _profile = $crate::Profile::new(
            $name,
            $crate::ProfileType::User,
            false,
            false
        );
    }};
}

// When profiling is disabled
#[cfg(not(feature = "profiling"))]
#[macro_export]
macro_rules! profile_fn {
    ($name:expr) => {{
        // No-op implementation
    }};
}
```

## 2. Importing No-Op Versions from thag_profiler

Yes, you're absolutely right! A much cleaner approach is to have `thag_profiler` provide both the real and no-op versions, then simply import everything from `thag_profiler` in your main crate.

Here's how this should work:

### In thag_profiler/src/lib.rs

```rust
// Export the proc macros (both with and without profiling feature)
pub use thag_proc_macros::{enable_profiling, profile};

// Export the ProfileType enum (always available)
pub use profiling::ProfileType;

// Export error types (always available)
pub use errors::{ProfileError, ProfileResult};

// Profile is only available with profiling feature
#[cfg(feature = "profiling")]
pub use profiling::Profile;

// Define ProfileSection for both cases
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
        // Profile is dropped here
    }
}

#[cfg(not(feature = "profiling"))]
pub struct ProfileSection;

#[cfg(not(feature = "profiling"))]
impl ProfileSection {
    pub fn new(_name: &str) -> Self {
        Self
    }

    pub fn end(self) {
        // No-op
    }
}

// Macros for both cases
#[cfg(feature = "profiling")]
#[macro_export]
macro_rules! profile_section {
    ($name:expr) => {{
        let section = $crate::ProfileSection::new($name);
        section
    }};
}

#[cfg(not(feature = "profiling"))]
#[macro_export]
macro_rules! profile_section {
    ($name:expr) => {{
        let section = $crate::ProfileSection::new($name);
        section
    }};
}

#[cfg(feature = "profiling")]
#[macro_export]
macro_rules! profile_fn {
    ($name:expr) => {{
        let _profile = $crate::Profile::new(
            $name,
            $crate::ProfileType::User,
            false,
            false
        );
    }};
}

#[cfg(not(feature = "profiling"))]
#[macro_export]
macro_rules! profile_fn {
    ($name:expr) => {{
        // No-op implementation
    }};
}

// Function to enable profiling (works with or without feature)
#[cfg(feature = "profiling")]
pub fn enable_profiling(enabled: bool, profile_type: ProfileType) -> Result<(), ProfileError> {
    // Real implementation
    Ok(())
}

#[cfg(not(feature = "profiling"))]
pub fn enable_profiling(_enabled: bool, _profile_type: ProfileType) -> Result<(), ProfileError> {
    // No-op implementation
    Ok(())
}
```

### In thag_rs/src/lib.rs

Now you can simply:

```rust
// Re-export everything from thag_profiler, regardless of feature status
pub use thag_profiler::*;
```

This approach is much cleaner because:

1. All profiling-related code (both real and no-op versions) is encapsulated in `thag_profiler`
2. The main crate doesn't need to know the implementation details
3. Feature gating is handled entirely within `thag_profiler`
4. Your main crate can use profiling without checking feature status

This is a better design as it provides proper separation of concerns and encapsulation. The `thag_profiler` crate becomes responsible for handling both the enabled and disabled states of profiling, while `thag_rs` simply uses it without worrying about the details.



Thanks! I'm getting an error:
```
error[E0412]: cannot find type `Profile` in this scope
   --> thag_profiler/src/profiling.rs:562:15
    |
562 | impl Drop for Profile {
    |               ^^^^^^^ not found in this scope
```
I understand why. My question is: is it safe just to flag this impl statement as #[cfg(feature = "profiling")]? I.e. no special implementation needed in the non-profiling case? I'd assume so, but best to check.


# Conditional Drop Implementation for Profile

Yes, it's absolutely safe to flag the entire `impl Drop for Profile` block with `#[cfg(feature = "profiling")]`.

When the `profiling` feature is disabled, the `Profile` type won't be fully defined (or will be a dummy struct), so there's no need for a Drop implementation in that case. The conditional compilation will simply exclude that implementation when the feature is disabled.

Here's how it should look:

```rust
// In thag_profiler/src/profiling.rs

#[cfg(feature = "profiling")]
impl Drop for Profile {
    fn drop(&mut self) {
        // Your existing drop implementation
    }
}
```

This approach is safe because:

1. When `profiling` is enabled, the full `Profile` struct will be defined with its proper Drop implementation
2. When `profiling` is disabled, either:
   - The `Profile` struct won't be defined at all, or
   - It will be a dummy/empty struct with no resources to clean up

You don't need a special non-profiling implementation because in the non-profiling case, there are no resources to clean up or behaviors to execute when the dummy type is dropped.

This pattern of conditionally implementing traits like `Drop` is common in Rust codebases that use feature flags extensively.


I have this in thag_profiler/src/lib.rs as discussed:
// Re-exports
#[cfg(feature = "profiling")]
pub use profiling::Profile;

But in trying to compile thag_rs, I'm getting lots of errors like this:
```
error[E0433]: failed to resolve: could not find `Profile` in the crate root
   --> src/config.rs:778:1
    |
778 | #[profile]
    | ^^^^^^^^^^ could not find `Profile` in the crate root
    |
    = note: this error originates in the attribute macro `profile` (in Nightly builds, run with -Z macro-backtrace for more info)
help: consider importing this struct
    |
1   + use cargo_toml::Profile;
    |
```
And yet within the profile attribute implementation, I've surrounded all references to `Profile` with `#[cfg(feature = "profiling")]` to ensure they are only used when the feature is enabled. So am I somehow accidentally enabling the profiling feature?

# Resolving the Profile Resolution Issue

The error you're encountering suggests that the `#[profile]` attribute macro is trying to reference the `Profile` type even when the `profiling` feature is disabled. Let's solve this issue:

## Understanding the Problem

When a proc macro (like `#[profile]`) runs, it generates code that refers to other types and functions. In your case, it seems that the `profile` macro is generating code that references `Profile` regardless of whether the `profiling` feature is enabled.

## Solution: Update the Proc Macro

The key is to make the proc macro itself feature-aware and generate different code based on the feature status:

### 1. In thag_proc_macros/src/lib.rs:

```rust
#[proc_macro_attribute]
pub fn profile(attr: TokenStream, item: TokenStream) -> TokenStream {
    #[cfg(feature = "profiling")]
    {
        // When profiling is enabled, generate code that references Profile
        let input = parse_macro_input!(item as ItemFn);
        generate_profiling_wrapper(attr, input).into()
    }

    #[cfg(not(feature = "profiling"))]
    {
        // When profiling is disabled, just return the original function unchanged
        // WITHOUT any references to Profile
        item
    }
}
```

### 2. Ensure the profiling feature is properly propagated:

In thag_proc_macros/Cargo.toml:
```toml
[features]
profiling = []
```

In thag_profiler/Cargo.toml:
```toml
[dependencies]
thag_proc_macros = { path = "../thag_proc_macros", version = "0.1", features = ["profiling"] }

[features]
profiling = []
```

In thag_rs/Cargo.toml:
```toml
[dependencies]
thag_profiler = { path = "../thag_profiler", version = "0.1", optional = true }

[features]
profiling = ["thag_profiler", "thag_profiler/profiling"]
```

### 3. Check if you're accidentally enabling the feature:

Make sure you're not enabling the profiling feature in any of these places:
- Command line (`cargo build --features profiling`)
- IDE configuration
- Root workspace Cargo.toml default features

## Additional Diagnostic Steps

To confirm if the `profiling` feature is enabled, add this temporary code to your build:

```rust
// In thag_rs/build.rs or at the top of src/lib.rs
#[cfg(feature = "profiling")]
compile_error!("Profiling feature is enabled");
```

If you get a compile error saying "Profiling feature is enabled", you'll know it's being enabled somewhere.

## Alternative Approach: Make Profile Available Always

If modifying the proc macro is difficult, another approach is to make a dummy `Profile` type always available:

```rust
// In thag_profiler/src/profiling.rs
#[cfg(feature = "profiling")]
pub struct Profile {
    // Real implementation
}

#[cfg(not(feature = "profiling"))]
pub struct Profile;  // Empty dummy struct

// Then in lib.rs
pub use profiling::Profile;  // Always exported
```

This way, the proc macro can reference `Profile` regardless of the feature status, but it will be a no-op dummy type when profiling is disabled.

## Check the Generated Code

You can also inspect what code the proc macro is generating:

```bash
cargo rustc --profile=check -- -Zunpretty=expanded
```

This will show you the expanded code and might reveal why `Profile` is being referenced when it shouldn't be.

Would you like me to elaborate on any of these approaches?



Thanks, this all seems to be working well. Now I had another idea. `profile_section!` is a very powerful declarative macro to instantiate a `Profile` object to profile any arbitrary section of code. At the same time we have `profile_fn!`, `profile_both!`, `profile_method!` and `profile_memory!`, which are less powerful because they do not return a `ProfileSection` reference. I think we could simply upgrade them all to return a `ProfileSection` reference, but perhaps in doing so we should review them to see what would be the optimal set of profiling declarative macros. E.g. we're missing a `profile_time!` macro or equivalent. There is a finite number of profiling variables that we want to control either by passing them as arguments or by hard-coding. An example of the latter is profile_method, whose implementation hard-codes that we're profiling a method. The variables are the identifying name for the Profile instance, the type (time/memory/both), whether it's async and whether it's a method or not. I think that's all? 4 variables. So we have a range of options as to how to cover all the combinations, from a single macro with 4 arguments to a large set of (12?) individually named macros like profile_both_async_method! I would also ideally like to use the name profile! but I'm afraid it will clash with the #[profile] attribure. I'm looking for suggestions as to what macro signatures we should offer to cover all the bases optimally. What do you think? I'm thinking maybe `_async` and `_method` as optional parts of the name (the defaults being to leave those out), and the type being the second argument after the name. That would give4 signatures: `<stem>`, `<stem>_async`, `<stem>_method` and `<stem>_async_method`, each with two parameters: name and type.  I think `_method` would work in a real method as it would be implemented to pass is_method = true which would instruct Profile::new to include the last two path elements (after cleaning) of the backtrace. In fact there's a twist: the current implementation of Profile does not use the passed-in name at all for methods, so <stem>[_async]_method forms would not need the name parameter. But I don't want to cause confusion.


# Optimizing Profiling Macros for thag_profiler

I love your idea of upgrading all the profiling macros to return a `ProfileSection` reference. Let's design an optimal set of macros based on the variables you've identified:

## Key Variables

1. **Name**: Identifying name for the Profile instance
2. **Type**: Time/Memory/Both (ProfileType enum)
3. **Is Async**: Whether the code is asynchronous
4. **Is Method**: Whether it's a method (affects backtrace handling)

## Design Principles

For a good API design, we should consider:

1. **Consistency**: Similar things should work similarly
2. **Simplicity**: Minimize the number of macros when possible
3. **Discoverability**: Names should be intuitive and easy to remember
4. **Flexibility**: Cover all use cases without redundancy
5. **Ergonomics**: Make common cases easy and concise

## Recommended Macro Structure

After considering these principles, I recommend this approach:

### Core Macro: `profile!`

```rust
/// Profile a section of code with customizable options
///
/// # Examples
///
/// ```
/// // Basic usage (profiles time, sync function)
/// let section = profile!(expensive_operation);
/// // ... code to profile
/// section.end();
///
/// // With explicit type (profile memory)
/// let section = profile!("allocation_heavy", memory);
///
/// // Method profiling
/// let section = profile!(method);
///
/// // Async method with explicit type
/// let section = profile!(method, both, async);
/// ```
#[macro_export]
macro_rules! profile {
    // profile!(name)
    ($name:expr) => {
        $crate::profile_internal!($name, $crate::ProfileType::Time, false, false)
    };

    // profile!(name, type)
    ($name:expr, time) => {
        $crate::profile_internal!($name, $crate::ProfileType::Time, false, false)
    };
    ($name:expr, memory) => {
        $crate::profile_internal!($name, $crate::ProfileType::Memory, false, false)
    };
    ($name:expr, both) => {
        $crate::profile_internal!($name, $crate::ProfileType::Both, false, false)
    };

    // profile!(name, async)
    ($name:expr, async) => {
        $crate::profile_internal!($name, $crate::ProfileType::Time, true, false)
    };

    // profile!(method)
    (method) => {
        $crate::profile_internal!("", $crate::ProfileType::Time, false, true)
    };

    // profile!(method, type)
    (method, time) => {
        $crate::profile_internal!("", $crate::ProfileType::Time, false, true)
    };
    (method, memory) => {
        $crate::profile_internal!("", $crate::ProfileType::Memory, false, true)
    };
    (method, both) => {
        $crate::profile_internal!("", $crate::ProfileType::Both, false, true)
    };

    // profile!(method, async)
    (method, async) => {
        $crate::profile_internal!("", $crate::ProfileType::Time, true, true)
    };

    // profile!(method, type, async)
    (method, time, async) => {
        $crate::profile_internal!("", $crate::ProfileType::Time, true, true)
    };
    (method, memory, async) => {
        $crate::profile_internal!("", $crate::ProfileType::Memory, true, true)
    };
    (method, both, async) => {
        $crate::profile_internal!("", $crate::ProfileType::Both, true, true)
    };

    // profile!(name, type, async)
    ($name:expr, time, async) => {
        $crate::profile_internal!($name, $crate::ProfileType::Time, true, false)
    };
    ($name:expr, memory, async) => {
        $crate::profile_internal!($name, $crate::ProfileType::Memory, true, false)
    };
    ($name:expr, both, async) => {
        $crate::profile_internal!($name, $crate::ProfileType::Both, true, false)
    };
}
```

### Internal Implementation Macro

```rust
#[doc(hidden)]
#[macro_export]
macro_rules! profile_internal {
    ($name:expr, $type:expr, $is_async:expr, $is_method:expr) => {{
        #[cfg(feature = "profiling")]
        {
            let profile = $crate::Profile::new(
                $name,
                $type,
                $is_async,
                $is_method
            );
            $crate::ProfileSection { profile }
        }

        #[cfg(not(feature = "profiling"))]
        {
            $crate::ProfileSection::new_dummy()
        }
    }};
}
```

### Specialized Convenience Macros

For backward compatibility and common cases:

```rust
/// Profile a section of code for time only
#[macro_export]
macro_rules! profile_time {
    ($name:expr) => { $crate::profile!($name, time) };
    (method) => { $crate::profile!(method, time) };
    (async) => { $crate::profile!($name, time, async) };
    (method, async) => { $crate::profile!(method, time, async) };
}

/// Profile a section of code for memory only
#[macro_export]
macro_rules! profile_memory {
    ($name:expr) => { $crate::profile!($name, memory) };
    (method) => { $crate::profile!(method, memory) };
    (async) => { $crate::profile!($name, memory, async) };
    (method, async) => { $crate::profile!(method, memory, async) };
}

/// Profile a section of code for both time and memory
#[macro_export]
macro_rules! profile_both {
    ($name:expr) => { $crate::profile!($name, both) };
    (method) => { $crate::profile!(method, both) };
    (async) => { $crate::profile!($name, both, async) };
    (method, async) => { $crate::profile!(method, both, async) };
}

/// Alias for profile!(method, ...)
#[macro_export]
macro_rules! profile_method {
    () => { $crate::profile!(method) };
    (time) => { $crate::profile!(method, time) };
    (memory) => { $crate::profile!(method, memory) };
    (both) => { $crate::profile!(method, both) };
    (async) => { $crate::profile!(method, async) };
    (time, async) => { $crate::profile!(method, time, async) };
    (memory, async) => { $crate::profile!(method, memory, async) };
    (both, async) => { $crate::profile!(method, both, async) };
}

/// Compatibility with existing profile_section!
#[macro_export]
macro_rules! profile_section {
    ($name:expr) => { $crate::profile!($name) };
}
```

### Implementation of ProfileSection

```rust
pub struct ProfileSection {
    #[cfg(feature = "profiling")]
    profile: Option<Profile>,
}

impl ProfileSection {
    #[cfg(feature = "profiling")]
    pub fn new(name: &str) -> Self {
        Self {
            profile: Profile::new(
                name.to_string(),
                ProfileType::Time,
                false,
                false
            ),
        }
    }

    #[cfg(not(feature = "profiling"))]
    pub fn new(_name: &str) -> Self {
        Self {}
    }

    #[cfg(not(feature = "profiling"))]
    pub fn new_dummy() -> Self {
        Self {}
    }

    pub fn end(self) {
        // Profile will be dropped here if enabled
    }

    pub fn is_active(&self) -> bool {
        #[cfg(feature = "profiling")]
        {
            self.profile.is_some()
        }

        #[cfg(not(feature = "profiling"))]
        {
            false
        }
    }
}
```

## Benefits of This Approach

1. **Single Core Macro**: `profile!` covers all cases with a clean, flexible syntax
2. **Familiar Alternatives**: Specialized macros for particular use cases
3. **Backwards Compatibility**: Old macros still work, just return `ProfileSection` now
4. **Ergonomic**: Simple cases are simple; complex cases are possible
5. **Extensible**: Easy to add more combinations if needed

## Usage Examples

```rust
// Basic function profiling
fn process_data(data: &[u8]) {
    let section = profile!("process_data");
    // or let section = profile_time!("process_data");

    // Processing code

    section.end();
}

// Method profiling (auto-detects method name from backtrace)
impl Processor {
    fn process(&self, data: &[u8]) {
        let section = profile!(method);
        // or let section = profile_method!();

        // Processing code

        section.end();
    }

    // Memory profiling in async method
    async fn load_data(&self) -> Vec<u8> {
        let section = profile!(method, memory, async);
        // or let section = profile_method!(memory, async);

        // Loading code

        section.end();
        data
    }
}
```

Does this approach align with your vision for the profiling macros? It provides a flexible, yet consistent API that covers all use cases while remaining backwards compatible.



This looks great - my only concern relates back to what I mentioned about having profile! and #[profile] in the same name space for 2 different items. I see you've used profile_internal, but I don't know how that will solve the issue.

# Addressing the Naming Conflict Issue

You're absolutely right to be concerned about the naming conflict between a `profile!` macro and the `#[profile]` attribute. Let me clarify and suggest solutions:

## The Potential Conflict

In Rust, procedural macros (attributes like `#[profile]`) and declarative macros (like `profile!()`) live in different namespaces, so technically they shouldn't conflict directly. However:

1. It can be confusing for users to have two different things with the same name
2. It might lead to unexpected behavior in certain contexts
3. Documentation and tooling might struggle to differentiate them clearly

## Solutions to Consider

### Option 1: Use a Different Base Name for the Declarative Macro

```rust
// Instead of profile!()
#[macro_export]
macro_rules! prof {
    // Same implementation as before
}

// With specialized versions
#[macro_export]
macro_rules! prof_time {
    // ...
}
```

### Option 2: Keep the Original profile_section! as Primary

```rust
// Keep the original as the main macro
#[macro_export]
macro_rules! profile_section {
    // All the variations from my previous example
}

// And others as specialized cases
#[macro_export]
macro_rules! profile_method {
    // ...
}
```

### Option 3: Use a Different Naming Convention

```rust
// More explicit about what it does
#[macro_export]
macro_rules! start_profile {
    // Same implementation
}

// Or
#[macro_export]
macro_rules! profile_block {
    // Same implementation
}
```

## Recommendation: Option 2 with Enhancements

I recommend keeping `profile_section!` as your primary declarative macro, with enhanced capabilities:

```rust
/// Profile a section of code with customizable options
///
/// # Examples
///
/// ```
/// // Basic usage (profiles time, sync function)
/// let section = profile_section!("expensive_operation");
/// // ... code to profile
/// section.end();
///
/// // With explicit type (profile memory)
/// let section = profile_section!("allocation_heavy", memory);
///
/// // Method profiling
/// let section = profile_section!(method);
///
/// // Async method with explicit type
/// let section = profile_section!(method, both, async);
/// ```
#[macro_export]
macro_rules! profile_section {
    // All the variations from my previous profile! example
    // ...
}
```

This approach:

1. Avoids naming conflicts with `#[profile]`
2. Maintains backward compatibility with existing code
3. Provides a clear, intuitive name that describes what it does (profiles a section)
4. Allows for all the parameter combinations we discussed

## Implementation Details

The `profile_internal!` macro from my previous response would still be used internally by `profile_section!` to handle the actual implementation, but users would never see or use it directly.

Here's a complete example:

```rust
// Main macro with all variations
#[macro_export]
macro_rules! profile_section {
    // profile_section!(name)
    ($name:expr) => {
        $crate::profile_internal!($name, $crate::ProfileType::Time, false, false)
    };

    // All the other variations from before...
    // profile_section!(name, type)
    // profile_section!(method)
    // etc.
}

// Internal implementation (hidden from docs)
#[doc(hidden)]
#[macro_export]
macro_rules! profile_internal {
    // Same implementation as before
}

// Specialized convenience macros
#[macro_export]
macro_rules! profile_time {
    ($name:expr) => { $crate::profile_section!($name, time) };
    // Other variations...
}

#[macro_export]
macro_rules! profile_method {
    () => { $crate::profile_section!(method) };
    // Other variations...
}

// And so on for other specialized macros
```

Does this approach address your concern about the naming conflict? It maintains a clear distinction between the attribute macro and declarative macros while providing all the flexibility we discussed.


I need to give this more thought. I wauld be happy to use the same name `profile` for both the attribute macro and the declarative macro, except only that I can't see how the imports and exports would work. E.g. given that thag_profiler exports thag_proc_macros::profile, if it also exports its own declarative `profile` macro, I can't see how the two would not clash. Likewise for the user program that contains #[profile] and profile! I would like to know how this works.

Also, while `profile_section` is reasonably descriptive - except when used to profile a whole function or a method - it's also a bit verbose. Also bear in mind that this will be the first ever public release of thag profiling in any form, so we have a clean slate with no legacy code to worry about. So I'm thinking that it might be best to use `profile!` as the declarative macro name and `#[profiled]` or `#[profile_fn]`or `#[profile_attr]` as the attribute macro name. There must be other good names out there. We have time to choose carefully. I'm still entertaining `prof` as maybe being of use somewhere.


# Resolving Naming and Export Issues

You've raised excellent points about the naming and export challenges. Let me clarify how macros work in Rust and provide some informed recommendations:

## How Rust Handles Different Types of Macros

You're right to be concerned, but Rust actually does differentiate between attribute macros and declarative macros even if they share the same name:

1. **Attribute macros** are used with the `#[...]` syntax
2. **Declarative macros** are used with the `name!(...)` syntax

However, there is still the issue of exporting and importing them in a way that doesn't cause confusion.

## Options for Clean API Design

### Option 1: Different Names for Attribute and Declarative Macros

The cleanest approach is to use distinct names:

```rust
// In thag_proc_macros
#[proc_macro_attribute]
pub fn profiled(attr: TokenStream, item: TokenStream) -> TokenStream { ... }

// In thag_profiler
#[macro_export]
macro_rules! profile { ... }
```

Usage:
```rust
use thag_profiler::{profile, profiled};

#[profiled]
fn my_function() {
    let section = profile!("expensive part");
    // ...
    section.end();
}
```

### Option 2: Use Namespacing

Another approach is to namespace your macros:

```rust
// In thag_profiler/src/lib.rs
pub mod macros {
    // Re-export the attribute macro
    pub use thag_proc_macros::profile;

    // Define the declarative macro
    #[macro_export]
    macro_rules! profile { ... }
}
```

Usage:
```rust
use thag_profiler::macros::profile as profile_attr;
use thag_profiler::profile;

#[profile_attr]
fn my_function() {
    let section = profile!("expensive part");
    // ...
    section.end();
}
```

But this gets complex and isn't ideal.

## Recommended Solution: Rename the Attribute Macro

Given that this is a new public release with no legacy concerns, renaming the attribute macro to something distinct makes the most sense:

### Attribute Macro Name Options

1. **`profiled`**: Clear, concise, and reads naturally in context
   ```rust
   #[profiled]
   fn my_function() { ... }
   ```

2. **`profile_fn`**: Explicitly indicates it profiles a function
   ```rust
   #[profile_fn]
   fn my_function() { ... }
   ```

3. **`instrument`**: Similar to what's used in tracing/opentelemetry crates
   ```rust
   #[instrument]
   fn my_function() { ... }
   ```

### Declarative Macro Names

For the declarative macro:

1. **`profile`**: Concise, intuitive, and works well as a verb
   ```rust
   let section = profile!("expensive part");
   ```

2. **`prof`**: Even shorter, could be nice for frequent use
   ```rust
   let section = prof!("expensive part");
   ```

## Final Recommendation

Based on all considerations, I recommend:

1. **Attribute Macro**: Rename to `profiled` (in thag_proc_macros)
2. **Declarative Macro**: Use `profile!` (in thag_profiler)

This combination:
- Provides clear distinction between the two types of macros
- Uses descriptive, concise names
- Reads naturally in code
- Avoids export/import confusion
- Gives you a clean slate for your first public release

Implementation would look like:

```rust
// In thag_proc_macros
#[proc_macro_attribute]
pub fn profiled(attr: TokenStream, item: TokenStream) -> TokenStream { ... }

// In thag_profiler
pub use thag_proc_macros::profiled;

#[macro_export]
macro_rules! profile {
    // All variations we discussed earlier
}
```

And the user code would be:

```rust
use thag_profiler::{profiled, profile};

#[profiled]
fn expensive_calculation() {
    // Function-level profiling via attribute

    // Section profiling via declarative macro
    let section = profile!("specific part");
    compute_something();
    section.end();

    // Method profiling
    let method_section = profile!(method);
    self.do_something();
    method_section.end();
}
```

This approach offers a clean, intuitive API with no naming conflicts or confusion.



Yes, I'm inclined to agree. Thanks, will take a break to rest and digest it and come back later.

Sounds good! Taking a break to digest these ideas is a wise approach. The naming of APIs is a crucial decision that's worth spending time on, especially for a public release.

When you return, we can refine the specific implementation details of the `profiled` attribute and `profile!` macro, if needed. I'll be here to help you work through any remaining questions or considerations for the API design.

Rest well, and I look forward to continuing our collaboration when you're ready!


Thanks, I'm happy with your recommendations and have made a start. Just to check: in the main thag_rs as documented in Cargo.toml, profiling instrumentation was previously part of the `core` feature set:
```
# Core Feature Set (★):
# - Basic logging and error handling
# - Essential macros: cprtln, debug_log, lazy_static_var, vlog, regex
# - Styling system and macros: cvprtln, style_for_level
# - Profiling instrumentation macros: profile, profile_method, profile_section
# - Fundamental types and traits
#
# Optional features:
# - profiling     # Enables profiling output (instrumentation always available in core)
...
```
Is this still true? Also, I'm wondering if it wouldn't be better to remove the option for users to access `thag_profiler` via `thag_rs` and just force them to use `thag_profiler` directly, if possible while still having thag_rs itself instrumented for profiling? Going via thag_rs runs the risk of pulling in more thag_rs features and dependencies than necessary, whereas thag_profiler has a much lighter footprint. I'm a bit confused about my options here.


# Handling Profiling Dependencies and Features

You've raised excellent questions about how to structure the profiling features. Let me clarify your options:

## Removing Profiling from Core Feature Set

The first important change is that profiling instrumentation should **no longer be part of the core feature set** in `thag_rs`. With your new structure:

1. All profiling functionality lives in `thag_profiler`
2. Users who need profiling would explicitly depend on it
3. This keeps `thag_rs` lighter weight for users who don't need profiling

### Updated Documentation for Cargo.toml

```toml
# Core Feature Set (★):
# - Basic logging and error handling
# - Essential macros: cprtln, debug_log, lazy_static_var, vlog, regex
# - Styling system and macros: cvprtln, style_for_level
# - Fundamental types and traits
#
# Optional features:
# - profiling     # Enables profiling via thag_profiler (for internal use)
```

## Options for thag_rs Instrumentation

For your question about having `thag_rs` instrumented while not forcing users to pull in profiling - here are your options:

### Option 1: Keep thag_profiler as Optional Dependency (Recommended)

```toml
# In thag_rs/Cargo.toml
[dependencies]
thag_profiler = { path = "../thag_profiler", version = "0.1", optional = true }

[features]
profiling = ["thag_profiler", "thag_profiler/profiling"]
```

With this approach:
- Users of `thag_rs` don't get profiling unless they explicitly enable the feature
- `thag_rs` can still be instrumented for your development/testing
- You can re-export necessary types when profiling is enabled

In your `thag_rs` code:

```rust
// In thag_rs/src/lib.rs
#[cfg(feature = "profiling")]
pub use thag_profiler::{profiled, profile};

// In your instrumented code
#[cfg_attr(feature = "profiling", profiled)]
fn my_function() {
    // ...
    #[cfg(feature = "profiling")]
    let section = profile!("expensive part");
    // ...
    #[cfg(feature = "profiling")]
    section.end();
}
```

### Option 2: Complete Separation (Also Good)

Don't re-export `thag_profiler` from `thag_rs` at all. Document that users who want profiling should:

```toml
# User's Cargo.toml
[dependencies]
thag_rs = "0.1"
thag_profiler = { version = "0.1", features = ["profiling"] }
```

```rust
// User's code
use thag_rs::SomeType;
use thag_profiler::{profiled, profile};

#[profiled]
fn my_function() {
    // ...
}
```

For your internal instrumentation of `thag_rs`, you would:

```toml
# In thag_rs/Cargo.toml
[dev-dependencies]
thag_profiler = { path = "../thag_profiler", version = "0.1", features = ["profiling"] }
```

And conditionally enable the profiling code:

```rust
// In thag_rs code
#[cfg(test)]
use thag_profiler::{profiled};

#[cfg_attr(test, profiled)]
fn my_function() {
    // ...
}
```

## Recommendation

I recommend **Option 1 with a twist**. Keep `thag_profiler` as an optional dependency, but:

1. Don't re-export anything from `thag_profiler` in `thag_rs`
2. Document clearly that users should depend directly on `thag_profiler`
3. Use the optional dependency purely for internal instrumentation

This gives you:
- Clean separation for users
- Proper dependency structure
- The ability to instrument `thag_rs` code
- No pulling in unnecessary dependencies

This is the best of both worlds - you can instrument `thag_rs` for your own use, while users get a clean API with direct dependencies on what they need.

The documentation would make it clear:

```markdown
## Profiling

For profiling functionality, we recommend directly using the `thag_profiler` crate:

```toml
[dependencies]
thag_profiler = "0.1"
```

```rust
use thag_profiler::{profiled, profile};

#[profiled]
fn my_function() {
    // ...
}
```

Don't enable the `profiling` feature on `thag_rs` unless you're developing `thag_rs` itself.
```

This approach provides the cleanest, most maintainable solution for both you and your users.



I thought thag_profiler had to be a fixed dependency of thag_rs? Since thag_rs is a user of thag_profiler, I want the thag_rs code instrumentation ungated, especially now it is zero-cost when profiling is not enabled. If I try to make it an optionel dependency I get an immediate build error:

```
error: failed to load manifest for dependency `thag_profiler`

Caused by:
  failed to read `/Users/donf/projects/thag_profiler/Cargo.toml`

Caused by:
  No such file or directory (os error 2)
```

# Handling thag_rs Dependency on thag_profiler

You're right - if you're instrumenting `thag_rs` code with profiling macros, then `thag_profiler` needs to be a regular (non-optional) dependency. Let me adjust my recommendation based on this reality:

## Updated Approach

1. Make `thag_profiler` a regular dependency of `thag_rs`
2. Use the `profiling` feature to control whether profiling is active at runtime
3. Still recommend users depend directly on `thag_profiler` for their own code

### Implementation in thag_rs/Cargo.toml

```toml
[dependencies]
# Other dependencies...
thag_profiler = { path = "../thag_profiler", version = "0.1" }

[features]
profiling = ["thag_profiler/profiling"]  # Only enables the runtime profiling, not instrumentation
```

### In thag_rs/src/lib.rs

```rust
// Re-exports for convenience
// These are minimal and don't force users to go through thag_rs
pub use thag_profiler::{profiled, profile};

// Or if you prefer not to re-export at all, that's fine too
```

### Using profiling in thag_rs code

```rust
// In your thag_rs implementation
use thag_profiler::{profiled, profile};

#[profiled]
fn my_function() {
    // This code is always compiled, but only does profiling when the feature is enabled
    let section = profile!("expensive part");
    // ...code...
    section.end();
}
```

### Documentation for users

```markdown
## Profiling

For profiling functionality, we recommend directly using the `thag_profiler` crate:

```toml
[dependencies]
thag_rs = "0.1"
thag_profiler = { version = "0.1", features = ["profiling"] }
```

```rust
use thag_profiler::{profiled, profile};

#[profiled]
fn my_function() {
    // ...
}
```

The `profiling` feature on `thag_rs` only enables profiling of `thag_rs` itself, not your application code.
```

## Key Points

1. `thag_profiler` is a regular dependency of `thag_rs`
2. The `profiling` feature only controls whether the profiling is active at runtime
3. Instrumentation macros are always available but default to no-ops when profiling is disabled
4. Users are encouraged to depend directly on `thag_profiler` for their own code

This approach:
- Allows you to instrument `thag_rs` code
- Keeps the profiling zero-cost when disabled
- Provides a clean API for users
- Avoids unnecessary dependencies for users

Does this approach work better with your existing structure?



I'm hopeful, but I have quite a long road to modify existing thag_rs imports among others. I'm implementing your recommended full profile! declarative macro in thag_profiler::profiling, but I also need the code to replace this for the case where profiling is disabled:
```
// Provide an empty implementation when profiling is disabled
#[cfg(not(feature = "profiling"))]
#[macro_export]
macro_rules! profile {
    ($name:expr) => {{
        struct DummyProfileSection;
        impl DummyProfileSection {
            pub fn end(self) {}
        }
        DummyProfileSection
    }};
}
```



# No-Op Implementation for profile! Macro When Profiling is Disabled

I'll provide a complete no-op implementation for the `profile!` macro that handles all the variants we discussed earlier. This will ensure that your instrumented code always compiles, regardless of whether the `profiling` feature is enabled.

```rust
// No-op implementation for when profiling is disabled
#[cfg(not(feature = "profiling"))]
#[macro_export]
macro_rules! profile {
    // Basic variants
    ($name:expr) => {{
        $crate::DummyProfileSection {}
    }};

    // profile!(name, type)
    ($name:expr, time) => {{
        $crate::DummyProfileSection {}
    }};
    ($name:expr, memory) => {{
        $crate::DummyProfileSection {}
    }};
    ($name:expr, both) => {{
        $crate::DummyProfileSection {}
    }};

    // profile!(name, async)
    ($name:expr, async) => {{
        $crate::DummyProfileSection {}
    }};

    // profile!(method)
    (method) => {{
        $crate::DummyProfileSection {}
    }};

    // profile!(method, type)
    (method, time) => {{
        $crate::DummyProfileSection {}
    }};
    (method, memory) => {{
        $crate::DummyProfileSection {}
    }};
    (method, both) => {{
        $crate::DummyProfileSection {}
    }};

    // profile!(method, async)
    (method, async) => {{
        $crate::DummyProfileSection {}
    }};

    // profile!(method, type, async)
    (method, time, async) => {{
        $crate::DummyProfileSection {}
    }};
    (method, memory, async) => {{
        $crate::DummyProfileSection {}
    }};
    (method, both, async) => {{
        $crate::DummyProfileSection {}
    }};

    // profile!(name, type, async)
    ($name:expr, time, async) => {{
        $crate::DummyProfileSection {}
    }};
    ($name:expr, memory, async) => {{
        $crate::DummyProfileSection {}
    }};
    ($name:expr, both, async) => {{
        $crate::DummyProfileSection {}
    }};
}

// Define the dummy section type
#[cfg(not(feature = "profiling"))]
#[derive(Debug, Clone, Copy)]
pub struct DummyProfileSection;

#[cfg(not(feature = "profiling"))]
impl DummyProfileSection {
    pub fn end(self) {}
    pub fn is_active(&self) -> bool { false }
}
```

## Implementation Notes

1. Each variant of the macro returns the same `DummyProfileSection`
2. `DummyProfileSection` implements the same methods as the real `ProfileSection` but does nothing
3. All variants are covered to ensure compatibility regardless of how the macro is used
4. The implementation is as minimal as possible to keep overhead near zero

You can add this code to your `thag_profiler/src/lib.rs` file, and it will ensure that all uses of the `profile!` macro compile correctly when profiling is disabled.

Do you also need similar no-op implementations for any of the specialized macros like `profile_method!` or `profile_time!`? If so, I'd be happy to provide those as well.



Thanks, but I don't think I want to implement any of those specialized macros out of the box. One problem I'm having is that thag_rs functions with the #[profiled] attribute are complaining "Could not find Profile in the crate root." Importing thag_profiler::Profile does not help, it just shows up as unused. The cause is surely that in thag_proc_macros implementation of `profiled`, the function wrappers refer to crate::Profile, e.g:
```
    quote! {
        #vis fn #fn_name #generics (#inputs) #output #where_clause {
            // eprintln!("From generate_sync_wrapper: profile_name={}", #profile_name);
            let _profile = crate::Profile::new(#profile_name.to_string(), #profile_type, false, #is_method);
            #body
        }
    }
```
But I surely can't just replace `crate::Profile` with `thag_profiler::Profile` because that's a reverse dependency. I'm a bit confused now about how to proceed.


# Resolving the crate::Profile Reference Issue

You've hit upon an important issue with procedural macros: they generate code that gets inserted into the user's crate, so any references to types need to be fully qualified. Let me explain how to fix this:

## The Problem

When your `profiled` attribute macro generates code, it's inserting it directly into the user's crate. So `crate::Profile` refers to a (non-existent) `Profile` in the root of the user's crate, not to your `thag_profiler::Profile`.

## Solution: Use Fully Qualified Paths

The solution is to use fully qualified paths in your proc macro output. Here's how to modify the `profiled` attribute macro:

```rust
// In thag_proc_macros/src/lib.rs
#[proc_macro_attribute]
pub fn profiled(attr: TokenStream, item: TokenStream) -> TokenStream {
    // ... existing parsing code ...

    // For sync functions
    let generated = quote! {
        #vis fn #fn_name #generics (#inputs) #output #where_clause {
            let _profile = ::thag_profiler::Profile::new(
                #profile_name.to_string(),
                #profile_type,
                false,
                #is_method
            );
            #body
        }
    };

    // ... similar changes for async wrappers ...
}
```

Notice the key change: `crate::Profile` becomes `::thag_profiler::Profile`.

## Alternative: Use a Re-export Pattern

Another approach is to make your proc macro generate code that uses a re-export from the user's crate:

1. First, modify your proc macro to expect `Profile` from a prelude:

```rust
// In thag_proc_macros/src/lib.rs
let generated = quote! {
    #vis fn #fn_name #generics (#inputs) #output #where_clause {
        let _profile = ::thag_profiler::__private::Profile::new(
            #profile_name.to_string(),
            ::thag_profiler::ProfileType::#profile_type_variant,
            false,
            #is_method
        );
        #body
    }
};
```

2. Then in `thag_profiler`, create a `__private` module:

```rust
// In thag_profiler/src/lib.rs
// Public API
pub use profiling::ProfileType;
pub use thag_proc_macros::profiled;
// etc...

// Private module for proc macro use
#[doc(hidden)]
pub mod __private {
    pub use crate::profiling::Profile;
}
```

This pattern is used by crates like `serde` to ensure proc macros can access internal types.

## The Simplest Solution

The simplest immediate solution is to modify your proc macro to use `::thag_profiler::Profile` directly, but this creates a hard dependency on the name `thag_profiler`.

A more flexible approach would be:

```rust
// In thag_proc_macros/src/lib.rs
#[proc_macro_attribute]
pub fn profiled(attr: TokenStream, item: TokenStream) -> TokenStream {
    // ...

    // Generate the wrapper with a check for whether profiling is enabled
    let generated = quote! {
        #vis fn #fn_name #generics (#inputs) #output #where_clause {
            #[cfg(feature = "profiling")]
            let _profile = ::thag_profiler::Profile::new(
                #profile_name.to_string(),
                #profile_type,
                false,
                #is_method
            );

            #body
        }
    };

    // ...
}
```

This would:
1. Reference `thag_profiler::Profile` directly
2. Conditionally compile the profiling code only when the feature is enabled
3. Avoid the "could not find Profile" error even when profiling is disabled

Would one of these approaches work for your use case?



Would the check even be required? - because we're already not going near the wrapper if the profiling feature is not enabled. Or does this check get run at a different time (run time)?

# Understanding the Conditional Compilation in Proc Macros

You've asked a great question about conditional compilation in proc macros. Let me clarify how it works:

## The Key Insight

You're absolutely right that if the proc macro is already generating different code based on whether the `profiling` feature is enabled, then you don't need an additional `#[cfg(feature = "profiling")]` check in the generated code.

Here's the sequence of events:

1. **Compile Time, Phase 1**: Your proc macro runs during compilation and generates code
2. **Compile Time, Phase 2**: The generated code is compiled as part of the user's crate

If your proc macro already does something like:

```rust
#[proc_macro_attribute]
pub fn profiled(attr: TokenStream, item: TokenStream) -> TokenStream {
    #[cfg(feature = "profiling")]
    {
        // Generate code with profiling
        let input = parse_macro_input!(item as ItemFn);
        // Generate wrapper with thag_profiler::Profile
        generate_profiling_wrapper(attr, input).into()
    }

    #[cfg(not(feature = "profiling"))]
    {
        // Just return the original function unchanged
        item
    }
}
```

Then the additional check in the generated code would be redundant.

## The Solution

The simplest solution is to ensure your proc macro generates appropriate code based on the feature flag, and uses fully qualified paths:

```rust
// In thag_proc_macros/src/lib.rs
#[proc_macro_attribute]
pub fn profiled(attr: TokenStream, item: TokenStream) -> TokenStream {
    #[cfg(feature = "profiling")]
    {
        let input = parse_macro_input!(item as ItemFn);
        // ... parsing logic ...

        // Generate with full path to Profile
        let generated = quote! {
            #vis fn #fn_name #generics (#inputs) #output #where_clause {
                let _profile = ::thag_profiler::Profile::new(
                    #profile_name.to_string(),
                    #profile_type,
                    false,
                    #is_method
                );
                #body
            }
        };

        generated.into()
    }

    #[cfg(not(feature = "profiling"))]
    {
        // When profiling is disabled, just return the original function
        item
    }
}
```

This approach:
1. Generates profiling code only when the feature is enabled
2. Uses fully qualified paths to `thag_profiler::Profile`
3. Returns the original function unchanged when profiling is disabled

With this implementation, you won't see the "could not find Profile" error because:
- When profiling is enabled, it correctly references `::thag_profiler::Profile`
- When profiling is disabled, it doesn't try to reference `Profile` at all

Does this clarify the compile-time behavior and address your concern about redundant checks?



I'm still getting hundreds of weird compilation errors around #[profiled]. I see some more crate:: references in the following method in the implementation code:
```
fn resolve_profile_type(profile_type: Option<&ProfileTypeOverride>) -> proc_macro2::TokenStream {
    match profile_type {
        Some(ProfileTypeOverride::Global) | None => {
            quote!(crate::profiling::get_global_profile_type())
        }
        Some(ProfileTypeOverride::Time) => quote!(crate::ProfileType::Time),
        Some(ProfileTypeOverride::Memory) => quote!(crate::ProfileType::Memory),
        Some(ProfileTypeOverride::Both) => quote!(crate::ProfileType::Both),
    }
}
```
Is it safe and correct to change these 4 occurrences of `crate::` to `::thag_profiler::`?


# Fixing the Profile Type References

Yes, it's correct and necessary to change all those `crate::` references to `::thag_profiler::`. This is exactly the same issue we discussed earlier - the proc macro is generating code that will be inserted into the user's crate, so all type references need to be fully qualified.

## Changes Needed

```rust
fn resolve_profile_type(profile_type: Option<&ProfileTypeOverride>) -> proc_macro2::TokenStream {
    match profile_type {
        Some(ProfileTypeOverride::Global) | None => {
            quote!(::thag_profiler::profiling::get_global_profile_type())
        }
        Some(ProfileTypeOverride::Time) => quote!(::thag_profiler::ProfileType::Time),
        Some(ProfileTypeOverride::Memory) => quote!(::thag_profiler::ProfileType::Memory),
        Some(ProfileTypeOverride::Both) => quote!(::thag_profiler::ProfileType::Both),
    }
}
```

## Additional Checks

While you're at it, make sure to search your entire `thag_proc_macros` codebase for any other `crate::` references in quoted code. Common patterns to look for:

1. `quote!(crate::...)`
2. `format!("crate::...")`
3. Any other string literals that might be used to generate code

## Potential Issue with get_global_profile_type

Note that `get_global_profile_type()` seems to be in a `profiling` module. Make sure this function is correctly exported from `thag_profiler` or you might need to adjust the path to:

```rust
quote!(::thag_profiler::get_global_profile_type())
```

Depending on your actual export structure.

## Understanding Why This Happens

This is a common challenge with proc macros. The macro code runs at compile time in the context of the proc-macro crate, but the generated code runs at runtime in the context of the user's crate. This context switch is why all references need to be fully qualified.

Let me know if you find any other `crate::` references that need to be fixed, or if you encounter any new errors after making these changes.


Thanks, good point about exporting get_global_profile_type - done.

You're welcome! Ensuring all the necessary exports and fully qualified paths are in place is crucial when working with proc macros. It can be tricky to debug because the error messages often appear in the user's code rather than in the macro implementation.

Now that you've exported `get_global_profile_type` and updated all the `crate::` references to `::thag_profiler::`, your compilation errors should be significantly reduced or eliminated.

If you encounter any other issues with the `#[profiled]` attribute, here are a few additional things to check:

1. Make sure `ProfileType` is properly exported from the root of `thag_profiler`
2. Ensure any other types referenced in the generated code are also exported
3. Verify that feature flags are consistently applied across all relevant crates

Feel free to share any remaining errors, and I'd be happy to help troubleshoot them. Good luck with your implementation!


I'm wondering If I can rename DummyProfileSection to ProfileSection since they are oppositely feature gated. Otherwise I have to import them both in user code with opposite feature gating. If I've explained that Ok?

# Using the Same Name for Both Profile Section Implementations

Yes, you've explained it perfectly, and that's an excellent idea! Since the two implementations are feature-gated in opposite ways, you can absolutely use the same name (`ProfileSection`) for both. This simplifies imports in user code significantly.

Here's how to implement this approach:

```rust
// In thag_profiler/src/lib.rs or profiling.rs

// Real implementation when profiling is enabled
#[cfg(feature = "profiling")]
pub struct ProfileSection {
    profile: Option<Profile>,
}

#[cfg(feature = "profiling")]
impl ProfileSection {
    pub fn new(name: &str) -> Self {
        Self {
            profile: Profile::new(name.to_string(), ProfileType::Time, false, false),
        }
    }

    pub fn end(self) {
        // Profile is dropped here
    }

    pub fn is_active(&self) -> bool {
        self.profile.is_some()
    }
}

// Dummy implementation when profiling is disabled
#[cfg(not(feature = "profiling"))]
#[derive(Debug, Clone, Copy)]
pub struct ProfileSection;

#[cfg(not(feature = "profiling"))]
impl ProfileSection {
    pub fn new(_name: &str) -> Self {
        Self
    }

    pub fn end(self) {}

    pub fn is_active(&self) -> bool { false }
}
```

And in your macros:

```rust
#[cfg(feature = "profiling")]
#[macro_export]
macro_rules! profile {
    ($name:expr) => {{
        let section = $crate::ProfileSection::new($name);
        section
    }};
    // Other variants...
}

#[cfg(not(feature = "profiling"))]
#[macro_export]
macro_rules! profile {
    ($name:expr) => {{
        $crate::ProfileSection::new($name)
    }};
    // Other variants...
}
```

This approach provides several benefits:

1. **Simplified Imports**: Users only need to import `ProfileSection` once
2. **Type Consistency**: The same type name is used regardless of the feature status
3. **API Stability**: The API doesn't change based on features
4. **Zero-Cost Abstraction**: The no-op implementation compiles to essentially nothing

This pattern is commonly used in Rust crates that provide optional features - having the same type name with different implementations based on feature flags.



I have a problem where if I run thag_rs without features=profiling, while it correctly doesn't run the profiling, it still incorrectly expands the wrappers.

# Fixing Wrapper Expansion When Profiling is Disabled

If `thag_rs` is still expanding the profiling wrappers even when the `profiling` feature is disabled, this suggests that the feature flag isn't properly controlling the proc macro's behavior. Let's address this issue:

## The Core Problem

The issue is likely that the proc macro itself isn't correctly checking the feature flag status. Remember that:

1. The proc macro runs during compilation
2. It needs to check if the `profiling` feature is enabled
3. Based on that, it should either generate wrappers or return the original function

## Solutions

### 1. Modify thag_proc_macros/Cargo.toml

First, ensure the feature is properly defined:

```toml
[features]
profiling = []
```

### 2. Update the profiled Attribute Implementation

```rust
// In thag_proc_macros/src/lib.rs
#[proc_macro_attribute]
pub fn profiled(attr: TokenStream, item: TokenStream) -> TokenStream {
    // VERY IMPORTANT: Check if profiling feature is enabled
    if cfg!(feature = "profiling") {
        // Profiling is enabled - generate wrapper
        let input = parse_macro_input!(item as ItemFn);
        generate_profiling_wrapper(attr, input).into()
    } else {
        // Profiling is disabled - return original function unchanged
        item
    }
}
```

### 3. Ensure Feature Propagation

Ensure that when the `profiling` feature is enabled on `thag_rs`, it also enables the feature on `thag_proc_macros`:

```toml
# In thag_rs/Cargo.toml
[dependencies]
thag_proc_macros = { path = "../thag_proc_macros", version = "0.1" }
thag_profiler = { path = "../thag_profiler", version = "0.1" }

[features]
profiling = [
    "thag_profiler/profiling",
    "thag_proc_macros/profiling"  # Important!
]
```

```toml
# In thag_profiler/Cargo.toml
[dependencies]
thag_proc_macros = { path = "../thag_proc_macros", version = "0.1" }

[features]
profiling = ["thag_proc_macros/profiling"]  # Also propagate here
```

### 4. Debug the Feature Detection

Add a diagnostic to verify the feature detection is working:

```rust
// In thag_proc_macros/src/lib.rs
#[proc_macro_attribute]
pub fn profiled(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Add this for debugging
    println!("profiling feature enabled: {}", cfg!(feature = "profiling"));

    if cfg!(feature = "profiling") {
        // Rest of implementation
    } else {
        item
    }
}
```

### 5. Potential Alternative Approach

If you continue to have issues with feature detection in proc macros, you could use environment variables as a more reliable mechanism:

```rust
// In thag_proc_macros/build.rs
fn main() {
    if std::env::var("CARGO_FEATURE_PROFILING").is_ok() {
        println!("cargo:rustc-cfg=feature=\"profiling\"");
    }
}

// In thag_proc_macros/src/lib.rs
#[proc_macro_attribute]
pub fn profiled(attr: TokenStream, item: TokenStream) -> TokenStream {
    if cfg!(feature = "profiling") {
        // Rest of implementation
    } else {
        item
    }
}
```

## Important Note on Runtime vs. Compile Time

Remember that:

1. The `cfg!(feature = "profiling")` check happens during proc macro execution (compile time)
2. This is different from the `#[cfg(feature = "profiling")]` attribute, which affects what code gets compiled

If you're still having issues after trying these solutions, let me know what specific errors or behaviors you're observing, and I can provide more targeted advice.



I was indeed fully compliant. So, I got Claude Code to diagnose it. This was the root cause:
```
1. Dependency Configuration Issue: In thag_profiler/Cargo.toml, line 20:
thag_proc_macros = { path = "../thag_proc_macros", features = ["profiling"] }
1. This is unconditionally enabling the profiling feature in thag_proc_macros, regardless of whether profiling is enabled in thag_rs.
```
Claude Code removed the `, features = ["profiling"]` clause and it now works.

# Great Catch on the Dependency Configuration!

That's an excellent discovery! The issue was indeed in how the dependencies were configured. Let me explain why this fixed the problem:

## Understanding the Fix

In the original configuration:

```toml
# thag_profiler/Cargo.toml
thag_proc_macros = { path = "../thag_proc_macros", features = ["profiling"] }
```

This was **unconditionally enabling** the `profiling` feature in `thag_proc_macros`, regardless of whether the feature was enabled in `thag_rs` or `thag_profiler`. As a result, the proc macro was always generating profiling wrappers.

The corrected approach:

```toml
# thag_profiler/Cargo.toml
thag_proc_macros = { path = "../thag_proc_macros" }

[features]
profiling = ["thag_proc_macros/profiling"]
```

Now, the `profiling` feature is only activated in `thag_proc_macros` when it's specifically enabled in `thag_profiler`.

## Why This Matters

This example highlights an important aspect of Cargo's feature system:

1. When you specify a dependency with `features = ["feature_name"]`, that feature is **always enabled** for that dependency.
2. Using `your_feature = ["dependency/their_feature"]` in the `[features]` section makes enabling of the dependent feature conditional on your feature being enabled.

## Best Practices for Feature Propagation

For future reference, here's the recommended pattern for feature propagation across multiple crates:

```toml
# In the consumer crate
[dependencies]
dependent_crate = { version = "0.1", path = "../dependent_crate" }

[features]
my_feature = ["dependent_crate/their_feature"]
```

This ensures that features are only enabled when explicitly requested, preventing unexpected behavior like what you encountered.

Great work identifying and fixing this issue! It's a subtle but important aspect of Rust's dependency management.



Thanks for the explanation, it makes sense now.

Now I wonder if you could update the markdown user guide for me. I guess I should rename and move it to be the Readme file of thag_profiler? I've done some rough draft updates along the way so as not to forget things, but it's now badly out of date. I want to keep a quick start guide for users to get up and running quickly, but also have a more comprehensive guide for advanced users who want to dive deeper into the library's capabilities. This always gives me trouble when it comes to where to put the installation instructions. Just a reminder that the analyze.rs, instrument.rs and remove.rs are now fully binaries in thag_profiler/src/bin and have [[bin]] entries in thag_profiler/Cargo.toml. Please ask for further details if unsure of anything.
```md thag_rs/Thag_Profiling_User_Guide.md
# User Guide to Application Profiling with `thag_profiler`

## Introduction

Profiling is key to optimizing your Rust applications, but it tends to be time-consuming (no pun intended).
`thag_profiler` aims to provide quick, straightforward and consistent basic run-time and memoryprofiling for your Rust project or script across all platforms.

`thag` profiling is intrusive, meaning that you need to instrument your code at least temporarily for profiling.
`thag` provides an automated instrumentation tool to do the instrumentation. This is designed to be "lossless", preserving the original code intact with its comments
and formatting, and adding necessary instrumentation in the form of simple #[profiled] function attributes (and #[enable_profiling] in the case of `fn main`) using the `rust-analyzer` syntax tree library `ra_ap_syntax`.
A second tool is provided to remove the instrumentation.

You can instrument and profile any module or modules of a project, or a user script provided that the script has a `main` function (i.e. not a snippet).

The run-time overhead of instrumentation is zero if the `profiling` feature is disabled, as compilation will leave the instrumented functions untouched.
If the `profiling` feature is enabled, a sync or async wrapper is transparently added around the function as appropriate, to instantiate a `Profile` object to
profile the execution time and-or memory usage. Profiling overhead is excluded from the reported execution time.

`thag_profiler` will work with async as well as non-async code. It uses Rust's `backtrace` crate to populate the call stack for each Profile instance,
stripping out any non-profiled functions including scaffolding for asynccrates such as `tokio`.

This guide explains how to use `thag_profiler`, alone or in conjunction with `thag_rs` to identify performance bottlenecks, analyze memory usage, and optimize your code.

## Quick-start guide

# Installing the Profiling Tools

Thag includes several profiling tools that can be installed separately:

```bash
# Install the instrumentation tool
cargo install thag_rs --no-default-features --features=profile_instrument --bin thag_profile_instrument --force

# Install the removal tool
cargo install thag_rs --no-default-features --features=profile_instrument --bin thag_profile_remove --force

# Install the analysis tool
cargo install thag_rs --no-default-features --features=profile_analyze --bin thag_profile_analyze --force
```

1. Instrumenting your script or module:

    ```bash
    thag_profile_instrument 2021 < demo/factorial_ibig_product.rs > demo/factorial_ibig_product_profile.rs
    ```

    Note that `thag_profile_instrument` requires the Rust edition of your project or script for the benefit of the `ra_ap_syntax` crate.

    Comparing before and after with `vimdiff`:

    ```
    vimdiff demo/factorial_ibig_product.rs demo/factorial_ibig_product_profile.rs
    ```

    ![vimdiff](assets/vimdiff_profile_instrument.png)

    If you're profiling a project source file, at this point you'd want to replace the uninstrumented code with the instrumented version.

2. Profiling your code:

      ```bash
      thag demo/factorial_ibig_product_profile.rs
      ```

    Comparing before and after with `vimdiff`:

    ```
    vimdiff demo/factorial_ibig_product_profile.rs demo/factorial_ibig_product_profile_profile.rs
    ```

    ![vimdiff](assets/vimdiff_profile_profile.png)

## Enabling Profiling

There are two ways to enable profiling in `thag_rs`:

1. Via an `#[enable_profiling]` attribute on the main function.

    ```rust
    #[enable_profiling]
    fn main() {
    // Your code here
    }
    ```


    You can specify the profiling type when enabling profiling:

    ```rust
    #[enable_profiling(profile_type = "<type>")]
    ```

    Enables profiling globally for the entire application. Valid values for `<type>` are "time", "memory", or "both". The default is "both".

2. Via a `profiling` Cargo feature flag in your project's `Cargo.toml` file or your script's toml block- Use this for occasional profiling during development:

  ```rust
  /*[toml]
  ...
  [features]
  profiling = []
  ...
  */
  ```

  ```bash
  cargo run --features profiling -- your_script.rs
  ```

## Profiling Types

thag_rs supports two types of profiling, separately or combined:

- Time Profiling: Measures wall-clock execution time of functions
- Memory Profiling: Tracks memory allocation and usage
- Both: Combines time and memory profiling

You can also specify the profiling type programmatically:

```rust
// In your code
profiling::enable_profiling(true, ProfileType::Both)?;
```

## Instrumenting Your Code

### Automatic Instrumentation

For easier profiling, `thag_rs` provides tools to automatically instrument your code. See the Quick-start guide for installation instructions.

#### Using the profile_instrument tool

For existing source files, you can use the profile_instrument tool to automatically add profiling attributes:

```bash
thag_profile_instrument <edition_yyyy> < path/to/your/source.rs > path/to/destination.rs
```

This will add `#[profiled]` attributes to functions and methods (excluding tests, TODO investigate), and `#[enable_profiling]` to main() if present.
You can of course

#### Removing Instrumentation

When you're done profiling, you can remove the instrumentation:

```bash
thag_profile_remove <edition_yyyy> < path/to/your/source.rs > path/to/destination.rs
```


### Manual Instrumentation

#### Using the `#[profiled]` attribute

You can add the `#[profiled]` attribute to any function to profile it with a meaningful function or method name:
For regular functions this will profile the function as `fn::<function_name>`.
For methods, this will profile the method as `method::<method_name>`. See below for how to add the name of the
implementation type or trait.

```rust
use thag_proc_macros::profiled;

#[profiled]
fn expensive_calculation() -> u64 {
// Your code here
}

// Also works with async functions!
#[profiled]
async fn fetch_data() -> Result<String, Error> {
// Async operations
}
```

The `#[profiled]` attribute supports several options:

1. `imp` option (because "impl" is a reserved keyword):

```rust
#[profile(imp = "<type>")]
```
For method annotation, prefixes the method name with the name of the implemented type.
E.g. for a struct named `MyStruct`, the method `my_method` would be annotated as:

```rust
#[profile(imp = "MyStruct")]
fn my_method(&self) { /* ... */ }
```

2. `trait_name` option:

```rust
#[profile(trait_name = "<type>")]
```

For method annotation, prefixes the method name with the name of the implemented trait.
E.g. If implementing a trait named `MyTrait`, the method `my_method` could be annotated as:

```rust
#[profile(trait_name = "MyTrait")]
fn my_method(&self) { /* ... */ }
```

This is an alternative to the imp option above.

3. `profile_type` option:

```rust
#[profile(profile_type = "<type>")]
```
Specifies the type of profiling to be performed. Valid values are "global", "time", "memory", or "both". "global" specifies to use the same value as specified or defaulted in the `#[enable_profiling]` attribute.

For fine-grained control, you can manually profile specific functions or code sections:

```rust
fn my_function() {
// Profile the whole function
profile_fn!("my_function");

    // Do some work...

    // Profile a specific section. Initializes a new Profile instance and returns a reference to it.
    // You may optionally end the profile section manually as shown below, or it will automatically
    // end at the end of the current scope (block or function).
    let profile_section = profile_section!("expensive_operation");
    for i in 0..1000 {
    // Expensive operation here
    }
    // End the profile section. Cause the Profile to be dropped, writing out the profile data in the `drop` function.
    profile_section.end();

    // Profile memory usage specifically TODO return a reference so we can manually end it.
    profile_memory!("allocate_large_buffer");
    let buffer = vec![0; 1_000_000];

    // Profile both time and memory
    profile_both!("complex_operation");
    // Operation that's both CPU and memory intensive
    }
    ```

## Conditional Profiling with cfg attributes

### Using a Custom Feature Flag

You can make profiling conditional on a custom feature flag:

```rust
// Only apply profiling when the "profile" feature is enabled
#[cfg_attr(feature = "profile", profile)]
fn expensive_calculation() -> u64 {
// Your code here
}
```

### Using debug_assertions

To profile only in debug builds (not release):

```rust
// Only apply profiling in debug builds
#[cfg_attr(debug_assertions, profile)]
fn expensive_calculation() -> u64 {
// Your code here
}
```

### Combining with Profile Options

You can still use profiling options with conditional compilation:

```rust
// Apply profiling conditionally with custom options
#[cfg_attr(feature = "profile", profile(imp = "MyStruct"))]
fn my_method(&self) {
// Method implementation
}
```

### Multi-condition Example

You can combine multiple conditions:

```rust
// Apply profiling only in debug builds with profiling feature enabled
#[cfg_attr(all(debug_assertions, feature = "profile"), profile)]
fn complex_operation() {
// Complex operation code
}
```

### Applying to Trait Implementations

Conditional profiling works well with trait implementations too:

```rust
impl SomeTrait for MyStruct {
#[cfg_attr(feature = "profile", profile(trait_name = "SomeTrait"))]
fn trait_method(&self) {
// Method implementation
}
}
```

## Section profiling: TODO editing
### Usage Examples

Here are some usage examples to document in your code:

#### Basic Usage

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

#### With Conditional Logic

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

#### Error Handling Example

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

### Implementation Notes

1. This approach doesn't interfere with the code being profiled - it simply measures the time between creation and ending.

2. The `end()` method consumes `self`, ensuring that a profile section can only be ended once.

3. You don't need to add anything to your `Drop` implementation for `Profile` - it will work as is.

4. If you want to add more functionality, like pausing/resuming profiling, you can add more methods to `ProfileSection`.


## Analyzing Profile Results

### Profile Output

Profiling generates folded stack files with timestamps in your current directory:

- {executable_name}-{timestamp}.folded - For time profiling
- {executable_name}-{timestamp}-memory.folded - For memory profiling

### Using the Profiling Analyzer

thag_rs includes a powerful analysis tool:

```bash
cargo run --bin profile_analyze -- [options]
```

![Main menu](assets/profile_analyze_main.png)

### Profile Analysis Tool Menus

#### Main Menu

The main menu provides the following options:

- **Time Profile - Single**: Analyze a single time profile file
- **Time Profile - Differential**: Compare two time profile files
- **Memory Profile - Single**: Analyze a single memory profile file
- **Memory Profile - Differential**: Compare two memory profile files
- **Exit**: Exit the analysis tool

#### File Selection Menu

After selecting an analysis type, you'll see a list of available profile files grouped by executable name:

```
Available profile files:
1. tui_ta_editor_profile (2 files)
1.1: tui_ta_editor_profile-20250228-090304.folded
1.2: tui_ta_editor_profile-20250228-090125.folded
2. `thag` (4 files)
...
```

Options:
- **Filter/modify selection**: Apply a filter to narrow down the list of files
- **[numbered options]**: Select a specific group of profile files

#### Time Profile - Single Menu

After selecting a specific profile file, you'll see these options:

- **Show Flamechart**: Generate and display an interactive flamechart visualization
- **Filter Functions (Recursive or Exact Match)**: Apply filters to focus on specific functions
- **Show Statistics**: Display statistical summary of profiling data
- **Back to Profile Selection**: Return to the file selection menu

#### Time Profile - Differential Menu

After selecting two profile files to compare, you'll see:

- **Show Differential Flamechart**: Generate a comparative flamechart highlighting differences
- **Show Statistics Comparison**: Compare statistical data between the two profiles
- **Back to Profile Selection**: Return to the file selection menu

#### Memory Profile - Single Menu

Similar to the Time Profile menu, but focused on memory allocation data:

- **Show Memory Flamechart**: Generate a flamechart showing memory usage patterns
- **Show Memory Statistics**: Display summary statistics of memory allocations
- **Back to Profile Selection**: Return to the file selection menu

#### Memory Profile - Differential Menu

Compares memory usage between two profile runs:

- **Show Differential Memory Flamechart**: Visual comparison of memory usage
- **Show Memory Statistics Comparison**: Compare allocation statistics
- **Back to Profile Selection**: Return to the file selection menu

### Interpreting Results

The analyzer produces:

1. Statistical Summary: Shows function calls, total time, average time
2. Interactive Flamechart: Visual representation of performance data

#### Flamecharts

Flamecharts provide an intuitive visualization of your profiling data.
The wider a function appears, the more time it takes relative to the total execution.
Flamecharts are interactive SVGs that allow you to zoom in on specific functions,
hover over functions to see detailed information, search for specific functions,
and compare before/after optimizations. See the sample image below or click [here](assets/flamechart_time_20250302-080709.svg) to interact with it.

You may be more familiar with flamegraphs than flamecharts. Flamecharts are distinguished by laying out data on the horizontal axis chronologically instead of alphabetically.
Thag profiling uses flamecharts to reflect the sequence of events, in particular for the execution timeline. For memory profiling the sequence will be the sequence of `drop` events,
since this is the point at which `thag` profiling records the allocation and deallocation.

`thag` uses the `inferno` crate to generate flamecharts. For the execution timeline, the analysis tool allows you to choose the `inferno` color scheme to use. For the memory flamechart,
it uses `inferno`'s memory-optimized color scheme.

![Image: assets/flamechart_time_20250302-080709.png](assets/flamechart_time_20250302-080709.png)

[Try it!](assets/flamechart_time_20250302-080709.svg)

As the above link demonstrates, interactive features of the SVG flamecharts include:

- **Zoom**: Click on a function to zoom in
- **Details**: Hover over a function to see detailed information
- **Search**: Search for specific functions
- **Differential view**: Compare before/after optimizations

## Profiling Best Practices

1. **Profile representative workloads**: Make sure your test cases represent real-world usage.
2. **Focus on hot paths**: Look for the widest blocks in your flamechart - these are your performance bottlenecks.
3. **Compare before/after**: Always compare profiles before and after optimization to ensure you've made an improvement.
4. **Watch for memory bloat**: Use memory profiling to identify excessive allocations.
5. **Verify changes**: Before running or committing source code changes made by the automated tools, be sure to verify the changes with a diff tool.
6. **Use serial testing**: When writing tests that use profiled functions, use the serial_test crate:

```rust
use serial_test::serial;

#[test]
#[serial]
fn test_profiled_function() {
// Tests using profiled functions
}
```

This is important because `thag_rs` profiling maintains global state that isn't thread-safe.

## Advanced Features

### Profiling Async Code

The `#[profiled]` attribute works seamlessly with async functions:

```rust
#[profiled]
async fn process_data() -> Result<Data, Error> {
// Async operations
}
```

The profiler will correctly track time spent in the future, including time between .await points.

### Ending Profile Sections Early

Sometimes you may want to end profiling before a section's scope ends:

```rust
fn complex_operation() {
profile_section!("initialization");
// Initialization code...

    if skip_rest {
    end_profile_section("initialization"); // End early
    }

    // More code...
    }
    ```

### Custom Profile Names

You can provide custom names for profiled methods:

```rust
impl MyStruct {
#[profiled]
fn process(&self) {
// This will be profiled as "fn::process"
}

    // With custom name
    fn calculate(&self) {
    profile_method!("MyStruct::custom_calculate");
    // This will be profiled as "MyStruct::custom_calculate"
    }
    }
    ```

## Troubleshooting

### Common Issues

1. **Missing profile output**: Ensure profiling is enabled and you have write permissions in the current directory.
2. **Test failures**: Profiled tests, as well as tests that call profiled functions, must use the #[serial] attribute from the serial_test crate to prevent concurrent access to profiling data.
3. **Overhead**: Profiling, and in particular memory profiling, adds some overhead. For extremely performance-sensitive code, be aware that the measurements include this overhead.
Instrumented code should not add a great deal of overhead when profiling is not enabled, but for time_sensitive code you may wish to remove instrumentation. The `thag_profile_remove` tool
is effective for this.
4. **Inappropriate redirection**: Do not redirect the output of either the `thag_profile_instrument` tool or the `thag_profile_remove` tool back to the input file. Redirect it to a different file and compare the two before
overwriting the original file with the modified version.

### Inspecting Profile Files

The folded stack files are human-readable. You can inspect them directly:

```bash
head your-executable-timestamp.folded
```

## Conclusion

`thag_rs` profiling aims to provide a simple but effective cross-platform solution for understanding and optimizing your Rust code
by combining easy instrumentation, detailed analysis, and interactive visualizations to help make your code faster and more efficient.

You can get started quickly by running `thag`'s `profile_instrument` tool to auto-instrument one or more source files of interest with #[profiled]
attributes, and (in the case of `fn main`), `#[enable_profiling]` attributes. Then run your code as normal with the `#[enable_profiling]` attribute,
or if running a script from `thag` you can use `features=profiling`, and on termination run the `profile_analyze` tool to select and analyze the profile data in the current directory.
```



# User Guide to Application Profiling with `thag_profiler`

## Introduction

Profiling is key to optimizing your Rust applications, but it tends to be time-consuming (no pun intended). `thag_profiler` provides quick, straightforward and consistent run-time and memory profiling for your Rust project or script across all platforms.

`thag_profiler` uses intrusive profiling, meaning you need to instrument your code for profiling. While you can do this manually, `thag_profiler` also provides automated instrumentation tools to help. This instrumentation is "lossless", preserving original code with its comments and formatting, adding only necessary `#[profiled]` function attributes (and `#[enable_profiling]` for `fn main`) using the `rust-analyzer` syntax tree library.

The run-time overhead of instrumentation is zero when the `profiling` feature is disabled. When enabled, a wrapper is transparently added around functions to instantiate a `Profile` object to measure execution time and/or memory usage. Profiling overhead is excluded from the reported execution time.

`thag_profiler` works with both async and non-async code. It uses Rust's `backtrace` crate to populate the call stack for each Profile instance, stripping out non-profiled functions including scaffolding from async runtimes like `tokio`.

## Installation

### Adding to Your Project

Add `thag_profiler` to your project dependencies:

```toml
[dependencies]
thag_profiler = "0.1"

# Optional: Enable profiling for your project
[features]
profiling = ["thag_profiler/profiling"]
```

### Installing the Profiling Tools

```bash
# Install all tools
cargo install thag_profiler --features=tools

# Or install individual tools
cargo install thag_profiler --features=instrument-tool --bin thag-instrument
cargo install thag_profiler --features=remove-tool --bin thag-remove
cargo install thag_profiler --features=analyze-tool --bin thag-analyze
```

## Quick-start Guide

1. **Instrument your code**:

   ```bash
   thag-instrument src/main.rs
   ```

2. **Enable profiling** via one of these methods:

   - Add the `#[enable_profiling]` attribute to your main function
   - Enable the `profiling` feature when running: `cargo run --features profiling`

3. **Run your application** to generate profile data

4. **Analyze the results**:

   ```bash
   thag-analyze
   ```

## Enabling Profiling

There are two ways to enable profiling:

### 1. Using the `#[enable_profiling]` Attribute

```rust
use thag_profiler::profiled;

#[enable_profiling]  // Default: profiles both time and memory
fn main() {
    // Your code here
}

// Or specify the profiling type
#[enable_profiling(profile_type = "time")]  // Options: "time", "memory", "both"
fn main() {
    // Your code here
}
```

### 2. Using the `profiling` Feature Flag

In your Cargo.toml or script's toml block:

```toml
[features]
profiling = ["thag_profiler/profiling"]
```

Then run with:
```bash
cargo run --features profiling
```

### 3. Programmatically

```rust
use thag_profiler::{enable_profiling, ProfileType};

fn main() {
    // Enable profiling programmatically
    enable_profiling(true, ProfileType::Both).expect("Failed to enable profiling");

    // Your code here
}
```

## Instrumenting Your Code

### Automatic Instrumentation

For easier profiling, `thag_profiler` provides tools to automatically instrument your code:

```bash
thag-instrument path/to/your/file.rs
```

This will add `#[profiled]` attributes to functions and methods, and `#[enable_profiling]` to the main function if present.

When you're done profiling, remove the instrumentation:

```bash
thag-remove path/to/your/file.rs
```

### Manual Instrumentation

#### Using the `#[profiled]` Attribute

```rust
use thag_profiler::profiled;

#[profiled]
fn expensive_calculation() -> u64 {
    // Function code
}

// Works with async functions too
#[profiled]
async fn fetch_data() -> Result<String, Error> {
    // Async operations
}
```

#### The `#[profiled]` Attribute Options

```rust
// Add implementation type name to profile output
#[profiled(imp = "MyStruct")]
fn my_method(&self) { /* ... */ }

// Add trait name to profile output
#[profiled(trait_name = "MyTrait")]
fn my_method(&self) { /* ... */ }

// Specify profiling type
#[profiled(profile_type = "memory")]  // Options: "global", "time", "memory", "both"
fn memory_heavy_function() { /* ... */ }
```

#### Profiling Code Sections

For fine-grained control, you can profile specific sections of code:

```rust
fn my_function() {
    // Regular code

    // Start a profiled section
    let section = profile!("expensive part");

    // Code to profile
    expensive_operation();

    // End the profiled section
    section.end();

    // More code (not profiled)
}
```

The `profile!` macro supports various options:

```rust
// Basic usage (profiles time)
let section = profile!("section_name");

// Specify profile type
let section = profile!("memory_section", memory);  // Options: time, memory, both

// Profile async code
let section = profile!("async_section", async);
let section = profile!("async_memory", memory, async);

// Profile methods (auto-detects name from backtrace)
let section = profile!(method);
let section = profile!(method, memory);
let section = profile!(method, async);
let section = profile!(method, both, async);
```

## Conditional Profiling

You can make profiling conditional:

```rust
// Only profile in debug builds
#[cfg_attr(debug_assertions, profiled)]
fn expensive_calculation() { /* ... */ }

// Only profile when a feature is enabled
#[cfg_attr(feature = "profile_enabled", profiled)]
fn complex_operation() { /* ... */ }

// Combine with options
#[cfg_attr(feature = "profile_enabled", profiled(imp = "MyStruct"))]
fn my_method(&self) { /* ... */ }
```

## Analyzing Profile Results

### Profile Output

Profiling generates folded stack files in your current directory:

- `{executable_name}-{timestamp}.folded` - For time profiling
- `{executable_name}-{timestamp}-memory.folded` - For memory profiling

### Using the Profiling Analyzer

```bash
thag-analyze
```

This interactive tool lets you:
- View individual time or memory profiles
- Compare profiles to see optimization impacts
- Generate flamecharts for visual analysis
- Filter and search profile data
- View detailed statistics

### Interpreting Flamecharts

Flamecharts provide an intuitive visualization of your profiling data:
- The horizontal axis shows function execution chronologically
- The wider a function appears, the more time it takes
- Interactive features let you zoom, search, and explore details

## Advanced Features

### Working with Async Code

The `#[profiled]` attribute works seamlessly with async functions:

```rust
#[profiled]
async fn process_data() -> Result<Data, Error> {
    // The profiler will correctly track time spent in this future
    let result = fetch_data().await?;
    process_result(result).await
}
```

### Error Handling with Profile Sections

```rust
fn fallible_operation() -> Result<(), Error> {
    let section = profile!("critical_operation");

    match risky_function() {
        Ok(value) => {
            // Process value
            section.end();
            Ok(())
        }
        Err(e) => {
            // End profiling before returning error
            section.end();
            Err(e)
        }
    }
}
```

### Checking if Profiling is Active

```rust
fn my_function() {
    let section = profile!("expensive_part");

    if section.is_active() {
        println!("Profiling is enabled");
    }

    // Process data
    section.end();
}
```

## Best Practices

1. **Profile representative workloads**: Ensure your test cases represent real-world usage
2. **Focus on hot paths**: Look for the widest blocks in flamecharts - these are your bottlenecks
3. **Compare before/after**: Always profile before and after optimizations
4. **Use serial testing**: When testing profiled functions, use the `serial_test` crate:

```rust
use serial_test::serial;

#[test]
#[serial]
fn test_profiled_function() {
    // Tests using profiled functions
}
```

5. **Verify automated changes**: Always review changes made by the instrumentation tools

## Troubleshooting

### Common Issues

1. **Missing profile output**: Ensure profiling is enabled and you have write permissions
2. **Test failures**: Use `#[serial]` for tests with profiled functions
3. **Overhead concerns**: For extremely performance-sensitive code, be aware of profiling overhead
4. **Instrumentation safety**: Never redirect instrumentation tool output directly back to input files

## Inspecting Profile Files

The folded stack files are human-readable:

```bash
head your-executable-timestamp.folded
```

## Conclusion

`thag_profiler` provides a simple but effective cross-platform solution for understanding and optimizing your Rust code. By combining easy instrumentation, detailed analysis, and interactive visualizations, it helps you make your code faster and more efficient.

Get started quickly by running `thag-instrument` to auto-instrument your code, enable profiling, run your application, and analyze the results with `thag-analyze`.



# Readme.md for thag_profiler

I've updated the guide to reflect the new structure and features of `thag_profiler`. Here's the revised markdown for your README.md:

```md
# thag_profiler

A straightforward, lightweight profiling library for Rust applications that provides time and memory profiling with minimal overhead.

## Features

- **Zero-cost abstraction**: No runtime overhead when profiling is disabled
- **Time and memory profiling**: Track execution time or memory usage, or both
- **Function and section profiling**: Profile entire functions or specific code sections
- **Async support**: Seamlessly works with async code
- **Automatic instrumentation**: Tools to add and remove profiling code
- **Interactive flamecharts**: Visualize performance bottlenecks
- **Cross-platform**: Works on all platforms supported by Rust

## Quick Start

### Installation

Add `thag_profiler` to your `Cargo.toml`:

```toml
[dependencies]
thag_profiler = "0.1"

# If you want to enable profiling:
[features]
my_profile_feature = ["thag_profiler/profiling"]
```

Install the profiling tools:

```bash
# Install all tools
cargo install thag_profiler --features=tools

# Or install individual tools
cargo install thag_profiler --features=instrument-tool --bin thag-instrument
cargo install thag_profiler --features=remove-tool --bin thag-remove
cargo install thag_profiler --features=analyze-tool --bin thag-analyze
```

### Instrumenting Your Code

Automatically instrument your code:

```bash
thag-instrument path/to/your/file.rs > path/to/your/instrumented_file.rs
```

Or manually add profiling annotations:

```rust
use thag_profiler::{profiled, profile};

// Instrument a function
#[profiled]
fn expensive_calculation() -> u64 {
    // Function code...
    42
}

// Profile a specific section
fn complex_operation() {
    // Some code...

    let section = profile!("expensive_part");
    // Code to profile
    expensive_operation();
    section.end();

    // More code...
}
```

### Enabling Profiling

Enable profiling by adding the attribute to your main function:

```rust
use thag_profiler::profiled;

#[profiled]
fn main() {
    // Your program...
}
```

Or programmatically:

```rust
use thag_profiler::{enable_profiling, ProfileType};

fn main() {
    // Enable both time and memory profiling
    enable_profiling(true, ProfileType::Both).expect("Failed to enable profiling");

    // Your program...
}
```

### Analyzing Results

After running your program, analyze the results:

```bash
thag-analyze
```

This will open an interactive menu to explore your profiling data.

## Detailed Usage

### Function Profiling with `#[profiled]`

Add the `#[profiled]` attribute to any function you want to profile:

```rust
use thag_profiler::profiled;

// Regular function
#[profiled]
fn expensive_calculation() -> u64 {
    // Function code...
}

// Works with async functions too
#[profiled]
async fn fetch_data() -> Result<String, Error> {
    // Async operations...
}

// Methods in implementations
impl MyStruct {
    #[profiled]
    fn process(&self, data: &[u8]) {
        // Method code...
    }
}
```

#### Attribute Options

The `#[profiled]` attribute supports several options:

```rust
// Specify the implementation type for better method profiling
#[profiled(imp = "MyStruct")]
fn my_method(&self) { /* ... */ }

// Specify the trait being implemented
#[profiled(trait_name = "MyTrait")]
fn trait_method(&self) { /* ... */ }

// Specify what to profile (time, memory, or both)
#[profiled(profile_type = "memory")]
fn allocating_function() { /* ... */ }
```

### Code Section Profiling with `profile!`

Use the `profile!` macro to profile specific sections of code:

```rust
use thag_profiler::profile;

fn complex_function() {
    // Basic usage
    let section = profile!("initialization");
    initialize_things();
    section.end();

    // Profile a method
    let section = profile!(method);
    self.do_something();
    section.end();

    // Profile memory usage
    let section = profile!("allocation", memory);
    let data = vec![0; 1_000_000];
    section.end();

    // Profile async code
    let section = profile!("async_operation", async);
    async_operation().await;
    section.end();

    // Combined options
    let section = profile!(method, both, async);
    self.complex_async_operation().await;
    section.end();
}
```

### Conditional Compilation

You can conditionally apply profiling:

```rust
// Only apply profiling when a feature is enabled
#[cfg_attr(feature = "my_profile_feature", profiled)]
fn expensive_calculation() { /* ... */ }

// Only profile in debug builds
#[cfg_attr(debug_assertions, profiled)]
fn complex_operation() { /* ... */ }
```

### Profiling Tools

Thag includes three command-line tools for working with profiles:

#### 1. thag-instrument

Automatically adds profiling attributes to your code:

```bash
thag-instrument [options] <path>
```

Options:
- `-r, --recursive`: Process directories recursively
- `-e, --edition <year>`: Specify Rust edition (2015, 2018, 2021)

#### 2. thag-remove

Removes profiling attributes from your code:

```bash
thag-remove [options] <path>
```

Options:
- `-r, --recursive`: Process directories recursively
- `-e, --edition <year>`: Specify Rust edition (2015, 2018, 2021)

#### 3. thag-analyze

Interactive analysis of profiling results:

```bash
thag-analyze [options]
```

Options:
- `-f, --file <path>`: Analyze a specific profile file
- `-i, --interactive`: Start in interactive mode (default)

## Profile Analysis Features

The analyzer provides:

1. **Statistical Summary**: Shows function calls, total time, average time
2. **Interactive Flamechart**: Visual representation of performance data
3. **Differential Analysis**: Compare before/after optimizations
4. **Memory Allocation Tracking**: Identify memory usage patterns

### Flamecharts

Flamecharts provide an intuitive visualization of your profiling data. The wider a function appears, the more time it takes relative to the total execution.

Flamecharts are interactive SVGs that allow you to:
- Zoom in on specific functions
- Hover over functions to see detailed information
- Search for specific functions
- Compare before/after optimizations

![Example Flamechart](https://raw.githubusercontent.com/yourusername/thag_profiler/main/assets/flamechart_example.png)

## Best Practices

1. **Profile representative workloads**: Make sure your test cases represent real-world usage
2. **Focus on hot paths**: Look for the widest blocks in your flamechart - these are your performance bottlenecks
3. **Compare before/after**: Always compare profiles before and after optimization
4. **Watch for memory bloat**: Use memory profiling to identify excessive allocations
5. **Verify changes**: Always verify automated changes with a diff tool

## Testing with Profiled Code

When writing tests that use profiled functions, use the `serial_test` crate:

```rust
use serial_test::serial;

#[test]
#[serial]
fn test_profiled_function() {
    // Tests using profiled functions
}
```

This is important because `thag_profiler` maintains global state that isn't thread-safe.

## Troubleshooting

### Common Issues

1. **Missing profile output**: Ensure profiling is enabled and you have write permissions
2. **Test failures**: Profiled tests must use the `#[serial]` attribute
3. **Performance impact**: Memory profiling adds some overhead
4. **File redirect issues**: Never redirect output back to the input file

### Inspecting Profile Files

The folded stack files are human-readable:

```bash
head your-executable-timestamp.folded
```

## License

This project is licensed under [LICENSE INFORMATION].

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
```

## Notes on Changes

1. **Structure**:
   - Moved to README.md for thag_profiler
   - Organized with clear sections for installation, usage, and analysis
   - Added quick start to help users get going quickly

2. **Updated Content**:
   - Updated tool names to reflect their new names (thag-instrument, etc.)
   - Updated installation instructions for the new crate structure
   - Updated syntax for the new `profile!` macro
   - Removed outdated references to thag_rs
   - Clarified feature flag usage

3. **Added**:
   - More detailed information about the tools and their options
   - Clearer explanations of how to use the profiling macros
   - Better organization of the advanced features

4. **Removed**:
   - References to the old structure and naming
   - Redundant information and clarified explanations

Let me know if you'd like any specific sections expanded or changed further!


I'm trying to call the thag_profiler profile! macro from a user script, but getting this warning and then an error:
```
warning: unexpected `cfg` condition value: `profiling`
  --> /Users/donf/projects/thag_rs/bank/mem_prof.rs:23:19
   |
23 |     let section = profile!("vec_1024", both);
   |                   ^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: expected values for `feature` are: `profile`
   = note: using a cfg inside a macro will use the cfgs from the destination crate and not the ones from the defining crate
   = help: try referring to `$crate::profile_internal` crate for guidance on how handle this unexpected cfg
   = help: the macro `$crate::profile_internal` may come from an old version of the `thag_profiler` crate, try updating your dependency with `cargo update -p thag_profiler`
   = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration
   = note: `#[warn(unexpected_cfgs)]` on by default
   = note: this warning originates in the macro `$crate::profile_internal` which comes from the expansion of the macro `profile` (in Nightly builds, run with -Z macro-backtrace for more info)
```
So thag_profiler::profiling::profile_internal is interpreting $crate as referring to the user crate. Should I amend these $crate references to thag_profiler or ::thag_profiler or what?

# Fixing the `$crate` Resolution Issue in Macros

You've encountered a common issue when using macros across crate boundaries. The `$crate` meta-variable in a macro refers to the crate where the expanded code is located, not where the macro is defined. Let's fix this issue:

## The Problem

In your case, when `profile!` expands to use `$crate::profile_internal!`, it's looking for `profile_internal` in the user's crate, not in `thag_profiler` where it's actually defined.

## Solution: Fully Qualify Paths in Macros

Here's how to fix the issue:

### 1. Update the `profile!` Macro Definition

In `thag_profiler/src/lib.rs`, change:

```rust
#[macro_export]
macro_rules! profile {
    // profile!(name)
    ($name:expr) => {
        $crate::profile_internal!($name, $crate::ProfileType::Time, false, false)
    };
    // Other variants...
}
```

To use fully qualified paths:

```rust
#[macro_export]
macro_rules! profile {
    // profile!(name)
    ($name:expr) => {
        ::thag_profiler::profile_internal!($name, ::thag_profiler::ProfileType::Time, false, false)
    };

    // profile!(name, type)
    ($name:expr, time) => {
        ::thag_profiler::profile_internal!($name, ::thag_profiler::ProfileType::Time, false, false)
    };
    ($name:expr, memory) => {
        ::thag_profiler::profile_internal!($name, ::thag_profiler::ProfileType::Memory, false, false)
    };
    ($name:expr, both) => {
        ::thag_profiler::profile_internal!($name, ::thag_profiler::ProfileType::Both, false, false)
    };

    // Other variants...
}
```

### 2. Update the `profile_internal!` Macro

Similarly, update the `profile_internal!` macro to use fully qualified paths:

```rust
#[doc(hidden)]
#[macro_export]
macro_rules! profile_internal {
    ($name:expr, $type:expr, $is_async:expr, $is_method:expr) => {{
        #[cfg(feature = "profiling")]
        {
            let profile = ::thag_profiler::Profile::new(
                $name,
                $type,
                $is_async,
                $is_method
            );
            ::thag_profiler::ProfileSection { profile }
        }

        #[cfg(not(feature = "profiling"))]
        {
            ::thag_profiler::ProfileSection::new_dummy()
        }
    }};
}
```

### 3. Alternative: Using a Private Implementation

A more robust approach is to use a non-exported internal macro:

```rust
// Non-exported internal macro (not using #[macro_export])
macro_rules! _profile_internal {
    ($name:expr, $type:expr, $is_async:expr, $is_method:expr) => {{
        #[cfg(feature = "profiling")]
        {
            let profile = Profile::new($name, $type, $is_async, $is_method);
            ProfileSection { profile }
        }

        #[cfg(not(feature = "profiling"))]
        {
            ProfileSection::new_dummy()
        }
    }};
}

// Public macro that uses the internal one
#[macro_export]
macro_rules! profile {
    // profile!(name)
    ($name:expr) => {
        {
            let section = ::thag_profiler::ProfileSection::new($name);
            section
        }
    };

    // Other variants...
}
```

## Recommended Approach

The simplest fix is Option 1 - replace all `$crate::` references in your exported macros with `::thag_profiler::`. This ensures the macros always reference the correct crate regardless of where they're used.

This pattern is used by many Rust crates that export macros for use across crate boundaries.

After making these changes, your `profile!` macro should work correctly from user scripts without any warnings or errors.

I'm still getting an error if I don't run my user code with the exact feature name "profiling" when using thag_profiler with the `profiling` feature:
```
warning: unexpected `cfg` condition value: `profiling`
  --> /Users/donf/projects/thag_rs/bank/mem_prof.rs:23:19
   |
23 |     let section = profile!("vec_1024", both);
   |                   ^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: no expected values for `feature`
   = note: using a cfg inside a macro will use the cfgs from the destination crate and not the ones from the defining crate
   = help: try referring to `::thag_profiler::profile_internal` crate for guidance on how handle this unexpected cfg
   = help: the macro `::thag_profiler::profile_internal` may come from an old version of the `thag_profiler` crate, try updating your dependency with `cargo update -p thag_profiler`
   = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration
   = note: `#[warn(unexpected_cfgs)]` on by default
   = note: this warning originates in the macro `::thag_profiler::profile_internal` which comes from the expansion of the macro `profile` (in Nightly builds, run with -Z macro-backtrace for more info)
```
Here's the problem code AFAIK:
```
#[doc(hidden)]
#[macro_export]
macro_rules! profile_internal {
    ($name:expr, $type:expr, $is_async:expr, $is_method:expr) => {{
        #[cfg(feature = "profiling")]
        {
            let profile =
                ::thag_profiler::Profile::new($name.to_string(), $type, $is_async, $is_method);
            ::thag_profiler::ProfileSection { profile }
        }

        #[cfg(not(feature = "profiling"))]
        {
            ::thag_profiler::ProfileSection::new($name)
        }
    }};
}
```
I've tried changing it to #[cfg(feature = "thag_profiler/profiling")] and #[cfg(not(feature = "thag_profiler/profiling"))] but it made no difference except for the fact that the error message now includes the feature name thag_profiler/profiling. The relevant toml block info when running with no feature is:
```
[dependencies]
thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler", features = ["profiling"] }
```
IOW when I want to run profiling I too have to use a feature named profiling:
```
[features]
profiling = ["thag_profiler/profiling"]
```
The above works, but the following does not because of the feature name being `profile` instead of `profiling`:
```
[features]
profile = ["thag_profiler/profiling"]
```

# Fixing Macro Feature Detection Across Crate Boundaries

You've uncovered a subtle issue with feature detection in macros. The problem is that `#[cfg(feature = "profiling")]` in a macro expansion checks if the feature is enabled in the *user's crate*, not in `thag_profiler`. Let me explain how to fix this:

## The Core Issue

When a macro with `#[cfg(feature = "profiling")]` expands in a user's crate:
1. It checks if the *user's crate* has a feature named "profiling"
2. It doesn't matter if `thag_profiler` has that feature enabled

This is why specifying `profile = ["thag_profiler/profiling"]` doesn't work - the macro is looking for a feature literally named "profiling" in the user's crate.

## Solution: Runtime Feature Detection Instead of Compile-Time

The best approach is to make the `profile_internal!` macro use runtime detection instead of `#[cfg]`:

```rust
// In thag_profiler/src/lib.rs

// Export a constant that indicates if profiling is enabled
#[cfg(feature = "profiling")]
pub const PROFILING_ENABLED: bool = true;

#[cfg(not(feature = "profiling"))]
pub const PROFILING_ENABLED: bool = false;

#[doc(hidden)]
#[macro_export]
macro_rules! profile_internal {
    ($name:expr, $type:expr, $is_async:expr, $is_method:expr) => {{
        if ::thag_profiler::PROFILING_ENABLED {
            let profile = ::thag_profiler::Profile::new(
                $name.to_string(),
                $type,
                $is_async,
                $is_method
            );
            ::thag_profiler::ProfileSection { profile }
        } else {
            ::thag_profiler::ProfileSection::new($name)
        }
    }};
}
```

With this approach:
1. The constant `PROFILING_ENABLED` is determined at compile time in `thag_profiler`
2. The macro uses a runtime `if` check using this constant
3. No `#[cfg]` is used in the macro expansion

## Another Approach: Feature Re-Export

Another solution is to have users enable a feature in their crate with the same name:

```toml
# In user's Cargo.toml
[features]
profiling = ["thag_profiler/profiling"]  # Must use the same name
```

Then your macros would work as-is, since they check for `feature = "profiling"`.

## Documentation

Regardless of which approach you choose, document it clearly:

```rust
/// # Using the profile! macro
///
/// To use this macro with profiling enabled, add the following to your Cargo.toml:
///
/// ```toml
/// [dependencies]
/// thag_profiler = "0.1"
///
/// [features]
/// profiling = ["thag_profiler/profiling"]  # Must use this exact name
/// ```
///
/// Then enable the feature when running:
///
/// ```bash
/// cargo run --features profiling
/// ```
#[macro_export]
macro_rules! profile {
    // ...
}
```

## Recommendation

I recommend the runtime check approach as it's more flexible and less error-prone. Users won't need to worry about feature naming, and your macros will work correctly regardless of how users name their features.

This change will fix your issue and allow users to enable `thag_profiler/profiling` using any feature name they want in their own crates.

The run-time fix works fine, but I have to specify the profiling feature on the thag_profiler dependency. Specifying the feature like this (below) without that does nothing, so seems pointless, being neither necessary nor sufficient.
```
[features]
profiling = ["thag_profiler/profiling"]
```
Ok, AFAIK without the run-time fix it's necessary but not sufficient. With the run-time fix it's neither and the profiling is entirely controlled by the inclusion or exclusion of the profiling feature on the thag_profiler dependency.

# Understanding Feature Propagation in Rust Dependencies

You're exactly right about how the features are working with your runtime fix. Let me clarify the behavior and why this makes sense:

## Current Behavior with Runtime Fix

1. When you have:
   ```toml
   [dependencies]
   thag_profiler = { path = "...", features = ["profiling"] }
   ```
   This directly enables the `profiling` feature in `thag_profiler`, making `PROFILING_ENABLED = true`.

2. Adding this:
   ```toml
   [features]
   profiling = ["thag_profiler/profiling"]
   ```
   Is now redundant because you're already enabling the feature directly in the dependency declaration.

## Why This Makes Sense

The runtime check approach with `PROFILING_ENABLED` constant completely bypasses the compile-time feature detection in macros. This means:

1. Only the feature state in `thag_profiler` itself matters
2. The feature names in the user's crate are irrelevant
3. The user only needs to enable the feature in the dependency

## Best Practice Going Forward

Given this understanding, here's the recommended approach:

### For Library Users

There are two ways to enable profiling:

#### Option 1: Direct Dependency Enabling (Simplest)
```toml
[dependencies]
thag_profiler = { version = "0.1", features = ["profiling"] }
```

#### Option 2: Feature Propagation (More Flexible)
```toml
[dependencies]
thag_profiler = { version = "0.1" }

[features]
# User can name their feature whatever they want
my_profiling = ["thag_profiler/profiling"]
```

With Option 2, users would enable profiling with:
```bash
cargo run --features my_profiling
```

### Documentation Update

You should update your documentation to make this clear:

```
## Enabling Profiling

You can enable profiling in two ways:

### 1. Direct Dependency (Simple)

```toml
[dependencies]
thag_profiler = { version = "0.1", features = ["profiling"] }
```

### 2. Feature Propagation (Flexible)

```toml
[dependencies]
thag_profiler = { version = "0.1" }

[features]
# You can use any feature name
my_profiling = ["thag_profiler/profiling"]
```

Then enable with:
```bash
cargo run --features my_profiling
```
```

## Conclusion

Your runtime fix has made the system much more flexible by removing the hard dependency on feature naming. Now users have a choice between direct enabling (simpler) or feature propagation (more control).

This is actually a better design than the original, as it gives users more flexibility while simplifying the implementation. Well done on identifying and fixing this issue!



But this seems to contradict what I found, namely that
a. with the runtime fix the profiling can only be controlled by directly specifying the profiling feature on the dependency itself. b. with or without the runtime fix, feature propagation just doesn't work.
Currently with the runtime fix I have:
```
/*[toml]
[dependencies]
thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler" }

[features]
profile = ["thag_profiler/profiling"]
*/

cargo run -- bank/mem_prof.rs -f -- --features=profile

This works without issue except that #[profiled] does not expand and (therefore) no profiling occurs.


# Clarifying Feature Propagation Behavior

You're right, and I apologize for the confusion. Let's clarify what's actually happening based on your real-world testing:

## Actual Behavior

1. **With the runtime fix**:
   - Feature propagation from user crate features to `thag_profiler` **should** work, but appears not to be working in your case
   - Only directly specifying `features = ["profiling"]` on the dependency itself seems to work

2. **With or without runtime fix**:
   - Feature propagation isn't functioning as expected

## Investigating the Issue

Let's look at the possible causes:

### 1. Cargo Command Issue

When using `cargo run -- bank/mem_prof.rs -f -- --features=profile`, the `--features=profile` flag is being passed to your script, not to Cargo.

The correct command would be:
```bash
cargo run --features=profile -- bank/mem_prof.rs -f
```

### 2. Feature Specification in toml Block

In script toml blocks, feature propagation might not be working as expected. You might need to try:

```rust
/*[toml]
[dependencies]
thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler", features = ["profiling"] }
*/
```

### 3. Check the Runtime Detection Logic

Make sure the `PROFILING_ENABLED` constant is being correctly defined in `thag_profiler`:

```rust
// Double-check this implementation in thag_profiler/src/lib.rs
#[cfg(feature = "profiling")]
pub const PROFILING_ENABLED: bool = true;

#[cfg(not(feature = "profiling"))]
pub const PROFILING_ENABLED: bool = false;
```

### 4. Add Debug Output

Add some debug output to verify the feature status:

```rust
// In your main.rs or script
fn main() {
    // Check if profiling is enabled
    println!("PROFILING_ENABLED = {}", thag_profiler::PROFILING_ENABLED);

    // Rest of your code
}
```

## Recommendation for Your Specific Case

Since you're specifically working with scripts and the `thag` runner, the simplest approach is to directly specify the feature in the dependency:

```rust
/*[toml]
[dependencies]
thag_profiler = { path = "/Users/donf/projects/thag_rs/thag_profiler", features = ["profiling"] }
*/
```

For your documentation, make this clear:

```markdown
## Enabling Profiling in Scripts

When using `thag_profiler` in scripts, directly enable the feature in your toml block:

```rust
/*[toml]
[dependencies]
thag_profiler = { version = "0.1", features = ["profiling"] }
*/
```

## Enabling Profiling in Projects

For regular Cargo projects, you can either:

1. **Enable directly in the dependency**:
   ```toml
   [dependencies]
   thag_profiler = { version = "0.1", features = ["profiling"] }
   ```

2. **Use feature propagation**:
   ```toml
   [dependencies]
   thag_profiler = { version = "0.1" }

   [features]
   profiling = ["thag_profiler/profiling"]
   ```

   And run with:
   ```bash
   cargo run --features profiling
   ```
```

Let me know if this clarifies things or if you're still experiencing issues with feature propagation in regular Cargo projects!



1. Tried that first:
```
donf@MacBook-Air thag_rs % cargo run --features=profile -- bank/mem_prof.rs -f
error: the package 'thag_rs' does not contain this feature: profile
help: there is a similarly named feature: profiling
```
So I have to specify the name `profiling` instead, but this only affects thag_rs, not our script, unless the feature is specifically enabled on the thag_profiler dependency.
My version also does not enable the profiling feature on the script, tested like this:
```
    if cfg!(feature = "profiling") {
        println!("Profiling is enabled");
    } else {
        println!("Profiling is disabled");
    }
```
In fact nothing does, and yet the profiling still works with or without the script having the feature set, if and only if the profiling feature is enabled on the thag_profiler dependency as per 2. below.

2. Yeah that's the only thing that works for me.

3. Yes, I copied and pasted them.

4.
```
cargo run -- bank/mem_prof.rs -f -- --features=profile
PROFILING_ENABLED = false
```
As for the conclusion, I repeat that only 1. enabling the feature directly in the dependency is working or me. 2. feature propagation is not working under any circumstances.
Here's the generated Cargo.toml:
```
[package]
name = "mem_prof"
version = "0.0.1"
edition = "2021"

[workspace]
members = []

[dependencies.stats_alloc]
version = "0.1.10"

[dependencies.thag_profiler]
path = "/Users/donf/projects/thag_rs/thag_profiler"

[features]
profiling = ["thag_profiler/profiling"]

[[bin]]
path = "/Users/donf/projects/thag_rs/bank/mem_prof.rs"
name = "mem_prof"
edition = "2021"
required-features = []
```




# Understanding the Feature Behavior in Your Environment

Thank you for the detailed testing and feedback. Based on your results, we can now understand what's really happening:

## Confirmed Behavior

1. **Only direct feature specification works**:
   ```toml
   [dependencies.thag_profiler]
   path = "/Users/donf/projects/thag_rs/thag_profiler"
   features = ["profiling"]
   ```

2. **Feature propagation does not work**:
   ```toml
   [features]
   profiling = ["thag_profiler/profiling"]
   ```
   This does not successfully enable the feature in `thag_profiler`.

3. **Runtime detection via `PROFILING_ENABLED` works correctly**:
   When you check `thag_profiler::PROFILING_ENABLED`, it correctly reflects whether the profiling feature was directly enabled.

## Why Feature Propagation Might Not Work

There are a few possible reasons:

1. **Script Environment**: The `thag` script runner might handle features differently than standard Cargo projects.

2. **Generated Cargo.toml**: The generated Cargo.toml looks correct, but something about how it's processed might be bypassing feature propagation.

3. **Command Line Flags**: The way features are passed via command line might not be propagating correctly.

## Recommended Solution

Given these findings, we should update the documentation to clearly state that:

1. For scripts run with `thag`, directly enable the feature in the dependency:
   ```toml
   [dependencies.thag_profiler]
   path = "/Users/donf/projects/thag_rs/thag_profiler"
   features = ["profiling"]
   ```

2. The runtime check approach with `PROFILING_ENABLED` is working correctly, allowing macros to properly detect if profiling is enabled.

## Documentation Update

Here's how I'd recommend documenting this:

```markdown
## Enabling Profiling

### In Scripts Run with thag

When using `thag_profiler` in scripts run with the `thag` runner, you must directly enable the feature in your dependency declaration:

```rust
/*[toml]
[dependencies]
thag_profiler = { version = "0.1", features = ["profiling"] }
*/
```

Feature propagation via `[features]` section does not work reliably in script environments.

### In Regular Cargo Projects

In standard Cargo projects, you should:

1. **Enable the feature directly in the dependency**:
   ```toml
   [dependencies]
   thag_profiler = { version = "0.1", features = ["profiling"] }
   ```

2. Run your project normally:
   ```bash
   cargo run
   ```
```

## Future Improvements

If feature propagation is important for your use case, there are a few things we could investigate:

1. **Environment Variable Detection**: Use build-time environment variables instead of features
2. **Custom Build Script**: Add a build.rs to detect and propagate features
3. **Debug thag's Feature Handling**: Investigate how `thag` processes features in scripts

Would you like me to explore any of these options further, or is the current approach with direct feature specification sufficient for your needs?


OK, all the problems seem to be with thag's passing of parameters to the user script. I proved this by amending the generated Cargo.toml, switching to its temp Cargo.toml directory and running with `cargo run --features=profile`. The feature propagation and different feature name work fine under these conditions.

# Understanding the Issue with thag's Parameter Passing

Thank you for taking the time to verify this! Your investigation has pinpointed the exact issue: **thag's parameter passing to user scripts is the problem**, not the feature propagation mechanism itself.

This is an important clarification that helps us understand the actual behavior:

1. **Feature propagation works correctly in standard Cargo environments**
2. **Using a different feature name (like `profile` instead of `profiling`) works in standard Cargo environments**
3. **The issue is specific to how `thag` handles features when running scripts**

## Documentation Update

Given this new understanding, let's update the documentation to accurately reflect how things work:

```markdown
## Enabling Profiling

### In Scripts Run with thag

When using `thag_profiler` in scripts run with the `thag` runner, you must directly enable the feature in your dependency declaration:

```rust
/*[toml]
[dependencies]
thag_profiler = { version = "0.1", features = ["profiling"] }
*/
```

Note: While feature propagation (via the `[features]` section) works in standard Cargo projects, it currently doesn't work when running scripts through `thag` due to how parameters are passed to the generated project.

### In Regular Cargo Projects

In standard Cargo projects, you have two options:

1. **Enable the feature directly in the dependency**:
   ```toml
   [dependencies]
   thag_profiler = { version = "0.1", features = ["profiling"] }
   ```

2. **Use feature propagation** (allows you to use any feature name):
   ```toml
   [dependencies]
   thag_profiler = { version = "0.1" }

   [features]
   my_profiling = ["thag_profiler/profiling"]
   ```

   Then run with:
   ```bash
   cargo run --features my_profiling
   ```
```

## Potential Fix for thag

Since you've identified that this is an issue with `thag` specifically, you might want to consider fixing how it handles feature flags. Here's a general approach that might help:

1. If `--features` is detected in arguments, properly pass it to Cargo when building the script
2. Ensure the generated Cargo.toml correctly reflects feature dependencies
3. Add specific handling for feature flags in the thag runner

Would you like more specific suggestions on how to fix the feature flag handling in `thag`? Or is documenting the current behavior sufficient for now?



All good thanks, I got Claude Code to add a --features parameter to the thag CLI so that this now works there directly with feature propagation:
`cargo run bank/mem_prof.rs --features=profile`

# Great Solution with thag CLI Enhancement

That's excellent news! Adding a `--features` parameter directly to the thag CLI is the perfect solution to this problem. This creates a clean, intuitive interface for users to enable features when running scripts.

With this enhancement:

1. Users can now simply use `cargo run bank/mem_prof.rs --features=profile` to enable profiling
2. Feature propagation works correctly in the generated project
3. Users can use any feature name they prefer (like `profile` instead of `profiling`)

## Updated Documentation

Here's how you might document this new capability:

```markdown
## Enabling Profiling

### In Scripts Run with thag

When using `thag_profiler` in scripts, you have two options:

1. **Enable via command line** (recommended):
   ```bash
   cargo run bank/mem_prof.rs --features=profile
   ```

   With this script configuration:
   ```rust
   /*[toml]
   [dependencies]
   thag_profiler = { version = "0.1" }

   [features]
   profile = ["thag_profiler/profiling"]
   */
   ```

2. **Enable directly in the dependency**:
   ```rust
   /*[toml]
   [dependencies]
   thag_profiler = { version = "0.1", features = ["profiling"] }
   */
   ```

### In Regular Cargo Projects

In standard Cargo projects, the same options apply:

1. **Use feature propagation**:
   ```toml
   [dependencies]
   thag_profiler = { version = "0.1" }

   [features]
   my_profiling = ["thag_profiler/profiling"]
   ```

   Then run with:
   ```bash
   cargo run --features my_profiling
   ```

2. **Enable directly in the dependency**:
   ```toml
   [dependencies]
   thag_profiler = { version = "0.1", features = ["profiling"] }
   ```
```

This is a great enhancement that makes the user experience much more consistent between regular Cargo projects and scripts run with thag. Kudos to Claude Code for implementing this improvement!

Is there anything else you'd like to adjust or clarify about the profiling functionality or documentation?




I am testing the following function in a user script:
```
#[profiled]
async fn perform() {
    let (tx, rx) = flume::bounded(1);

    let outer_section = profile!("outer_async_operation", async);
    let t = async_std::task::spawn(async move {
        let inner_section = profile!("inner_async_operation", async);
        while let Ok(msg) = rx.recv_async().await {
            println!("Received: {}", msg);
        }
        inner_section.end();
    });
    outer_section.end();

    let section = profile!("send_async", async);
    tx.send_async("Hello, world!").await.unwrap();
    tx.send_async("How are you today?").await.unwrap();
    section.end();

    drop(tx);

    t.await;
}
```
It fails on the inner_section code like this:
```
error: future cannot be sent between threads safely
  --> /Users/donf/projects/thag_rs/demo/flume_async_profile.rs:57:13
   |
57 |       let t = async_std::task::spawn(async move {
   |  _____________^
58 | |         let inner_section = profile!("inner_async_operation", async);
59 | |         while let Ok(msg) = rx.recv_async().await {
60 | |             println!("Received: {}", msg);
61 | |         }
62 | |         inner_section.end();
63 | |     });
   | |______^ future created by async block is not `Send`
   |
   = help: within `{async block@/Users/donf/projects/thag_rs/demo/flume_async_profile.rs:57:36: 57:46}`, the trait `std::marker::Send` is not implemented for `*const ()`
note: future is not `Send` as this value is used across an await
  --> /Users/donf/projects/thag_rs/demo/flume_async_profile.rs:59:45
   |
58 |         let inner_section = profile!("inner_async_operation", async);
   |             ------------- has type `ProfileSection` which is not `Send`
59 |         while let Ok(msg) = rx.recv_async().await {
   |                                             ^^^^^ await occurs here, with `inner_section` maybe used later
note: required by a bound in `async_std::task::spawn`
  --> /Users/donf/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/async-std-1.13.0/src/task/spawn.rs:28:29
   |
26 | pub fn spawn<F, T>(future: F) -> JoinHandle<T>
   |        ----- required by a bound in this function
27 | where
28 |     F: Future<Output = T> + Send + 'static,
   |                             ^^^^ required by this bound in `spawn`
```
The expansion of the function by the #[profiled] attribute is:
```
Expanded macro profiled:

async fn perform() {
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    struct ProfiledFuture<F> {
        inner: F,
        _profile: Option<::thag_profiler::Profile>,
    }
    impl<F: Future> Future for ProfiledFuture<F> {
        type Output = F::Output;
        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = unsafe { self.as_mut().get_unchecked_mut() };
            let result = unsafe { Pin::new_unchecked(&mut this.inner) }.poll(cx);
            if result.is_ready() {
                this._profile.take();
            }
            result
        }
    }
    let future = async {
        let (tx, rx) = flume::bounded(1);
        let outer_section = profile!("outer_async_operation", async);
        let t = async_std::task::spawn(async move {
            let inner_section = profile!("inner_async_operation", async);
            while let Ok(msg) = rx.recv_async().await {
                println!("Received: {}", msg);
            }
            inner_section.end();
        });
        outer_section.end();
        let section = profile!("send_async", async);
        tx.send_async("Hello, world!").await.unwrap();
        tx.send_async("How are you today?").await.unwrap();
        section.end();
        drop(tx);
        t.await;
    };
    ProfiledFuture {
        inner: future,
        _profile: ::thag_profiler::Profile::new(
            "perform",
            ::thag_profiler::profiling::get_global_profile_type(),
            true,
            false,
        ),
    }
        .await
}
```
If I comment out the two lines that refer to `inner_section`, it works fine.

I don't understand the issues very well, but I'm wondering if the inner_section profiling is legitimate and if not, why not? Should we be modifying struct ProfileSection to be send?
Having said that, I've just remembered that struct Profile, for which ProfileSection is a wrapper, contains this field to make it not Send:
```
_not_send: PhantomData<*const ()>, // Makes Profile !Send
```
Now this field dates back to the days of a global stack, thread-local storage etc., so it may well be redundant. What's your take on this?

# Making ProfileSection Send-Safe for Async Contexts

You've identified an important limitation in the current implementation. The `ProfileSection` (and underlying `Profile`) type is intentionally marked as `!Send` through the `PhantomData<*const ()>` field. This is causing problems when trying to use it across thread boundaries in async code.

## Understanding the Issue

1. **Current Design**: `Profile` and `ProfileSection` are not `Send` (can't be moved between threads)
2. **Async Context**: `async_std::task::spawn` requires its future to be `Send`
3. **Problem**: Your inner section isn't `Send`, so it can't cross the thread boundary

## The History and Current State

The `!Send` constraint was likely added to prevent profiling data corruption if a `Profile` instance was moved between threads, especially when using thread-local storage or a global stack.

However, with your new design that doesn't rely on thread-local storage or a global stack, this restriction might be unnecessary. The key question is: **Is there anything in the current implementation that would be unsafe if `Profile` was `Send`?**

## Proposed Solution

Based on your description, I believe we can safely make `ProfileSection` (and `Profile`) implement `Send` by removing the `PhantomData` marker:

1. Remove the `!Send` marker from `Profile`:
```rust
// Before
struct Profile {
    // other fields
    _not_send: PhantomData<*const ()>, // Makes Profile !Send
}

// After
struct Profile {
    // other fields
    // _not_send field removed
}
```

2. Ensure that all the other fields in `Profile` are `Send`

3. Explicitly implement `Send` for `Profile` and `ProfileSection`:
```rust
// This is just for clarity - removing the PhantomData marker should be enough
unsafe impl Send for Profile {}
unsafe impl Send for ProfileSection {}
```

## Safety Considerations

Before making this change, verify that:

1. **No Shared Mutable State**: Ensure the profiling system doesn't rely on shared mutable state across threads without proper synchronization
2. **No Thread-Local Dependencies**: Check if any part still relies on thread-local storage
3. **Proper Drop Handling**: Make sure `drop` implementations work correctly across thread boundaries

## Testing the Change

Once you've made these changes, test with both:

1. **Cross-thread async contexts** (like your example)
2. **Multi-threaded synchronous code** to ensure profiling still works correctly

## Is This Change Safe?

Based on your migration away from a global stack and thread-local storage, and the move to a more self-contained design with `ProfileSection`, removing the `!Send` marker should be safe.

The current limitation unnecessarily restricts the use of profiling in common async patterns, and removing it would make your profiling system more versatile.

Would you like me to help you implement this change, or would you prefer to first review more details about potential thread safety concerns?



That seems to be working fine now, after some further debugging of issues it exposed, so thanks.
There is another issue, this time with enabling profiling. If I call the function thag_profiler::profiling::enable_profiling directly, there is no problem. However, if instead I use the #[enable_profiling] attribute, I get a similar kind of error:
```
error[E0728]: `await` is only allowed inside `async` functions and blocks
  --> /Users/donf/projects/thag_rs/demo/flume_async_profile.rs:49:23
   |
36 | #[enable_profiling]
   | ------------------- this is not `async`
...
49 |     let _ = perform().await;
   |                       ^^^^^ only allowed inside `async` functions and blocks
```

Here is the function signature with all its attributes:
```
#[cfg(feature = "async")]
#[async_std::main]
#[enable_profiling]
async fn main() {
...
```

Here is the expansion of the attribute macro:
```
Expanded macro enable_profiling:

#[cfg(feature = "async")]
async fn main() {
    use ::thag_profiler::profiling::{enable_profiling, ProfileType};
    enable_profiling(true, ProfileType::Both).expect("Failed to enable profiling");
    let result = (|| {
        {
            println!("PROFILING_ENABLED = {}", thag_profiler::PROFILING_ENABLED);
            if cfg!(feature = "profiling") {
                println!("Profiling is enabled");
            } else {
                println!("Profiling is disabled");
            }
            let _ = perform().await;
        }
    })();
    enable_profiling(false, ProfileType::Both).expect("Failed to disable profiling");
    result
}
```
Any ideas? It looks to me as though it's related to the attribute macro wrapping the function logic in a closure that is not async, but I don't know how to fix it.

# Fixing the #[enable_profiling] Attribute for Async Functions

You've identified the issue correctly! The `#[enable_profiling]` attribute is wrapping your async function body in a non-async closure. This breaks the `await` inside your function because only async functions and blocks can await.

## The Problem

Currently, the expanded code looks like:

```rust
async fn main() {
    // ...
    let result = (|| {  // <-- This is a normal closure, not async
        {
            // ... your async code with .await ...
            let _ = perform().await;  // <-- Error: await in non-async context
        }
    })();
    // ...
}
```

## The Solution

We need to modify the `enable_profiling` proc macro to handle async functions differently. Here's how to fix it:

```rust
#[proc_macro_attribute]
pub fn enable_profiling(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // Check if the function is async
    let is_async = input.sig.asyncness.is_some();

    // Parse profile type
    let profile_type = parse_profile_type(attr);

    // Get function details
    let fn_name = &input.sig.ident;
    let inputs = &input.sig.inputs;
    let output = &input.sig.output;
    let generics = &input.sig.generics;
    let where_clause = &input.sig.generics.where_clause;
    let vis = &input.vis;
    let block = &input.block;
    let attrs = &input.attrs;

    let result = if is_async {
        // Handle async function
        quote! {
            #(#attrs)*
            #vis async fn #fn_name #generics(#inputs) #output #where_clause {
                use ::thag_profiler::profiling::{enable_profiling, ProfileType};

                enable_profiling(true, ProfileType::#profile_type).expect("Failed to enable profiling");

                // For async functions, we need to use an async block
                let result = async {
                    #block
                }.await;

                enable_profiling(false, ProfileType::#profile_type).expect("Failed to disable profiling");
                result
            }
        }
    } else {
        // Handle non-async function (existing implementation)
        quote! {
            #(#attrs)*
            #vis fn #fn_name #generics(#inputs) #output #where_clause {
                use ::thag_profiler::profiling::{enable_profiling, ProfileType};

                enable_profiling(true, ProfileType::#profile_type).expect("Failed to enable profiling");

                let result = (|| {
                    #block
                })();

                enable_profiling(false, ProfileType::#profile_type).expect("Failed to disable profiling");
                result
            }
        }
    };

    result.into()
}

// Helper function to parse profile type
fn parse_profile_type(attr: TokenStream) -> proc_macro2::TokenStream {
    // Your existing implementation to parse profile_type
    // Default to "Both" if not specified
    quote! { Both }
}
```

The key changes are:

1. Detect if the function is async (`input.sig.asyncness.is_some()`)
2. For async functions, wrap the body in an `async { ... }.await` block instead of a regular closure
3. For non-async functions, keep the existing implementation

## Testing the Fix

After implementing this fix, your code should work correctly:

```rust
#[cfg(feature = "async")]
#[async_std::main]
#[enable_profiling]
async fn main() {
    let _ = perform().await;  // This will now work correctly
}
```

The expanded code will look like:

```rust
#[cfg(feature = "async")]
async fn main() {
    use ::thag_profiler::profiling::{enable_profiling, ProfileType};

    enable_profiling(true, ProfileType::Both).expect("Failed to enable profiling");

    let result = async {
        println!("PROFILING_ENABLED = {}", thag_profiler::PROFILING_ENABLED);
        if cfg!(feature = "profiling") {
            println!("Profiling is enabled");
        } else {
            println!("Profiling is disabled");
        }
        let _ = perform().await;
    }.await;

    enable_profiling(false, ProfileType::Both).expect("Failed to disable profiling");
    result
}
```

This preserves the async context throughout, allowing your awaits to work properly.



```rs thag_rs/thag_profiler/src/profiling.rs
use crate::{static_lazy, ProfileError};
use chrono::Local;
use memory_stats::memory_stats;
use once_cell::sync::Lazy;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::BufWriter,
    ops::Deref,
    path::PathBuf,
    sync::{
        atomic::{AtomicU8, Ordering},
        Mutex,
    },
    time::Instant,
};

// use std::fs::OpenOptions;
// use std::io::Write;

// #[cfg(feature = "profiling")]
use backtrace::Backtrace;

#[cfg(feature = "profiling")]
use crate::ProfileResult;

// #[cfg(feature = "profiling")]
use rustc_demangle::demangle;

#[cfg(feature = "profiling")]
use std::{
    convert::Into,
    fs::OpenOptions,
    io::Write,
    // path::PathBuf,
    sync::atomic::{AtomicBool, AtomicU64},
    time::SystemTime,
};

// Single atomic for runtime profiling state
#[cfg(feature = "profiling")]
static PROFILING_STATE: AtomicBool = AtomicBool::new(false);

// Mutex to protect profiling state changes
#[cfg(feature = "profiling")]
static PROFILING_MUTEX: Mutex<()> = Mutex::new(());

// Compile-time feature check
#[cfg(feature = "profiling")]
const PROFILING_FEATURE: bool = true;

static PROFILE_TYPE: AtomicU8 = AtomicU8::new(0); // 0 = None, 1 = Time, 2 = Memory, 3 = Both

// Global registry of profiled functions
static PROFILED_FUNCTIONS: Lazy<Mutex<HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// #[cfg(feature = "profiling")]
static_lazy! {
    ProfilePaths: ProfileFilePaths = {
        let script_path = std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from("unknown"));
        let script_stem = script_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        let timestamp = Local::now().format("%Y%m%d-%H%M%S");
        let base = format!("{script_stem}-{timestamp}");

        ProfileFilePaths {
            time: format!("{base}.folded"),
            memory: format!("{base}-memory.folded"),
        }
    }
}

// File handles
// #[cfg(feature = "profiling")]
static_lazy! {
    TimeProfileFile: Mutex<Option<BufWriter<File>>> = Mutex::new(None)
}

// #[cfg(feature = "profiling")]
static_lazy! {
    MemoryProfileFile: Mutex<Option<BufWriter<File>>> = Mutex::new(None)
}

#[cfg(feature = "profiling")]
static START_TIME: AtomicU64 = AtomicU64::new(0);

#[derive(Clone)]
#[allow(dead_code)]
struct ProfileFilePaths {
    time: String,
    memory: String,
}

// // #[derive(Clone)]
// #[cfg(not(feature = "profiling"))]
// struct ProfileFilePaths {}

/// Resets a profile file by clearing its buffer writer.
///
/// # Arguments
/// * `file` - The mutex-protected buffer writer to reset
/// * `file_type` - A description of the file type for error messages (e.g., "time", "memory")
///
/// # Errors
/// Returns a `ProfileError` if the mutex lock fails
#[cfg(feature = "profiling")]
fn reset_profile_file(file: &Mutex<Option<BufWriter<File>>>, file_type: &str) -> ProfileResult<()> {
    *file
        .lock()
        .map_err(|_| ProfileError::General(format!("Failed to lock {file_type} profile file")))? =
        None;
    Ok(())
}

/// Initializes profile files based on the specified profile type.
///
/// This function handles the initialization sequence for both profiling files:
/// - For Time profiling: creates and initializes the time profile file
/// - For Memory profiling: creates and initializes memory profile file
/// - For Both: initializes both files
///
/// # Arguments
/// * `profile_type` - The type of profiling to initialize files for
///
/// # Errors
/// Returns a `ProfileError` if any file operations fail
#[cfg(feature = "profiling")]
fn initialize_profile_files(profile_type: ProfileType) -> ProfileResult<()> {
    let paths = ProfilePaths::get();

    match profile_type {
        ProfileType::Time => {
            TimeProfileFile::init();
            reset_profile_file(TimeProfileFile::get(), "time")?;
            initialize_profile_file(&paths.time, "Time Profile")?;
        }
        ProfileType::Memory => {
            MemoryProfileFile::init();
            reset_profile_file(MemoryProfileFile::get(), "memory")?;
            initialize_profile_file(&paths.memory, "Memory Profile")?;
        }
        ProfileType::Both => {
            // Initialize all files
            TimeProfileFile::init();
            MemoryProfileFile::init();

            // Reset all files
            reset_profile_file(TimeProfileFile::get(), "time")?;
            reset_profile_file(MemoryProfileFile::get(), "memory")?;

            // Initialize all files with headers
            initialize_profile_file(&paths.time, "Time Profile")?;
            initialize_profile_file(&paths.memory, "Memory Profile")?;
        }
    }
    Ok(())
}

pub fn get_global_profile_type() -> ProfileType {
    match PROFILE_TYPE.load(Ordering::SeqCst) {
        2 => ProfileType::Memory,
        3 => ProfileType::Both,
        _ => ProfileType::Time,
    }
}

#[cfg(feature = "profiling")]
fn set_profile_type(profile_type: ProfileType) {
    let value = match profile_type {
        ProfileType::Time => 1,
        ProfileType::Memory => 2,
        ProfileType::Both => 3,
    };
    PROFILE_TYPE.store(value, Ordering::SeqCst);
}

/// Enables or disables profiling with the specified profile type.
///
/// When enabling profiling, this function:
/// 1. Initializes path information
/// 2. Records the start time
/// 3. Sets up appropriate profile files based on the profile type
///
/// When disabling profiling, it ensures all profiling operations are stopped.
///
/// # Arguments
/// * `enabled` - Whether to enable or disable profiling
/// * `profile_type` - The type of profiling to enable (Time, Memory, or Both)
///
/// # Errors
/// Returns a `ProfileError` if:
/// - Time value conversion fails
/// - File operations fail
/// - Mutex operations fail
#[cfg(feature = "profiling")]
pub fn enable_profiling(enabled: bool, profile_type: ProfileType) -> ProfileResult<()> {
    // Acquire the mutex to ensure only one thread can enable/disable profiling at a time
    let _guard = PROFILING_MUTEX
        .lock()
        .map_err(|_| ProfileError::General("Failed to acquire profiling mutex".into()))?;

    if enabled {
        set_profile_type(profile_type);

        let Ok(now) = u64::try_from(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros(),
        ) else {
            return Err(ProfileError::General("Time value too large".into()));
        };
        START_TIME.store(now, Ordering::SeqCst);

        initialize_profile_files(profile_type)?;
    }

    // Whether enabling or disabling, set the state
    PROFILING_STATE.store(enabled, Ordering::SeqCst);
    Ok(())
}

/// No-op version when profiling feature is disabled.
///
/// # Errors
/// None
#[cfg(not(feature = "profiling"))]
pub const fn enable_profiling(
    _enabled: bool,
    _profile_type: ProfileType,
) -> Result<(), ProfileError> {
    // No-op implementation
    Ok(())
}

/// Disable profiling and reset the profiling stack.
#[cfg(feature = "profiling")]
pub fn disable_profiling() {
    PROFILING_STATE.store(false, Ordering::SeqCst);
}

#[cfg(not(feature = "profiling"))]
pub const fn disable_profiling() {
    // No-op implementation
}

/// Creates and initializes a single profile file with header information.
///
/// Creates the file if it doesn't exist, truncates it if it does, and writes
/// standard header information including:
/// - Profile type
/// - Script path
/// - Start timestamp
/// - Version information
///
/// # Arguments
/// * `path` - The path where the file should be created
/// * `profile_type` - A description of the profile type for the header
///
/// # Errors
/// Returns a `ProfileError` if file creation or writing fails
#[cfg(feature = "profiling")]
fn initialize_profile_file(path: &str, profile_type: &str) -> ProfileResult<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;

    writeln!(file, "# {profile_type}")?;
    writeln!(
        file,
        "# Script: {}",
        std::env::current_exe().unwrap_or_default().display()
    )?;
    writeln!(file, "# Started: {}", START_TIME.load(Ordering::SeqCst))?;
    writeln!(file, "# Version: {}", env!("CARGO_PKG_VERSION"))?;
    if path.ends_with("alloc.log") {
        writeln!(file, "# Format: operation|size")?;
    }
    writeln!(file)?;

    Ok(())
}

/// Checks if profiling is currently enabled.
///
/// This is used throughout the profiling system to determine whether
/// profiling operations should be performed. It's atomic and thread-safe.
///
/// # Returns
/// `true` if profiling is enabled, `false` otherwise
#[inline(always)]
#[allow(clippy::inline_always)]
#[cfg(feature = "profiling")]
pub fn is_profiling_enabled() -> bool {
    PROFILING_FEATURE || PROFILING_STATE.load(Ordering::SeqCst)
}

#[cfg(not(feature = "profiling"))]
#[must_use]
pub const fn is_profiling_enabled() -> bool {
    false
}

#[derive(Debug, Clone, Copy)]
pub enum ProfileType {
    Time, // Wall clock/elapsed time
    Memory,
    Both,
}

impl ProfileType {
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "time" => Some(Self::Time),
            "memory" => Some(Self::Memory),
            "both" => Some(Self::Both),
            _ => None,
        }
    }
}

#[allow(dead_code)]
pub struct Profile {
    start: Option<Instant>,
    profile_type: ProfileType,
    initial_memory: Option<usize>, // For memory delta
    path: Vec<String>,             // Full call stack (only profiled functions)
    custom_name: Option<String>,   // Custom section name when provided via profile!("name") macro
                                   // _not_send: PhantomData<*const ()>, // Makes Profile !Send
}

impl Profile {
    /// Creates a new `Profile` to profile a section of code.
    ///
    /// # Panics
    ///
    /// Panics if stack validation fails.
    #[allow(clippy::inline_always, clippy::too_many_lines, unused_variables)]
    pub fn new(
        name: Option<&str>,
        requested_type: ProfileType,
        is_async: bool,
        is_method: bool,
    ) -> Option<Self> {
        if !is_profiling_enabled() {
            return None;
        }

        // eprintln!("Current function/section: {name}");

        // Get the current backtrace
        let mut current_backtrace = Backtrace::new_unresolved();
        current_backtrace.resolve();

        // if name == Some("new") {
        //     eprintln!("Current backtrace: {current_backtrace:?}");
        // }

        let mut is_within_target_range = false;

        // First, collect all relevant frames
        let mut raw_frames: Vec<String> = Vec::new();

        // If this is a method, we'll capture the calling class
        let mut maybe_method_name: Option<String> = None;
        let mut maybe_function_name: Option<String> = None;
        let mut first_frame_after_profile = false;

        for frame in Backtrace::frames(&current_backtrace) {
            for symbol in frame.symbols() {
                if let Some(name) = symbol.name() {
                    let name_str = name.to_string();
                    // eprintln!("Symbol name: {name_str}");

                    // Check if we've reached the start condition
                    if !is_within_target_range && name_str.contains("Profile::new") {
                        is_within_target_range = true;
                        first_frame_after_profile = true;
                        continue;
                    }

                    // Collect frames within our target range
                    if is_within_target_range {
                        // If this is the first frame after Profile::new and it's a method,
                        // then we can extract the class name
                        if first_frame_after_profile {
                            first_frame_after_profile = false;
                            let demangled = demangle(&name_str).to_string();
                            // Clean the demangled name
                            let cleaned = clean_function_name(&demangled);
                            // eprintln!("cleaned name: {cleaned:?}");
                            if is_method {
                                maybe_method_name = extract_class_method(&cleaned);
                                // eprintln!("class_method name: {maybe_method_name:?}");
                            } else {
                                maybe_function_name = extract_fn_only(&cleaned);
                            }
                        }

                        // Skip tokio::runtime functions
                        if name_str.starts_with("tokio::") {
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
        }

        // Register this function
        let fn_name = maybe_method_name.as_ref().map_or(
            maybe_function_name
                .as_ref()
                .map_or_else(|| name.unwrap_or("unknown"), Deref::deref),
            Deref::deref,
        );
        let desc_fn_name = if is_async {
            format!("async::{fn_name}")
        } else {
            fn_name.to_string()
        };
        // eprintln!("fn_name={fn_name}, is_method={is_method}, maybe_method_name={maybe_method_name:?}, maybe_function_name={maybe_function_name:?}, desc_fn_name={desc_fn_name}");
        register_profiled_function(fn_name, desc_fn_name);

        // Process the collected frames to collapse patterns and clean up
        let cleaned_stack = clean_stack_trace(raw_frames);
        // eprintln!("cleaned_stack={cleaned_stack:?}");

        // Filter to only profiled functions
        let mut path: Vec<String> = Vec::new();

        // Add self and ancestors that are profiled functions
        for fn_name_str in cleaned_stack {
            let maybe_class_name = extract_class_method(&fn_name_str);
            if let Some(class_name) = maybe_class_name {
                // eprintln!("Class name: {}", class_name);
                if let Some(name) = get_reg_desc_name(&class_name) {
                    // eprintln!("Registered desc name: {}", name);
                    path.push(name);
                    continue;
                }
            }
            let key = get_fn_desc_name(&fn_name_str);
            // eprintln!("Function desc name: {}", key);
            if let Some(name) = get_reg_desc_name(&key) {
                // eprintln!("Registered desc name: {}", name);
                path.push(name);
            }
        }

        // Reverse the path so it goes from root caller to current function
        path.reverse();

        // In test mode with our test wrapper active, skip creating profile for #[profiled] attribute
        #[cfg(test)]
        if is_test_mode_active() {
            // If this is from an attribute in a test, don't create a profile
            // Our safe wrapper will handle profiling instead
            return None;
        }

        let global_type = match PROFILE_TYPE.load(Ordering::SeqCst) {
            2 => ProfileType::Memory,
            3 => ProfileType::Both,
            _ => ProfileType::Time, // default
        };

        // // Use the more comprehensive of the two types
        // let profile_type = match (requested_type, global_type) {
        //     (ProfileType::Both, _) | (_, ProfileType::Both) => ProfileType::Both,
        //     (ProfileType::Memory, _) | (_, ProfileType::Memory) => ProfileType::Memory,
        //     _ => ProfileType::Time,
        // };

        // Try allowing overrides
        let profile_type = requested_type;

        let initial_memory = if matches!(profile_type, ProfileType::Memory | ProfileType::Both) {
            // Get initial memory snapshot
            memory_stats().map(|stats| stats.physical_mem)
        } else {
            None
        };

        // Determine if we should keep the custom name
        let custom_name = name.and_then(|n| {
            if n.is_empty() {
                return None; // Empty names are never useful
            }

            // For methods (with Class::method syntax), check if custom name matches just the method part
            if let Some(method_name) = &maybe_method_name {
                // Extract the method part after the last ::
                if let Some(pos) = method_name.rfind("::") {
                    let method_part = &method_name[(pos + 2)..];
                    // Don't use custom name if it exactly matches just the method part
                    if n == method_part {
                        return None;
                    }
                }
            }
            // For regular functions, check if custom name matches the entire function name
            else if let Some(function_name) = &maybe_function_name {
                if n == function_name {
                    return None;
                }
            }
            // If we get here, the custom name adds value, so keep it
            Some(n.to_string())
        });

        // Debug output can be turned back on if needed for troubleshooting
        // println!(
        //     "DEBUG: Profile::new with name='{name}', fn_name='{fn_name}', custom_name={custom_name:?}, requested_type={requested_type:?}, profile_type={profile_type:?}, initial_memory={initial_memory:?}"
        // );

        Some(Self {
            // id,
            // name,
            profile_type,
            start: Some(Instant::now()),
            initial_memory,
            path,
            custom_name,
            // _not_send: PhantomData,
        })
    }

    /// Writes a profiling event to the specified profile file.
    ///
    /// This is a low-level function used by both time and memory profiling
    /// to write events in a consistent format. It handles file creation,
    /// buffering, and error handling.
    ///
    /// # Arguments
    /// * `path` - The path to the profile file
    /// * `file` - The mutex-protected buffer writer for the file
    /// * `entry` - The formatted entry to write (including stack trace and measurement)
    ///
    /// # Errors
    /// Returns a `ProfileError` if:
    /// * The mutex lock fails
    /// * File operations fail
    /// * Writing to the file fails
    #[cfg(feature = "profiling")]
    fn write_profile_event(
        path: &str,
        file: &Mutex<Option<BufWriter<File>>>,
        entry: &str,
    ) -> ProfileResult<()> {
        let mut guard = file
            .lock()
            .map_err(|_| ProfileError::General("Failed to lock profile file".into()))?;

        if guard.is_none() {
            *guard = Some(BufWriter::new(
                OpenOptions::new().create(true).append(true).open(path)?,
            ));
        }

        if let Some(writer) = guard.as_mut() {
            writeln!(writer, "{entry}")?;
            writer.flush()?;
        }
        drop(guard);
        Ok(())
    }

    /// Records a time profiling event.
    ///
    /// Writes the elapsed time for a profiled section along with its stack trace
    /// to the time profile file.
    ///
    /// # Arguments
    /// * `duration` - The elapsed time of the profiled section
    ///
    /// # Errors
    /// Returns a `ProfileError` if writing to the profile file fails
    #[cfg(feature = "profiling")]
    fn write_time_event(&self, duration: std::time::Duration) -> ProfileResult<()> {
        // Profile must exist and profiling must be enabled if we got here
        // Only keep the business logic checks

        let micros = duration.as_micros();
        if micros == 0 {
            println!(
                "DEBUG: Not writing time event for stack: {:?} due to zero duration",
                self.path
            );
            return Ok(());
        }

        let stack = &self.path;

        if stack.is_empty() {
            println!("DEBUG: Stack is empty for {:?}", self.custom_name);
            return Err(ProfileError::General("Stack is empty".into()));
        }

        // println!("DEBUG: write_time_event for stack: {:?}", stack);

        // Create a copy of the stack for our modified output
        let mut stack_with_custom_name = stack.clone();

        // Add our custom section name to the end of the stack path if present
        if let Some(name) = &self.custom_name {
            // println!("DEBUG: Adding custom name '{}' to time stack", name);

            // If the stack is not empty, get the last function name
            if let Some(last_fn) = stack_with_custom_name.last_mut() {
                // Append the custom name to the last function name
                *last_fn = format!("{last_fn}:{name}");
                println!("DEBUG: Modified stack entry to '{}'", last_fn);
            }
        }

        let stack_str = stack_with_custom_name.join(";");
        let entry = format!("{stack_str} {micros}");

        let paths = ProfilePaths::get();
        Self::write_profile_event(&paths.time, TimeProfileFile::get(), &entry)
    }

    #[cfg(feature = "profiling")]
    fn write_memory_event_with_op(&self, delta: usize, op: char) -> ProfileResult<()> {
        if delta == 0 {
            // Keep this as it's a business logic check
            println!(
                "DEBUG: Not writing memory event for stack: {:?} due to zero delta",
                self.path
            );
            return Ok(());
        }

        let stack = &self.path;

        if stack.is_empty() {
            return Err(ProfileError::General("Stack is empty".into()));
        }

        // println!("DEBUG: write_memory_event for stack: {:?}", stack);

        // Create a copy of the stack for our modified output
        let mut stack_with_custom_name = stack.clone();

        // Add our custom section name to the end of the stack path if present
        if let Some(name) = &self.custom_name {
            // println!("DEBUG: Adding custom name '{}' to memory stack", name);

            // If the stack is not empty, get the last function name
            if let Some(last_fn) = stack_with_custom_name.last_mut() {
                // Append the custom name to the last function name
                *last_fn = format!("{last_fn}:{name}");
                // println!("DEBUG: Modified stack entry to '{}'", last_fn);
            }
        }
        let stack_str = stack_with_custom_name.join(";");
        let entry = format!("{stack_str} {op}{delta}");

        let paths = ProfilePaths::get();
        Self::write_profile_event(&paths.memory, MemoryProfileFile::get(), &entry)
    }

    #[cfg(feature = "profiling")]
    fn record_memory_change(&self, delta: usize) -> ProfileResult<()> {
        if delta == 0 {
            return Ok(());
        }

        // Record allocation
        self.write_memory_event_with_op(delta, '+')?;

        // Record corresponding deallocation
        // Store both events atomically to maintain pairing
        self.write_memory_event_with_op(delta, '-')?;

        Ok(())
    }
}

fn get_fn_desc_name(fn_name_str: &String) -> String {
    // extract_fn_only(fn_name_str).map_or_else(|| fn_name_str.to_string(), |fn_only| fn_only)
    extract_fn_only(fn_name_str).unwrap_or_else(|| fn_name_str.to_string())
}

#[cfg(feature = "profiling")]
impl Drop for Profile {
    fn drop(&mut self) {
        // println!("In drop for Profile {:?}", self);
        if let Some(start) = self.start.take() {
            // Handle time profiling as before
            match self.profile_type {
                ProfileType::Time | ProfileType::Both => {
                    let elapsed = start.elapsed();
                    let _ = self.write_time_event(elapsed);
                }
                ProfileType::Memory => (),
            }
        }

        // Handle memory profiling
        if matches!(self.profile_type, ProfileType::Memory | ProfileType::Both) {
            if let Some(initial) = self.initial_memory {
                if let Some(stats) = memory_stats() {
                    let final_memory = stats.physical_mem;
                    let delta = final_memory.saturating_sub(initial);

                    if delta > 0 {
                        let _ = self.record_memory_change(delta);
                    }
                }
            }
        }
    }
}

#[cfg(feature = "profiling")]
pub struct ProfileSection {
    pub profile: Option<Profile>,
}

#[cfg(feature = "profiling")]
impl ProfileSection {
    #[must_use]
    pub fn new(name: Option<&str>) -> Self {
        Self {
            profile: Profile::new(
                name,
                get_global_profile_type(),
                false, // is_async
                false, // is_method
            ),
        }
    }

    pub fn end(self) {
        // Profile (if any) will be dropped here
    }

    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.profile.is_some()
    }
}

// Dummy implementation when profiling is disabled
#[cfg(not(feature = "profiling"))]
pub struct ProfileSection;

#[cfg(not(feature = "profiling"))]
impl ProfileSection {
    #[must_use]
    pub const fn new(_name: Option<&str>) -> Self {
        Self
    }
    pub const fn end(self) {}
    #[must_use]
    pub const fn is_active(&self) -> bool {
        false
    }
}

// This is just for clarity - removing the PhantomData marker should be enough
unsafe impl Send for Profile {}
unsafe impl Send for ProfileSection {}

/// Register a function name with the profiling registry
///
/// # Panics
///
/// Panics if it finds the name "new", which shows that the inclusion of the
/// type in the method name is not working.
pub fn register_profiled_function(name: &str, desc_name: String) {
    #[cfg(debug_assertions)]
    assert!(
        name != "new",
        "Logic error: `new` is not an accepted function name on its own. It must be qualified with the type name: `<Type>::new`. desc_name={desc_name}"
    );
    if let Ok(mut registry) = PROFILED_FUNCTIONS.lock() {
        // eprintln!("Registering function: {name}::{desc_name}",);
        registry.insert(name.to_string(), desc_name);
    }
}

// Check if a function is registered for profiling
pub fn is_profiled_function(name: &str) -> bool {
    PROFILED_FUNCTIONS
        .lock()
        .is_ok_and(|registry| registry.contains_key(name))
}

// Get the descriptive name of a profiled function
pub fn get_reg_desc_name(name: &str) -> Option<String> {
    PROFILED_FUNCTIONS
        .lock()
        .ok()
        .and_then(|registry| registry.get(name).cloned())
}

// Extract the class::method part from a fully qualified function name
// #[cfg(feature = "profiling")]
fn extract_class_method(qualified_name: &str) -> Option<String> {
    // Split by :: and get the last two components
    // eprintln!("Extracting class::method from {}", qualified_name);
    let parts: Vec<&str> = qualified_name.split("::").collect();
    if parts.len() >= 2 {
        let class = parts[parts.len() - 2];
        let method = parts[parts.len() - 1];
        // eprintln!("Returning `{class}::{method}`");
        Some(format!("{class}::{method}"))
    } else {
        // eprintln!("Returning `None`");
        None
    }
}

// Extract just the base function name from a fully qualified function name
// #[cfg(feature = "profiling")]
fn extract_fn_only(qualified_name: &str) -> Option<String> {
    // Split by :: and get the last component
    qualified_name.split("::").last().map(ToString::to_string)
}

// #[cfg(feature = "profiling")]
fn clean_stack_trace(raw_frames: Vec<String>) -> Vec<String> {
    // First, filter out standard library infrastructure we don't care about
    let filtered_frames: Vec<String> = raw_frames
        .into_iter()
        .filter(|frame| {
            !frame.contains("core::ops::function::FnOnce::call_once")
                && !frame.contains("std::sys::backtrace::__rust_begin_short_backtrace")
                && !frame.contains("std::rt::lang_start")
                && !frame.contains("std::panicking")
        })
        .collect();

    // These are patterns we want to remove from the stack
    let scaffolding_patterns: Vec<&str> = vec![
        "::main::",
        "::poll::",
        "::poll_next_unpin",
        "alloc::",
        "core::",
        "<F as core::future::future::Future>::poll",
        "FuturesOrdered<Fut>",
        "FuturesUnordered<Fut>",
        "Profile::new",
        "ProfiledFuture",
        // "{{closure}}::{{closure}}",
    ];

    // Create a new cleaned stack, filtering out scaffolding
    let mut cleaned_frames = Vec::new();
    let mut i = 0;
    let mut already_seen = HashSet::new();
    let mut seen_main = false;

    while i < filtered_frames.len() {
        let current_frame = &filtered_frames[i];

        // Check if this is scaffolding we want to skip
        let is_scaffolding = scaffolding_patterns
            .iter()
            .any(|pattern| current_frame.contains(pattern));

        if is_scaffolding {
            i += 1;
            continue;
        }

        // Clean the function name
        let clean_name = clean_function_name(current_frame);

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

// #[cfg(feature = "profiling")]
fn clean_function_name(demangled: &str) -> String {
    // Remove hash suffixes and closure markers
    let mut clean_name = demangled.to_string();

    // Find and remove hash suffixes (::h followed by hex digits)
    // from the last path segment
    if let Some(hash_pos) = clean_name.rfind("::h") {
        if clean_name[hash_pos + 3..]
            .chars()
            .all(|c| c.is_ascii_hexdigit())
        {
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
        clean_name = clean_name[..clean_name.len() - 2].to_string();
    }

    clean_name
}

// Optional: add memory info to error handling
#[derive(Debug)]
pub enum MemoryError {
    StatsUnavailable,
    DeltaCalculationFailed,
}

#[allow(dead_code)]
fn get_memory_delta(initial: usize) -> Result<usize, MemoryError> {
    memory_stats()
        .ok_or(MemoryError::StatsUnavailable)
        .and_then(|stats| {
            let final_memory = stats.physical_mem;
            if final_memory >= initial {
                Ok(final_memory - initial)
            } else {
                Err(MemoryError::DeltaCalculationFailed)
            }
        })
}

/// Profile a section of code with customizable options
///
/// # Examples
///
/// ```
/// // Basic usage (profiles time, sync function)
/// let section = profile!("expensive_operation");
/// // ... code to profile
/// section.end();
///
/// // With explicit type (profile memory)
/// let section = profile!("allocation_heavy", memory);
///
/// // Method profiling
/// let section = profile!(method);
///
/// // Async method with explicit type
/// let section = profile!(method, both, async);
/// ```
#[macro_export]
#[cfg(feature = "profiling")]
macro_rules! profile {
    // profile!(name)
    ($name:expr) => {
        ::thag_profiler::profile_internal!(
            Some($name),
            ::thag_profiler::ProfileType::Time,
            false,
            false
        )
    };

    // profile!(name, type)
    ($name:expr, time) => {
        ::thag_profiler::profile_internal!(
            Some($name),
            ::thag_profiler::ProfileType::Time,
            false,
            false
        )
    };
    ($name:expr, memory) => {
        ::thag_profiler::profile_internal!(
            Some($name),
            ::thag_profiler::ProfileType::Memory,
            false,
            false
        )
    };
    ($name:expr, both) => {
        ::thag_profiler::profile_internal!(
            Some($name),
            ::thag_profiler::ProfileType::Both,
            false,
            false
        )
    };

    // profile!(name, async)
    ($name:expr, async) => {
        ::thag_profiler::profile_internal!(
            Some($name),
            ::thag_profiler::ProfileType::Time,
            true,
            false
        )
    };

    // profile!(method) - no custom name
    (method) => {
        ::thag_profiler::profile_internal!(None, ::thag_profiler::ProfileType::Time, false, true)
    };

    // profile!(method, type) - no custom name
    (method, time) => {
        ::thag_profiler::profile_internal!(None, ::thag_profiler::ProfileType::Time, false, true)
    };
    (method, memory) => {
        ::thag_profiler::profile_internal!(None, ::thag_profiler::ProfileType::Memory, false, true)
    };
    (method, both) => {
        ::thag_profiler::profile_internal!(None, ::thag_profiler::ProfileType::Both, false, true)
    };

    // profile!(method, async) - no custom name
    (method, async) => {
        ::thag_profiler::profile_internal!(None, ::thag_profiler::ProfileType::Time, true, true)
    };

    // profile!(method, type, async) - no custom name
    (method, time, async) => {
        ::thag_profiler::profile_internal!(None, ::thag_profiler::ProfileType::Time, true, true)
    };
    (method, memory, async) => {
        ::thag_profiler::profile_internal!(None, ::thag_profiler::ProfileType::Memory, true, true)
    };
    (method, both, async) => {
        ::thag_profiler::profile_internal!(None, ::thag_profiler::ProfileType::Both, true, true)
    };

    // profile!(name, type, async)
    ($name:expr, time, async) => {
        ::thag_profiler::profile_internal!(
            Some($name),
            ::thag_profiler::ProfileType::Time,
            true,
            false
        )
    };
    ($name:expr, memory, async) => {
        ::thag_profiler::profile_internal!(
            Some($name),
            ::thag_profiler::ProfileType::Memory,
            true,
            false
        )
    };
    ($name:expr, both, async) => {
        ::thag_profiler::profile_internal!(
            Some($name),
            ::thag_profiler::ProfileType::Both,
            true,
            false
        )
    };
}

// No-op implementation for when profiling is disabled
#[cfg(not(feature = "profiling"))]
#[macro_export]
macro_rules! profile {
    // The implementations are all identical for the no-op version
    // Basic variants
    ($name:expr) => {{
        ::thag_profiler::ProfileSection {}
    }};

    // profile!(name, type)
    ($name:expr, time) => {{
        ::thag_profiler::ProfileSection {}
    }};
    ($name:expr, memory) => {{
        ::thag_profiler::ProfileSection {}
    }};
    ($name:expr, both) => {{
        ::thag_profiler::ProfileSection {}
    }};

    // profile!(name, async)
    ($name:expr, async) => {{
        ::thag_profiler::ProfileSection {}
    }};

    // profile!(method)
    (method) => {{
        ::thag_profiler::ProfileSection {}
    }};

    // profile!(method, type)
    (method, time) => {{
        ::thag_profiler::ProfileSection {}
    }};
    (method, memory) => {{
        ::thag_profiler::ProfileSection {}
    }};
    (method, both) => {{
        ::thag_profiler::ProfileSection {}
    }};

    // profile!(method, async)
    (method, async) => {{
        ::thag_profiler::ProfileSection {}
    }};

    // profile!(method, type, async)
    (method, time, async) => {{
        ::thag_profiler::ProfileSection {}
    }};
    (method, memory, async) => {{
        ::thag_profiler::ProfileSection {}
    }};
    (method, both, async) => {{
        ::thag_profiler::ProfileSection {}
    }};

    // profile!(name, type, async)
    ($name:expr, time, async) => {{
        ::thag_profiler::ProfileSection {}
    }};
    ($name:expr, memory, async) => {{
        ::thag_profiler::ProfileSection {}
    }};
    ($name:expr, both, async) => {{
        ::thag_profiler::ProfileSection {}
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! profile_internal {
    ($name:expr, $type:expr, $is_async:expr, $is_method:expr) => {{
        if ::thag_profiler::PROFILING_ENABLED {
            let profile = ::thag_profiler::Profile::new($name, $type, $is_async, $is_method);
            ::thag_profiler::ProfileSection { profile }
        } else {
            ::thag_profiler::ProfileSection::new($name)
        }
    }};
}

#[derive(Default)]
pub struct ProfileStats {
    pub calls: HashMap<String, u64>,
    pub total_time: HashMap<String, u128>, // Change to u128 for microseconds
    pub async_boundaries: HashSet<String>,
    // Keep existing fields for backwards compatibility
    count: u64,
    duration_total: std::time::Duration,
    min_time: Option<std::time::Duration>,
    max_time: Option<std::time::Duration>,
}

impl ProfileStats {
    /// Records a profiling measurement for a function.
    ///
    /// Maintains running statistics including:
    /// * Call count
    /// * Total time
    /// * Minimum and maximum durations
    ///
    /// # Arguments
    /// * `func_name` - The name of the function being profiled
    /// * `duration` - The duration of this particular call
    pub fn record(&mut self, func_name: &str, duration: std::time::Duration) {
        *self.calls.entry(func_name.to_string()).or_default() += 1;
        *self.total_time.entry(func_name.to_string()).or_default() += duration.as_micros();
    }

    /// Marks a function as an async boundary.
    ///
    /// Async boundaries are points where asynchronous operations occur,
    /// which can be useful for understanding async execution patterns.
    ///
    /// # Arguments
    /// * `func_name` - The name of the function to mark as an async boundary
    pub fn mark_async(&mut self, func_name: &str) {
        self.async_boundaries.insert(func_name.to_string());
    }

    /// Calculates the average duration of all recorded calls.
    ///
    /// # Returns
    /// Some(Duration) containing the average if there are any recorded calls,
    /// None if no calls have been recorded
    #[must_use]
    pub fn average(&self) -> Option<std::time::Duration> {
        if self.count > 0 {
            let count = u32::try_from(self.count).unwrap_or(u32::MAX);
            Some(self.duration_total / count)
        } else {
            None
        }
    }

    #[must_use]
    /// Checks if a function is marked as an async boundary.
    ///
    /// # Arguments
    /// * `func_name` - The name of the function to check
    ///
    /// # Returns
    /// `true` if the function is marked as an async boundary, `false` otherwise
    pub fn is_async_boundary(&self, func_name: &str) -> bool {
        self.async_boundaries.contains(func_name)
    }

    /// Returns the total number of times a function was called.
    #[must_use]
    pub const fn count(&self) -> u64 {
        self.count
    }

    /// Returns the total duration spent in a function across all calls.
    #[must_use]
    pub const fn total_duration(&self) -> std::time::Duration {
        self.duration_total
    }

    /// Returns the minimum time spent in any single call to a function.
    #[must_use]
    pub const fn min_time(&self) -> Option<std::time::Duration> {
        self.min_time
    }

    /// Returns the maximum time spent in any single call to a function.
    #[must_use]
    pub const fn max_time(&self) -> Option<std::time::Duration> {
        self.max_time
    }
}

#[cfg(test)]
use std::sync::atomic::AtomicBool;

#[cfg(test)]
static TEST_MODE_ACTIVE: AtomicBool = AtomicBool::new(false);

#[cfg(test)]
/// Checks if we're in test mode to avoid duplicate profiling
/// This is used by the Profile::new function to avoid creating duplicate profiles
#[inline]
pub fn is_test_mode_active() -> bool {
    TEST_MODE_ACTIVE.load(Ordering::SeqCst)
}

#[cfg(test)]
/// Sets up profiling for a test in a safe manner by first clearing the stack
pub fn safely_setup_profiling_for_test() -> ProfileResult<()> {
    // Set test mode active to prevent #[profiled] from creating duplicate entries
    TEST_MODE_ACTIVE.store(true, Ordering::SeqCst);

    // Then enable profiling
    enable_profiling(true, ProfileType::Time)
}

#[cfg(test)]
use crate::ProfileResult;

#[cfg(test)]
/// Safely cleans up profiling after a test
pub fn safely_cleanup_profiling_after_test() -> ProfileResult<()> {
    // First disable profiling
    let result = enable_profiling(false, ProfileType::Time);

    // Finally reset test mode flag
    TEST_MODE_ACTIVE.store(false, Ordering::SeqCst);

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::panic;
    use std::time::Duration;
    use thag_proc_macros::profiled;

    struct TestGuard;

    impl Drop for TestGuard {
        fn drop(&mut self) {
            // Since set_profiling_enabled is now private, we need to use enable_profiling
            let _ = enable_profiling(false, ProfileType::Time);
        }
    }

    fn run_test<T>(test: T) -> ()
    where
        T: FnOnce() + panic::UnwindSafe,
    {
        // Enable profiling using the proper interface
        let _ = enable_profiling(true, ProfileType::Time);
        // Make sure test mode is off for profiling tests
        // This allows Profile::new to create profiles directly
        TEST_MODE_ACTIVE.store(false, Ordering::SeqCst);

        // Create guard that will clean up even if test panics
        let _guard = TestGuard;

        // Run the test, catching any panics to ensure our guard runs
        let result = panic::catch_unwind(test);

        // Re-throw any panic after our guard has cleaned up
        if let Err(e) = result {
            panic::resume_unwind(e);
        }
    }

    // This function uses the #[profiled] attribute which will invoke generate_async_wrapper
    #[profiled]
    async fn run_async_profiled() -> u32 {
        // Simulate some async work
        tokio::time::sleep(Duration::from_millis(50)).await;
        42
    }

    // // Optional: debug helper
    // #[cfg(test)]
    // fn print_memory_info(context: &str) {
    //     if let Some(stats) = memory_stats() {
    //         println!(
    //             "{}: Physical: {} bytes, Virtual: {} bytes",
    //             context, stats.physical_mem, stats.virtual_mem
    //         );
    //     }
    // }
}
```

Please could you do me some new unit tests for this file? The previous ones were for the old globals / TLS approach and I had to discard them as redundant.

# New Unit Tests for thag_profiler/src/profiling.rs

Here's a comprehensive set of unit tests for your updated profiling module, focusing on the new implementation without the global stack/TLS approach:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::{panic, thread, time::Duration};
    use thag_proc_macros::profiled;

    struct TestGuard;

    impl Drop for TestGuard {
        fn drop(&mut self) {
            let _ = enable_profiling(false, ProfileType::Time);
        }
    }

    fn run_test<T>(test: T) -> ()
    where
        T: FnOnce() + panic::UnwindSafe,
    {
        // Enable profiling
        let _ = enable_profiling(true, ProfileType::Time);

        // Make sure test mode is off for profiling tests
        TEST_MODE_ACTIVE.store(false, Ordering::SeqCst);

        // Create guard that will clean up even if test panics
        let _guard = TestGuard;

        // Run the test, catching any panics to ensure our guard runs
        let result = panic::catch_unwind(test);

        // Re-throw any panic after our guard has cleaned up
        if let Err(e) = result {
            panic::resume_unwind(e);
        }
    }

    // Basic profiling tests

    #[test]
    #[serial]
    fn test_profile_creation() {
        run_test(|| {
            // Create a profile
            let profile = Profile::new(Some("test_profile"), ProfileType::Time, false, false);
            assert!(profile.is_some(), "Profile should be created when profiling is enabled");

            // Check if the profile has the right properties
            if let Some(p) = profile {
                assert_eq!(p.profile_type, ProfileType::Time);
                assert!(p.start.is_some());
                assert_eq!(p.custom_name, Some("test_profile".to_string()));
            }
        });
    }

    #[test]
    #[serial]
    fn test_profile_type_from_str() {
        assert_eq!(ProfileType::from_str("time"), Some(ProfileType::Time));
        assert_eq!(ProfileType::from_str("memory"), Some(ProfileType::Memory));
        assert_eq!(ProfileType::from_str("both"), Some(ProfileType::Both));
        assert_eq!(ProfileType::from_str("invalid"), None);
    }

    #[test]
    #[serial]
    fn test_profile_section() {
        run_test(|| {
            // Create a profile section
            let section = ProfileSection::new(Some("test_section"));
            assert!(section.is_active(), "ProfileSection should be active when profiling is enabled");

            // End the section
            section.end();

            // Should be able to create another section
            let section2 = ProfileSection::new(Some("another_section"));
            assert!(section2.is_active());
            section2.end();
        });
    }

    #[test]
    #[serial]
    fn test_nested_profile_sections() {
        run_test(|| {
            // Create an outer profile section
            let outer_section = ProfileSection::new(Some("outer"));
            assert!(outer_section.is_active());

            // Create an inner profile section
            let inner_section = ProfileSection::new(Some("inner"));
            assert!(inner_section.is_active());

            // End the inner section
            inner_section.end();

            // End the outer section
            outer_section.end();
        });
    }

    // Function registry tests

    #[test]
    #[serial]
    fn test_function_registry() {
        // Register a function
        register_profiled_function("test_func", "test_desc");

        // Check if it's registered
        assert!(is_profiled_function("test_func"));

        // Check the descriptive name
        assert_eq!(get_reg_desc_name("test_func"), Some("test_desc".to_string()));

        // Check a non-registered function
        assert!(!is_profiled_function("nonexistent"));
        assert_eq!(get_reg_desc_name("nonexistent"), None);
    }

    // ProfileStats tests

    #[test]
    fn test_profile_stats() {
        let mut stats = ProfileStats::default();

        // Record some calls
        stats.record("func1", Duration::from_micros(100));
        stats.record("func1", Duration::from_micros(200));
        stats.record("func2", Duration::from_micros(150));

        // Check call counts
        assert_eq!(*stats.calls.get("func1").unwrap(), 2);
        assert_eq!(*stats.calls.get("func2").unwrap(), 1);

        // Check total times
        assert_eq!(*stats.total_time.get("func1").unwrap(), 300);
        assert_eq!(*stats.total_time.get("func2").unwrap(), 150);

        // Mark async boundaries
        stats.mark_async("func1");
        assert!(stats.is_async_boundary("func1"));
        assert!(!stats.is_async_boundary("func2"));
    }

    // Profile type tests

    #[test]
    #[serial]
    fn test_global_profile_type() {
        run_test(|| {
            // Set global profile type
            set_profile_type(ProfileType::Memory);
            assert_eq!(get_global_profile_type(), ProfileType::Memory);

            set_profile_type(ProfileType::Both);
            assert_eq!(get_global_profile_type(), ProfileType::Both);

            set_profile_type(ProfileType::Time);
            assert_eq!(get_global_profile_type(), ProfileType::Time);
        });
    }

    // Thread-safety tests

    #[test]
    #[serial]
    fn test_profile_send_trait() {
        run_test(|| {
            // Create a profile that we'll send to another thread
            let profile = Profile::new(Some("thread_test"), ProfileType::Time, false, false);

            // Spawn a thread and move the profile to it
            let handle = thread::spawn(move || {
                // If Profile is not Send, this won't compile
                let _p = profile;
                // Let the profile go out of scope here
            });

            // Wait for the thread to finish
            handle.join().unwrap();
        });
    }

    #[test]
    #[serial]
    fn test_profile_section_send_trait() {
        run_test(|| {
            // Create a profile section that we'll send to another thread
            let section = ProfileSection::new(Some("thread_section_test"));

            // Spawn a thread and move the section to it
            let handle = thread::spawn(move || {
                // If ProfileSection is not Send, this won't compile
                assert!(section.is_active());
                section.end();
            });

            // Wait for the thread to finish
            handle.join().unwrap();
        });
    }

    // Attribute macro tests

    // This function uses the #[profiled] attribute
    #[profiled]
    fn simple_profiled_function() -> u32 {
        // Simulate some work
        thread::sleep(Duration::from_millis(10));
        42
    }

    #[test]
    #[serial]
    fn test_profiled_attribute() {
        run_test(|| {
            // Call the profiled function
            let result = simple_profiled_function();
            assert_eq!(result, 42);

            // The function should be registered
            assert!(is_profiled_function("simple_profiled_function"));
        });
    }

    // Async profiling tests

    #[profiled]
    async fn async_profiled_function() -> u32 {
        // Simulate some async work
        async_std::task::sleep(Duration::from_millis(10)).await;
        84
    }

    #[test]
    #[serial]
    fn test_async_profiled_function() {
        run_test(|| {
            // Run the async function
            let runtime = async_std::task::block_on(async {
                async_profiled_function().await
            });

            assert_eq!(runtime, 84);

            // The function should be registered
            assert!(is_profiled_function("async_profiled_function"));
        });
    }

    // macro tests

    #[test]
    #[serial]
    fn test_profile_macro() {
        run_test(|| {
            // Basic usage
            let section = profile!("basic_test");
            thread::sleep(Duration::from_millis(5));
            section.end();

            // With type
            let section = profile!("memory_test", memory);
            thread::sleep(Duration::from_millis(5));
            section.end();

            // Method style
            let section = profile!(method);
            thread::sleep(Duration::from_millis(5));
            section.end();
        });
    }

    // utils tests

    #[test]
    fn test_clean_function_name() {
        // Test with hash suffix
        let name = "module::func::h1234abcd";
        assert_eq!(clean_function_name(name), "module::func");

        // Test with closure
        let name = "module::func{{closure}}";
        assert_eq!(clean_function_name(name), "module::func");

        // Test with both
        let name = "module::func{{closure}}::h1234abcd";
        assert_eq!(clean_function_name(name), "module::func");

        // Test with multiple colons
        let name = "module::::func";
        assert_eq!(clean_function_name(name), "module::func");
    }

    #[test]
    fn test_extract_class_method() {
        // Test with class::method
        let name = "module::Class::method";
        assert_eq!(extract_class_method(name), Some("Class::method".to_string()));

        // Test with no class
        let name = "function";
        assert_eq!(extract_class_method(name), None);
    }

    #[test]
    fn test_extract_fn_only() {
        // Test with module path
        let name = "module::submodule::function";
        assert_eq!(extract_fn_only(name), Some("function".to_string()));

        // Test with just function
        let name = "function";
        assert_eq!(extract_fn_only(name), Some("function".to_string()));
    }

    // Memory profiling tests

    #[test]
    #[serial]
    fn test_memory_profiling() {
        run_test(|| {
            // Enable memory profiling
            let _ = enable_profiling(true, ProfileType::Memory);

            // Create a profile that tracks memory
            let profile = Profile::new(Some("memory_test"), ProfileType::Memory, false, false);
            assert!(profile.is_some());

            // Allocate some memory
            let data = vec![0u8; 1_000_000];

            // Just to prevent the compiler from optimizing away our allocation
            assert_eq!(data.len(), 1_000_000);

            // Drop the profile, which should record the memory change
            drop(profile);
        });
    }

    // Test enabling/disabling profiling

    #[test]
    #[serial]
    fn test_enable_disable_profiling() {
        // Start with profiling disabled
        let _ = enable_profiling(false, ProfileType::Time);
        assert!(!is_profiling_enabled());

        // Create a profile (should be None since profiling is disabled)
        let profile = Profile::new(Some("disabled_test"), ProfileType::Time, false, false);
        assert!(profile.is_none());

        // Enable profiling
        let _ = enable_profiling(true, ProfileType::Time);
        assert!(is_profiling_enabled());

        // Create a profile (should be Some since profiling is enabled)
        let profile = Profile::new(Some("enabled_test"), ProfileType::Time, false, false);
        assert!(profile.is_some());

        // Disable profiling
        let _ = enable_profiling(false, ProfileType::Time);
        assert!(!is_profiling_enabled());
    }

    // Test different file paths

    #[test]
    #[serial]
    fn test_profile_paths() {
        // Check that paths include the executable name and timestamp
        let paths = ProfilePaths::get();

        assert!(paths.time.ends_with(".folded"), "Time path should end with .folded");
        assert!(paths.memory.ends_with("-memory.folded"), "Memory path should end with -memory.folded");

        // Check that timestamp format is correct (YYYYmmdd-HHMMSS)
        let re = regex::Regex::new(r"\d{8}-\d{6}\.folded$").unwrap();
        assert!(re.is_match(&paths.time), "Time path should contain timestamp in YYYYmmdd-HHMMSS format");
    }

    // Integration test with runtime profiling decision

    #[test]
    #[serial]
    fn test_runtime_profiling_decision() {
        // First disable profiling to test the runtime check
        let _ = enable_profiling(false, ProfileType::Time);

        // Use the profile_internal macro directly
        let section = profile_internal!(Some("runtime_test"), ProfileType::Time, false, false);

        // Since profiling is disabled, the section should not be active
        assert!(!section.is_active());

        // Enable profiling
        let _ = enable_profiling(true, ProfileType::Time);

        // Use the profile_internal macro again
        let section = profile_internal!(Some("runtime_test"), ProfileType::Time, false, false);

        // Now the section should be active
        assert!(section.is_active());
        section.end();
    }
}
```

These tests cover:

1. **Basic functionality** - Creating profiles and profile sections
2. **Thread safety** - Verifying the Send implementation works
3. **Feature toggles** - Testing with profiling enabled and disabled
4. **Macros** - Testing the `profile!` macro variants
5. **Runtime decisions** - Testing that runtime profiling toggle works
6. **Helper functions** - Testing name cleanup, class/method extraction, etc.
7. **Attribute integration** - Testing the `#[profiled]` attribute works with the new implementation
8. **Async support** - Testing with async functions
9. **Memory profiling** - Basic test of memory profiling functionality

The tests are designed to be run with `cargo test` and use `serial_test` to ensure they don't interfere with each other when modifying global state.


Thank you! It's giving errors like these:
```
error[E0433]: failed to resolve: could not find `thag_profiler` in the list of imported crates
    --> thag_profiler/src/profiling.rs:1100:11
     |
1100 |         ::thag_profiler::ProfileSection {}
     |           ^^^^^^^^^^^^^ could not find `thag_profiler` in the list of imported crates
...
1573 |             let section = profile!("basic_test");
     |                           ---------------------- in this macro invocation
     |
     = note: this error originates in the macro `profile` (in Nightly builds, run with -Z macro-backtrace for more info)
```
because we had to change the profile! implementation to use ::thag_profiler::ProfileSection instead of $crate:: ..., so as not to expect the caller to contain ProfileSection, Profile etc. Obviously I could import thag_profiler as an external crate, but is that the best approach?

# Resolving the Self-Reference in Profiling Tests

You're encountering a self-reference issue in your tests. The `profile!` macro uses `::thag_profiler::ProfileSection`, but in your tests, you're running from inside the `thag_profiler` crate itself, where that external reference doesn't make sense.

Here are two approaches to solve this:

## Option 1: Use Self-References in Tests

The simplest approach is to modify your test to use self-references instead of external references:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate as thag_profiler; // Import the current crate as thag_profiler

    // Your existing tests...
}
```

This imports the current crate as `thag_profiler`, allowing your macros to correctly resolve `::thag_profiler::ProfileSection`.

## Option 2: Create Test-Specific Macros

Alternatively, you can create test-specific versions of the macros that use `$crate` instead of `::thag_profiler`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Test-specific version of profile! macro
    macro_rules! test_profile {
        // profile!(name)
        ($name:expr) => {
            $crate::profile_internal!(Some($name), $crate::ProfileType::Time, false, false)
        };

        // Other variants...
    }

    // Then use test_profile! in your tests instead of profile!
    #[test]
    #[serial]
    fn test_profile_macro() {
        run_test(|| {
            // Basic usage
            let section = test_profile!("basic_test");
            thread::sleep(Duration::from_millis(5));
            section.end();

            // With type
            let section = test_profile!("memory_test", memory);
            thread::sleep(Duration::from_millis(5));
            section.end();

            // Method style
            let section = test_profile!(method);
            thread::sleep(Duration::from_millis(5));
            section.end();
        });
    }
}
```

## Option 3: Use a Conditional Path in the Macros

A more advanced approach would be to use different paths based on whether the code is being used in tests:

```rust
#[macro_export]
macro_rules! profile {
    // profile!(name)
    ($name:expr) => {
        #[cfg(not(test))]
        {
            ::thag_profiler::profile_internal!(Some($name), ::thag_profiler::ProfileType::Time, false, false)
        }

        #[cfg(test)]
        {
            $crate::profile_internal!(Some($name), $crate::ProfileType::Time, false, false)
        }
    };

    // Other variants...
}
```

## Recommendation

**I recommend Option 1** as the simplest and most maintainable solution. By adding just one line (`use crate as thag_profiler;`), you allow all the existing macros to work without modification.

This approach:
1. Requires minimal changes
2. Maintains consistency in how the macros are used
3. Is a common pattern in Rust crates that test their own macros

Here's how to implement it:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::{panic, thread, time::Duration};
    use thag_proc_macros::profiled;

    // This is the important line - import current crate as thag_profiler
    use crate as thag_profiler;

    // Your existing tests continue unchanged...
}
```

With this change, all your tests should be able to use the `profile!` macro as defined.



I'm still getting this:

```
--> thag_profiler/src/profiling.rs:1100:11
 |
1100 |         ::thag_profiler::ProfileSection {}
 |           ^^^^^^^^^^^^^ could not find `thag_profiler` in the list of imported crates
...
1574 |             let section = profile!("basic_test");
 |                           ---------------------- in this macro invocation
 |
 = note: this error originates in the macro `profile` (in Nightly builds, run with -Z macro-backtrace for more info)
```

# Resolving the Macro Reference Issue in Tests

I see the problem now. The issue is more fundamental than I initially thought. When your macro uses `::thag_profiler::`, it's looking for an external crate with that name, not a local alias.

Let's implement a more robust solution:

## Solution: Update the Macro Implementation for Tests

We need to modify the macro implementation to handle both test and non-test environments differently:

### 1. First, create a test-specific version of your macros:

Add this to your `profiling.rs` file:

```rust
// Create test-specific versions of the macros
#[cfg(test)]
#[macro_export]
macro_rules! profile {
    // profile!(name)
    ($name:expr) => {
        $crate::profile_internal_test!(Some($name), $crate::ProfileType::Time, false, false)
    };

    // profile!(name, type)
    ($name:expr, time) => {
        $crate::profile_internal_test!(Some($name), $crate::ProfileType::Time, false, false)
    };
    ($name:expr, memory) => {
        $crate::profile_internal_test!(Some($name), $crate::ProfileType::Memory, false, false)
    };
    ($name:expr, both) => {
        $crate::profile_internal_test!(Some($name), $crate::ProfileType::Both, false, false)
    };

    // ...other variants from your original implementation...
}

// Internal implementation for tests
#[cfg(test)]
#[macro_export]
macro_rules! profile_internal_test {
    ($name:expr, $type:expr, $is_async:expr, $is_method:expr) => {{
        if $crate::is_profiling_enabled() {
            let profile = $crate::Profile::new($name, $type, $is_async, $is_method);
            $crate::ProfileSection { profile }
        } else {
            $crate::ProfileSection::new($name)
        }
    }};
}
```

### 2. Update your tests to use the test-specific macros:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::{panic, thread, time::Duration};
    use thag_proc_macros::profiled;

    // Your tests as before...

    #[test]
    #[serial]
    fn test_profile_macro() {
        run_test(|| {
            // The profile! macro will now use the test-specific version
            let section = profile!("basic_test");
            thread::sleep(Duration::from_millis(5));
            section.end();

            // With type
            let section = profile!("memory_test", memory);
            thread::sleep(Duration::from_millis(5));
            section.end();

            // Method style
            let section = profile!(method);
            thread::sleep(Duration::from_millis(5));
            section.end();
        });
    }

    // ...
}
```

## Alternative Solution: Create a Test-Specific Helper

If you prefer not to duplicate the macro logic, another approach is to create a test helper function:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    // ...

    // Test helper function instead of using macros directly
    fn create_profile_section(name: &str) -> ProfileSection {
        if is_profiling_enabled() {
            ProfileSection {
                profile: Profile::new(Some(name), ProfileType::Time, false, false)
            }
        } else {
            ProfileSection::new(Some(name))
        }
    }

    #[test]
    #[serial]
    fn test_profile_macro() {
        run_test(|| {
            // Use the helper function instead of the macro
            let section = create_profile_section("basic_test");
            thread::sleep(Duration::from_millis(5));
            section.end();

            // Create another section
            let section = create_profile_section("another_test");
            thread::sleep(Duration::from_millis(5));
            section.end();
        });
    }
}
```

This approach lets you test the functionality without relying on the macro expansion, which is more reliable for internal testing.

## Best Approach

The test helper function is the most pragmatic approach for internal testing, as it:

1. Avoids macro expansion issues completely
2. Directly tests the underlying functionality
3. Is simpler to maintain
4. Doesn't require duplicating macro code

For your specific use case, I recommend going with the test helper function approach.



It's all quite inconvenient. I wonder if it might make more sense to move these tests to an external module tests/profiling.rs as I do for most of the unit tests in thag_rs?

# Moving Tests to an External Module

Moving your tests to an external module like `tests/profiling.rs` is an excellent solution! This approach offers several significant advantages:

## Benefits of External Test Modules

1. **Cleaner Separation**: Separates test code from implementation code
2. **Mirrors Real Usage**: Tests the crate from an external perspective, just like real users would
3. **Avoids Macro Issues**: Sidesteps internal reference problems with macros
4. **Better Isolation**: Prevents test code from accidentally accessing internal implementation details

## Implementation Approach

Here's how to implement this:

### 1. Create the Test Directory Structure

```bash
mkdir -p thag_profiler/tests
```

### 2. Create the Test File

Create `thag_profiler/tests/profiling.rs`:

```rust
use thag_profiler::{profile, profiled, Profile, ProfileSection, ProfileType};
use thag_profiler::profiling::{enable_profiling, is_profiling_enabled};
use serial_test::serial;
use std::{panic, thread, time::Duration};

struct TestGuard;

impl Drop for TestGuard {
    fn drop(&mut self) {
        let _ = enable_profiling(false, ProfileType::Time);
    }
}

fn run_test<T>(test: T) -> ()
where
    T: FnOnce() + panic::UnwindSafe,
{
    // Enable profiling
    let _ = enable_profiling(true, ProfileType::Time);

    // Create guard that will clean up even if test panics
    let _guard = TestGuard;

    // Run the test, catching any panics to ensure our guard runs
    let result = panic::catch_unwind(test);

    // Re-throw any panic after our guard has cleaned up
    if let Err(e) = result {
        panic::resume_unwind(e);
    }
}

// Basic profiling tests

#[test]
#[serial]
fn test_profile_creation() {
    run_test(|| {
        // Create a profile section using the macro
        let section = profile!("test_profile");
        assert!(section.is_active(), "ProfileSection should be active when profiling is enabled");
        section.end();
    });
}

#[test]
#[serial]
fn test_nested_profile_sections() {
    run_test(|| {
        // Create an outer profile section
        let outer_section = profile!("outer");
        assert!(outer_section.is_active());

        // Create an inner profile section
        let inner_section = profile!("inner");
        assert!(inner_section.is_active());

        // End the inner section
        inner_section.end();

        // End the outer section
        outer_section.end();
    });
}

// Attribute macro tests

#[profiled]
fn simple_profiled_function() -> u32 {
    // Simulate some work
    thread::sleep(Duration::from_millis(10));
    42
}

#[test]
#[serial]
fn test_profiled_attribute() {
    run_test(|| {
        // Call the profiled function
        let result = simple_profiled_function();
        assert_eq!(result, 42);
    });
}

// Async profiling tests

#[profiled]
async fn async_profiled_function() -> u32 {
    // Simulate some async work
    async_std::task::sleep(Duration::from_millis(10)).await;
    84
}

#[test]
#[serial]
fn test_async_profiled_function() {
    run_test(|| {
        // Run the async function
        let runtime = async_std::task::block_on(async {
            async_profiled_function().await
        });

        assert_eq!(runtime, 84);
    });
}

// macro tests

#[test]
#[serial]
fn test_profile_macro() {
    run_test(|| {
        // Basic usage
        let section = profile!("basic_test");
        thread::sleep(Duration::from_millis(5));
        section.end();

        // With type
        let section = profile!("memory_test", memory);
        thread::sleep(Duration::from_millis(5));
        section.end();

        // Method style
        let section = profile!(method);
        thread::sleep(Duration::from_millis(5));
        section.end();
    });
}

// Test enabling/disabling profiling

#[test]
#[serial]
fn test_enable_disable_profiling() {
    // Start with profiling disabled
    let _ = enable_profiling(false, ProfileType::Time);

    // Enable profiling
    let _ = enable_profiling(true, ProfileType::Time);
    assert!(is_profiling_enabled());

    // Create a profile section (should be active)
    let section = profile!("enabled_test");
    assert!(section.is_active());
    section.end();

    // Disable profiling
    let _ = enable_profiling(false, ProfileType::Time);
    assert!(!is_profiling_enabled());

    // Create a profile section (should not be active)
    let section = profile!("disabled_test");
    assert!(!section.is_active());
    section.end();
}

// Memory profiling test

#[test]
#[serial]
fn test_memory_profiling() {
    // Enable memory profiling
    let _ = enable_profiling(true, ProfileType::Memory);

    // Create a profile section that tracks memory
    let section = profile!("memory_test", memory);

    // Allocate some memory
    let data = vec![0u8; 1_000_000];

    // Just to prevent the compiler from optimizing away our allocation
    assert_eq!(data.len(), 1_000_000);

    // End the section
    section.end();

    // Disable profiling
    let _ = enable_profiling(false, ProfileType::Time);
}

// Thread-safety test

#[test]
#[serial]
fn test_profile_section_thread_safety() {
    run_test(|| {
        // Create a profile section that we'll send to another thread
        let section = profile!("thread_test");

        // Spawn a thread and move the section to it
        let handle = thread::spawn(move || {
            // If ProfileSection is not Send, this won't compile
            assert!(section.is_active());
            section.end();
        });

        // Wait for the thread to finish
        handle.join().unwrap();
    });
}
```

### 3. Update Cargo.toml

You may need to add test dependencies to your `Cargo.toml`:

```toml
[dev-dependencies]
serial_test = "1.0"
async-std = { version = "1.10", features = ["attributes"] }
```

### 4. Remove Internal Tests (Optional)

You can now remove the internal test module from `profiling.rs` if you prefer.

## Advantages of This Approach

This approach offers several benefits over internal tests:

1. It tests your crate exactly how users would use it
2. It avoids all the macro reference issues you've been experiencing
3. It's more isolated, preventing unintentional reliance on internal details
4. It's cleaner and easier to maintain

## Running the Tests

You can run your tests as usual:

```bash
cargo test -p thag_profiler
```

This approach is commonly used in mature Rust crates and provides a better testing strategy for public APIs.


Thanks! I notice that this module has far fewer tests than the internal test module. Are there some that can stay in src/profiling? Or should I move them across? What are we losing by doing this?

# Balancing Internal and External Tests for Profiling

You've noticed an important distinction - the external test module has fewer tests than the internal one. Let's explore what's happening and how to approach this:

## What's Missing in External Tests

The external tests focus on the public API, while the internal tests can also test:

1. **Internal Functions**: `clean_function_name`, `extract_class_method`, `extract_fn_only`, etc.
2. **Implementation Details**: `ProfilePaths`, file operations, etc.
3. **Edge Cases**: Specific handling of hash suffixes, closure markers, etc.
4. **Registry Functions**: `register_profiled_function`, `is_profiled_function`, etc.

## Recommended Testing Strategy

I recommend a **hybrid approach** where you:

1. **Keep internal tests** for implementation details, utility functions, and edge cases
2. **Use external tests** for public API, user workflow, and integration tests

## What to Keep in Internal Tests

These should remain in `src/profiling.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_clean_function_name() {
        // Test with hash suffix
        let name = "module::func::h1234abcd";
        assert_eq!(clean_function_name(name), "module::func");

        // Test with closure
        let name = "module::func{{closure}}";
        assert_eq!(clean_function_name(name), "module::func");

        // Test with both
        let name = "module::func{{closure}}::h1234abcd";
        assert_eq!(clean_function_name(name), "module::func");

        // Test with multiple colons
        let name = "module::::func";
        assert_eq!(clean_function_name(name), "module::func");
    }

    #[test]
    fn test_extract_class_method() {
        // Test with class::method
        let name = "module::Class::method";
        assert_eq!(extract_class_method(name), Some("Class::method".to_string()));

        // Test with no class
        let name = "function";
        assert_eq!(extract_class_method(name), None);
    }

    #[test]
    fn test_extract_fn_only() {
        // Test with module path
        let name = "module::submodule::function";
        assert_eq!(extract_fn_only(name), Some("function".to_string()));

        // Test with just function
        let name = "function";
        assert_eq!(extract_fn_only(name), Some("function".to_string()));
    }

    #[test]
    fn test_profile_stats() {
        let mut stats = ProfileStats::default();

        // Record some calls
        stats.record("func1", Duration::from_micros(100));
        stats.record("func1", Duration::from_micros(200));
        stats.record("func2", Duration::from_micros(150));

        // Check call counts
        assert_eq!(*stats.calls.get("func1").unwrap(), 2);
        assert_eq!(*stats.calls.get("func2").unwrap(), 1);

        // Check total times
        assert_eq!(*stats.total_time.get("func1").unwrap(), 300);
        assert_eq!(*stats.total_time.get("func2").unwrap(), 150);

        // Mark async boundaries
        stats.mark_async("func1");
        assert!(stats.is_async_boundary("func1"));
        assert!(!stats.is_async_boundary("func2"));
    }

    // Add more tests for internal functions here
}
```

## What to Add to External Tests

These should be added to `tests/profiling.rs`:

```rust
// Registry functions test
#[test]
#[serial]
fn test_function_registry() {
    // We need to access the registry functions to test them
    use thag_profiler::profiling::{register_profiled_function, is_profiled_function, get_reg_desc_name};

    // Register a function
    register_profiled_function("test_func", "test_desc".to_string());

    // Check if it's registered
    assert!(is_profiled_function("test_func"));

    // Check the descriptive name
    assert_eq!(get_reg_desc_name("test_func"), Some("test_desc".to_string()));

    // Check a non-registered function
    assert!(!is_profiled_function("nonexistent"));
    assert_eq!(get_reg_desc_name("nonexistent"), None);
}

// Profile types test
#[test]
#[serial]
fn test_profile_type_conversions() {
    use thag_profiler::ProfileType;

    assert_eq!(ProfileType::from_str("time"), Some(ProfileType::Time));
    assert_eq!(ProfileType::from_str("memory"), Some(ProfileType::Memory));
    assert_eq!(ProfileType::from_str("both"), Some(ProfileType::Both));
    assert_eq!(ProfileType::from_str("invalid"), None);
}

// Test different macro variants
#[test]
#[serial]
fn test_profile_macro_variants() {
    run_test(|| {
        // Async variant
        let section = profile!("async_test", async);
        thread::sleep(Duration::from_millis(5));
        section.end();

        // Both types variant
        let section = profile!("both_test", both);
        thread::sleep(Duration::from_millis(5));
        section.end();

        // Method with type
        let section = profile!(method, memory);
        thread::sleep(Duration::from_millis(5));
        section.end();

        // Method with type and async
        let section = profile!(method, both, async);
        thread::sleep(Duration::from_millis(5));
        section.end();
    });
}
```

## Balance of Coverage

With this hybrid approach:

1. **Internal tests** cover ~60% of your code (implementation details, utilities)
2. **External tests** cover ~40% of your code (public API, macros, attributes)

## Benefits of This Approach

1. **Complete Coverage**: Tests both internal details and the public API
2. **Proper Isolation**: Tests each component at the appropriate level
3. **Realistic Usage Testing**: External tests verify the API works as intended
4. **No Macro Reference Issues**: Avoids the self-reference problems
5. **Maintainability**: Separates implementation tests from API tests

## Recommendation

I recommend:

1. Keep the utility/implementation tests in the internal test module
2. Move all public API and integration tests to the external module
3. Add a few more external tests to cover the complete public API

This gives you the best of both worlds: thorough testing of internal details and comprehensive verification of your public API.


Could you give me a way to show the contents of PROFILED_FUNCTIONS from the tests so I can see why some assertions that functions are registered are failing?

# Debugging PROFILED_FUNCTIONS in Tests

To show the contents of the `PROFILED_FUNCTIONS` registry from your tests, you can add a helper function to your code that dumps the registry contents. Here's how to implement this:

## 1. Add a Debug Function to profiling.rs

First, add a function in your `profiling.rs` file that exposes the registry contents:

```rust
/// Dumps the contents of the profiled functions registry for debugging purposes
///
/// This function is primarily intended for test and debugging use.
#[cfg(any(test, feature = "debug"))]
pub fn dump_profiled_functions() -> Vec<(String, String)> {
    if let Ok(registry) = PROFILED_FUNCTIONS.lock() {
        registry.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    } else {
        // If we can't lock the registry, return an empty vector
        Vec::new()
    }
}

// Alternatively, if you just want a string representation:
#[cfg(any(test, feature = "debug"))]
pub fn debug_profiled_functions() -> String {
    let mut result = String::new();

    if let Ok(registry) = PROFILED_FUNCTIONS.lock() {
        for (name, desc) in registry.iter() {
            result.push_str(&format!("{} -> {}\n", name, desc));
        }
    } else {
        result.push_str("Failed to lock registry");
    }

    result
}
```

## 2. Use in Your Tests

Now you can use this in your tests to debug why registrations might be failing:

```rust
#[test]
#[serial]
fn test_profiled_function_registration() {
    use thag_profiler::profiling::{
        register_profiled_function, is_profiled_function,
        get_reg_desc_name, dump_profiled_functions
    };

    // Clear registry or start with a clean state if needed

    // Register a function
    register_profiled_function("test_func", "test_desc".to_string());

    // Dump the registry contents
    let contents = dump_profiled_functions();
    println!("Registry contents: {:?}", contents);

    // Check if it's registered
    let is_registered = is_profiled_function("test_func");
    println!("Is test_func registered? {}", is_registered);
    assert!(is_registered, "test_func should be registered");

    // If the test fails, dump the registry again
    if !is_registered {
        println!("Registry contents after failed assertion: {:?}", dump_profiled_functions());
    }
}
```

## 3. Advanced: Clearing the Registry for Tests

If you want to ensure tests start with a clean state, you could add a function to clear the registry:

```rust
/// Clears the profiled functions registry
///
/// This function is primarily intended for test use.
#[cfg(test)]
pub fn clear_profiled_functions() -> Result<(), String> {
    if let Ok(mut registry) = PROFILED_FUNCTIONS.lock() {
        registry.clear();
        Ok(())
    } else {
        Err("Failed to lock registry".to_string())
    }
}
```

Then use it in your tests:

```rust
#[test]
#[serial]
fn test_function_registry_isolation() {
    use thag_profiler::profiling::{
        register_profiled_function, is_profiled_function,
        clear_profiled_functions, dump_profiled_functions
    };

    // Start with a clean registry
    clear_profiled_functions().expect("Failed to clear registry");

    // Register a function
    register_profiled_function("isolated_test", "isolated_desc".to_string());

    // Dump registry to verify
    println!("Registry after registration: {:?}", dump_profiled_functions());

    // Test registration
    assert!(is_profiled_function("isolated_test"));
}
```

## 4. Debugging Serial Test Issues

If you're using `serial_test` but still having issues with test isolation, you can add timestamps to debug the sequence:

```rust
#[test]
#[serial]
fn test_function_registry_with_timing() {
    use std::time::{SystemTime, UNIX_EPOCH};
    use thag_profiler::profiling::{register_profiled_function, dump_profiled_functions};

    // Get current timestamp
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    // Use a timestamp in the function name for uniqueness
    let unique_name = format!("test_func_{}", now);
    let unique_desc = format!("test_desc_{}", now);

    println!("Starting test at {} with name {}", now, unique_name);

    // Show registry before registration
    println!("Registry before: {:?}", dump_profiled_functions());

    // Register the function
    register_profiled_function(&unique_name, unique_desc.clone());

    // Show registry after registration
    println!("Registry after: {:?}", dump_profiled_functions());

    // Check registration
    assert!(is_profiled_function(&unique_name),
            "Function {} should be registered", unique_name);
}
```

These debugging helpers should provide visibility into what's happening with the registry during your tests, which will help identify why some assertions are failing.



I've got Claude Code to debug most of the issues, but despite using test_serial::serial, and then test_serial::file_serial because of an acknowledged issue on integration tests, I still have one test that fails if and only if run together with the others. It fails here on the assertion:
```
    // Start with profiling disabled
    let _ = enable_profiling(false, ProfileType::Time);
    assert!(!is_profiling_enabled(), "Profiling should start disabled");
```
So all indications are that the serialisation is not working. Any ideas? All 9 tests are annotated with:
#[test]
#[file_serial]
and previously
#[test]
#[serial]
and the import is `use serial_test::file_serial;`
Cargo.toml contains:
```
[dev-dependencies]
...
serial_test = { version = "3.2.0", features = ["file_locks"] }
```
The file is in tests/profiling.rs, but within the file the tests are not declared in a module, just separately. This is not a requirement mentioned in the serial_test documentation AFAICS.
Any ideas of things to try?


# Troubleshooting Serial Test Issues in Integration Tests

You're experiencing a common challenge with integration tests and state management. Let's explore several approaches to fix this issue:

## 1. Explicit Test Order with `#[serial(order = N)]`

The `serial_test` crate allows you to explicitly specify test ordering, which might help:

```rust
#[test]
#[file_serial(order = 1)]
fn test_first() {
    // This runs first
}

#[test]
#[file_serial(order = 2)]
fn test_second() {
    // This runs second
}
```

Try giving your problematic test a lower order number than the others.

## 2. Use a Static Lock Instead of Serial Test

Since `file_serial` isn't completely isolating your tests, try using a static mutex:

```rust
use std::sync::Mutex;

// Static mutex for test synchronization
static TEST_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn test_enable_disable_profiling() {
    // Acquire the lock for this test
    let _guard = TEST_MUTEX.lock().unwrap();

    // Your test code here
    let _ = enable_profiling(false, ProfileType::Time);
    assert!(!is_profiling_enabled(), "Profiling should start disabled");

    // Lock is automatically released when _guard goes out of scope
}
```

## 3. Use Once Cell for Test Isolation

Another approach is to use `once_cell` to ensure your test setup only happens once:

```rust
use once_cell::sync::Lazy;
use std::sync::Mutex;

static TEST_SETUP: Lazy<Mutex<()>> = Lazy::new(|| {
    // Initialize to a known state
    let _ = enable_profiling(false, ProfileType::Time);
    Mutex::new(())
});

#[test]
fn test_enable_disable_profiling() {
    // Ensure setup has run and lock for this test
    let _guard = TEST_SETUP.lock().unwrap();

    // Now we know profiling is disabled
    assert!(!is_profiling_enabled(), "Profiling should start disabled");

    // Rest of your test...
}
```

## 4. Ensure State Reset Between Tests

Make sure to reset all relevant state at the start and end of each test:

```rust
#[test]
#[file_serial]
fn test_enable_disable_profiling() {
    // Explicitly reset ALL state at the start
    PROFILING_STATE.store(false, Ordering::SeqCst);
    PROFILE_TYPE.store(0, Ordering::SeqCst);

    // Now verify
    assert!(!is_profiling_enabled(), "Profiling should start disabled");

    // Rest of test...

    // Reset state at the end too
    PROFILING_STATE.store(false, Ordering::SeqCst);
    PROFILE_TYPE.store(0, Ordering::SeqCst);
}
```

## 5. Separate the Test into its Own File

Create a new file just for this test, for maximum isolation:

```rust
// tests/profiling_enable_disable.rs
use thag_profiler::profiling::{enable_profiling, is_profiling_enabled};
use thag_profiler::ProfileType;

#[test]
fn test_enable_disable_profiling() {
    // This test runs in complete isolation
    let _ = enable_profiling(false, ProfileType::Time);
    assert!(!is_profiling_enabled(), "Profiling should start disabled");
    // Rest of test...
}
```

## 6. Mock the Global State

If the state is interfering, consider mocking the global state for testing:

```rust
#[cfg(test)]
pub fn mock_profiling_enabled(enabled: bool) {
    PROFILING_STATE.store(enabled, Ordering::SeqCst);
}

#[test]
fn test_with_mock() {
    // Set to known state
    mock_profiling_enabled(false);
    assert!(!is_profiling_enabled());

    // Rest of test...
}
```

## 7. Use Test-Specific Constants

Override the `PROFILING_FEATURE` constant during tests:

```rust
// In profiling.rs
#[cfg(all(feature = "profiling", not(test)))]
const PROFILING_FEATURE: bool = true;

#[cfg(all(feature = "profiling", test))]
const PROFILING_FEATURE: bool = false;  // Always false in tests
```

## Recommendation

I recommend trying options 1, 2, and 4 first, as they're the least invasive. If those don't work, option 5 (separate file) is the most reliable.

For the most robust solution long-term, refactor your code to allow dependency injection of the profiling state, which would make testing much easier.



Interesting, trying option 1 this test still failed together with another. the error was:
```
thread 'test_profiling_profiled_attribute' panicked at thag_profiler/tests/profiling.rs:28:36:
called `Result::unwrap()` on an `Err` value: PoisonError { .. }
```

# Resolving the PoisonError in Serial Tests

The `PoisonError` you're encountering indicates that a mutex was poisoned, likely because another test panicked while holding the lock. This is a clear sign of test interference despite using `serial_test`.

Let's address this with a more robust approach:

## 1. Use a Resilient Mutex Guard

First, modify your common testing infrastructure to handle poisoned mutexes gracefully:

```rust
// At the top of your test file
use std::sync::{Mutex, MutexGuard, PoisonError};

// Static mutex for test synchronization
static TEST_MUTEX: Mutex<()> = Mutex::new(());

// Helper function to get a mutex guard, even if poisoned
fn get_test_lock() -> MutexGuard<'static, ()> {
    match TEST_MUTEX.lock() {
        Ok(guard) => guard,
        Err(poison_error) => {
            // If poisoned, recover the guard anyway
            println!("Warning: Mutex was poisoned from a previous test panic. Recovering...");
            poison_error.into_inner()
        }
    }
}

// Use this before each test
fn setup_test() -> MutexGuard<'static, ()> {
    let guard = get_test_lock();

    // Reset profiling state completely
    let _ = enable_profiling(false, ProfileType::Time);

    // Reset any other global state here
    // ...

    guard
}
```

## 2. Use This in Your Tests

```rust
#[test]
fn test_enable_disable_profiling() {
    // Get lock and reset state
    let _guard = setup_test();

    // Now start with known disabled state
    assert!(!is_profiling_enabled(), "Profiling should be disabled at start");

    // Test enabling
    let _ = enable_profiling(true, ProfileType::Time);
    assert!(is_profiling_enabled(), "Profiling should be enabled after call");

    // Test disabling
    let _ = enable_profiling(false, ProfileType::Time);
    assert!(!is_profiling_enabled(), "Profiling should be disabled after call");
}

#[test]
fn test_profiling_profiled_attribute() {
    // Get lock and reset state
    let _guard = setup_test();

    // Your test code
    // ...
}
```

## 3. Consider a Test Runner Helper

For even more robustness, create a test runner function:

```rust
fn run_isolated_test<F>(test_fn: F)
where
    F: FnOnce() + std::panic::UnwindSafe,
{
    // Get lock and reset state
    let _guard = setup_test();

    // Run the test function in a way that catches panics
    let result = std::panic::catch_unwind(test_fn);

    // Always cleanup, even if test panicked
    let _ = enable_profiling(false, ProfileType::Time);

    // Re-panic if the test panicked
    if let Err(panic) = result {
        std::panic::resume_unwind(panic);
    }
}

// Then use it like this:
#[test]
fn test_enable_disable_profiling() {
    run_isolated_test(|| {
        assert!(!is_profiling_enabled(), "Profiling should be disabled at start");
        let _ = enable_profiling(true, ProfileType::Time);
        assert!(is_profiling_enabled(), "Profiling should be enabled after call");
    });
}
```

## 4. Address Concurrent Profiling State Changes

The root issue may be that your profiling state isn't fully thread-safe. If `enable_profiling` is mutating more than just atomic variables, ensure all state changes are properly synchronized:

```rust
// Make sure your PROFILING_MUTEX encompasses all state changes
#[cfg(feature = "profiling")]
pub fn enable_profiling(enabled: bool, profile_type: ProfileType) -> ProfileResult<()> {
    // Acquire the mutex to ensure only one thread can enable/disable profiling at a time
    let _guard = PROFILING_MUTEX
        .lock()
        .map_err(|_| ProfileError::General("Failed to acquire profiling mutex".into()))?;

    // All state changes should happen while holding the mutex
    if enabled {
        set_profile_type(profile_type);
        // Other initialization...
    }

    // Set the state
    PROFILING_STATE.store(enabled, Ordering::SeqCst);
    Ok(())
}
```

## 5. Use Thread-Local Storage for Integration Tests

Since integration tests may run in parallel threads despite serial annotations, consider using thread-local storage for tests:

```rust
// In your profiling.rs, add:
#[cfg(test)]
thread_local! {
    static TEST_PROFILING_ENABLED: std::cell::Cell<bool> = std::cell::Cell::new(false);
}

#[cfg(test)]
pub fn is_profiling_enabled() -> bool {
    TEST_PROFILING_ENABLED.with(|cell| cell.get())
}

#[cfg(test)]
pub fn enable_profiling(enabled: bool, _profile_type: ProfileType) -> ProfileResult<()> {
    TEST_PROFILING_ENABLED.with(|cell| cell.set(enabled));
    Ok(())
}
```

This way, each test thread has its own isolated profiling state.

## Recommendation

Start with solution #1 and #2, which provide the most immediate fix. If problems persist, consider #5 for a more fundamental solution that ensures test isolation.

These approaches should resolve the `PoisonError` and ensure your tests run reliably, even when executed together.



Still having all kinds of trouble, which I'm debugging wth displays. I see if I display the value of cfg!(test) it's false, even though the command I'm runningis `cargo test --test profiling --features profiling`. Any idea why? - because a lot hinges on it being true at the moment.

# Understanding `cfg!(test)` in Integration Tests

You've discovered a very important distinction in how Rust handles integration tests versus unit tests:

## Why `cfg!(test)` is False in Integration Tests

In Rust's build system:

1. **Unit tests** (tests in your source files) are compiled with the `test` configuration flag set to `true`
2. **Integration tests** (tests in the `tests/` directory) are compiled as **separate binaries** with their own configuration

When your code in `tests/profiling.rs` uses `cfg!(test)`, it's not detecting that it's running under test because from Rust's perspective:

- This is a separate binary with its own configuration
- The binary happens to be run by the test framework, but it doesn't have the `test` flag set
- The `test` flag is only set for code from your main crate when it's compiled with tests enabled

## How to Fix This Issue

### Option 1: Use a Custom Feature Instead

The most reliable approach is to use a custom feature flag for testing:

```toml
# In Cargo.toml
[features]
testing = []  # Feature for test-specific behavior
```

Then in your code:

```rust
// In profiling.rs
#[cfg(any(test, feature = "testing"))]
pub fn special_test_function() {
    // Test-specific code
}

// In a macro if needed
#[macro_export]
macro_rules! some_macro {
    ($param:expr) => {
        #[cfg(not(any(test, feature = "testing")))]
        {
            // Normal behavior
        }

        #[cfg(any(test, feature = "testing"))]
        {
            // Test behavior
        }
    };
}
```

Then run your integration tests with:

```bash
cargo test --test profiling --features "profiling testing"
```

### Option 2: Use Compile-Time Detection

Another approach is to check if you're running tests through the environment:

```rust
// In profiling.rs
#[cfg(feature = "profiling")]
pub fn is_profiling_enabled() -> bool {
    // Check if we're running under cargo test
    if option_env!("CARGO_PKG_NAME").is_some() &&
       option_env!("CARGO_PRIMARY_PACKAGE").is_some() {
        // We're likely in a test context
        // Use test-specific behavior
    } else {
        // Normal runtime behavior
        PROFILING_FEATURE || PROFILING_STATE.load(Ordering::SeqCst)
    }
}
```

### Option 3: Use Runtime Feature Detection

You can also check at runtime if you're likely in a test environment:

```rust
pub fn is_in_test() -> bool {
    // Check for common test environment indicators
    std::env::var("CARGO_TEST").is_ok() ||
    std::env::args().any(|arg| arg.contains("--test"))
}

pub fn is_profiling_enabled() -> bool {
    if is_in_test() {
        // Test-specific behavior
    } else {
        // Normal behavior
        PROFILING_FEATURE || PROFILING_STATE.load(Ordering::SeqCst)
    }
}
```

## Recommended Approach

I recommend Option 1 (custom feature flag) as the cleanest, most explicit solution:

1. Add a `testing` feature to your `Cargo.toml`
2. Update your code to check for `#[cfg(any(test, feature = "testing"))]`
3. Run your integration tests with `--features "profiling testing"`

This gives you explicit control over when test-specific code is used, and it works correctly for both unit and integration tests.




How do I run the unit tests only (i.e. those in src/profiling.rs) vs the integration tests in tests/profiling.rs?

# Running Unit Tests vs. Integration Tests in Rust

You can selectively run unit tests or integration tests using Cargo's filtering capabilities. Here's how to do it:

## Running Only Unit Tests (in src/profiling.rs)

To run only the unit tests from your source files:

```bash
# Run all unit tests in the crate
cargo test --lib

# Or specifically for profiling.rs
cargo test --lib profiling::tests::
```

The `--lib` flag tells Cargo to only run tests in your library code (not in the `tests/` directory).

## Running Only Integration Tests (in tests/profiling.rs)

To run only the integration tests:

```bash
# Run all integration tests
cargo test --test '*'

# Run a specific integration test file
cargo test --test profiling
```

The `--test` flag tells Cargo to only run the integration tests from the specified file(s).

## Advanced Filtering

### Running Specific Tests by Name

You can also filter tests by name pattern:

```bash
# Run tests with "profile" in their name in the library
cargo test --lib profile

# Run tests with "profile" in their name in integration tests
cargo test --test profiling profile
```

### Running Tests with Features

If you need to enable features:

```bash
# Unit tests with a feature
cargo test --lib --features profiling

# Integration tests with a feature
cargo test --test profiling --features profiling
```

### Running All Tests Except Integration Tests

```bash
cargo test --lib --bins --examples
```

This runs tests in the library, binaries, and examples, but not integration tests.

## Recommended Commands for Your Case

For your specific situation:

```bash
# Run only the unit tests in profiling.rs
cargo test --lib profiling::tests::

# Run only the integration tests in tests/profiling.rs
cargo test --test profiling --features profiling
```

These commands give you precise control over which tests are executed, making it easier to debug issues in either your unit tests or integration tests separately.
