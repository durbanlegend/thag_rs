use backtrace::{Backtrace, BacktraceFrame};

fn strip_hex_suffix_slice(name: &str) -> String {
    name.rfind("::h").map_or_else(
        || name.to_string(),
        |hash_pos| {
            if name[hash_pos + 3..].chars().all(|c| c.is_ascii_hexdigit()) {
                name[..hash_pos].to_string()
            } else {
                name.to_string()
            }
        },
    )
}

fn clean_function_name_orig(clean_name: &mut str) -> String {
    let mut clean_name: &mut str = if let Some(closure_pos) = clean_name.find("::{{closure}}") {
        &mut clean_name[..closure_pos]
    } else if let Some(hash_pos) = clean_name.rfind("::h") {
        if clean_name[hash_pos + 3..]
            .chars()
            .all(|c| c.is_ascii_hexdigit())
        {
            &mut clean_name[..hash_pos]
        } else {
            clean_name
        }
    } else {
        clean_name
    };

    while clean_name.ends_with("::") {
        let len = clean_name.len();
        clean_name = &mut clean_name[..len - 2];
    }

    let mut clean_name = (*clean_name).to_string();
    while clean_name.contains("::::") {
        clean_name = clean_name.replace("::::", "::");
    }

    clean_name
}

fn clean_function_name_opt(name: &mut str) -> String {
    let trimmed = if let Some(pos) = name.find("::{{closure}}") {
        &name[..pos]
    } else if let Some(pos) = name.rfind("::h") {
        let hex = &name[pos + 3..];
        if hex.chars().all(|c| c.is_ascii_hexdigit()) {
            &name[..pos]
        } else {
            name
        }
    } else {
        name
    };

    let trimmed = trimmed.trim_end_matches("::");

    let mut result = String::with_capacity(trimmed.len());
    let mut chars = trimmed.chars().peekable();
    while let Some(c) = chars.next() {
        if c == ':' && chars.peek() == Some(&':') {
            while chars.peek() == Some(&':') {
                chars.next();
            }
            result.push_str("::");
        } else {
            result.push(c);
        }
    }

    result
}

fn main() {
    let samples = [
        "my_crate::module::::my_fn::{{closure}}",
        "another::module::some_fn::h4d2c6a7b9e8f1c3a",
        "just::some::::weird::path::::::",
        "regular::path::to::function",
        "path::with::::multiple::::colons::hdeadbeef",
        "ends::with::double_colon::",
    ];

    for s in &samples {
        let mut s = s.to_string();
        let orig = clean_function_name_orig(&mut s);
        let optimised = clean_function_name_opt(&mut s);
        println!("{orig}\n{optimised}");
        assert_eq!(optimised, orig);
    }

    let backtrace = backtrace::Backtrace::new();

    let _ = Backtrace::frames(&backtrace)
        .iter()
        .flat_map(BacktraceFrame::symbols)
        .filter_map(|symbol| symbol.name().map(|name| name.to_string()))
        .map(|s: String| strip_hex_suffix_slice(&s))
        .map(|mut name| {
            let orig = clean_function_name_orig(&mut name);
            let optimised = clean_function_name_opt(&mut name);
            println!("{name}\n{orig}\n{optimised}\n");
            assert_eq!(optimised, orig);
            orig
        })
        .collect::<Vec<_>>();

    println!();
}
