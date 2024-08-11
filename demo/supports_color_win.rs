/*[toml]
[dependencies]
supports-color= "3.0.0"
*/

/// Windows-friendly logic extracted from crate `supports-color`.
///
//# Purpose: Proof of concept for Windows environment
use std::env;
use std::io::{self, Read};

/**
Color level support details.
This type is returned from [on]. See documentation for its fields for more details.
*/
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ColorLevel {
    level: usize,
    /// Basic ANSI colors are supported.
    pub has_basic: bool,
    /// 256-bit colors are supported.
    pub has_256: bool,
    /// 16 million (RGB) colors are supported.
    pub has_16m: bool,
}

fn env_force_color() -> usize {
    if let Ok(force) = env::var("FORCE_COLOR") {
        match force.as_ref() {
            "true" | "" => 1,
            "false" => 0,
            f => std::cmp::min(f.parse().unwrap_or(1), 3),
        }
    } else if let Ok(cli_clr_force) = env::var("CLICOLOR_FORCE") {
        if cli_clr_force != "0" {
            1
        } else {
            0
        }
    } else {
        0
    }
}

fn env_no_color() -> bool {
    match as_str(&env::var("NO_COLOR")) {
        Ok("0") | Err(_) => false,
        Ok(_) => true,
    }
}

// same as Option::as_deref
fn as_str<E>(option: &Result<String, E>) -> Result<&str, &E> {
    match option {
        Ok(inner) => Ok(inner),
        Err(e) => Err(e),
    }
}

fn translate_level(level: usize) -> Option<ColorLevel> {
    if level == 0 {
        None
    } else {
        Some(ColorLevel {
            level,
            has_basic: true,
            has_256: level >= 2,
            has_16m: level >= 3,
        })
    }
}

fn supports_color() -> usize {
    let force_color = env_force_color();
    if force_color > 0 {
        force_color
    } else if env_no_color()
        || as_str(&env::var("TERM")) == Ok("dumb")
        || env::var("IGNORE_IS_TERMINAL").map_or(false, |v| v != "0")
    {
        0
    } else if env::var("COLORTERM").map(|colorterm| check_colorterm_16m(&colorterm)) == Ok(true)
        || env::var("TERM").map(|term| check_term_16m(&term)) == Ok(true)
        || as_str(&env::var("TERM_PROGRAM")) == Ok("iTerm.app")
    {
        3
    } else if as_str(&env::var("TERM_PROGRAM")) == Ok("Apple_Terminal")
        || env::var("TERM").map(|term| check_256_color(&term)) == Ok(true)
    {
        2
    } else if env::var("COLORTERM").is_ok()
        || env::var("TERM").map(|term| check_ansi_color(&term)) == Ok(true)
        || env::consts::OS == "windows"
        || env::var("CLICOLOR").map_or(false, |v| v != "0")
    {
        1
    } else {
        0
    }
}

fn check_ansi_color(term: &str) -> bool {
    term.starts_with("screen")
        || term.starts_with("xterm")
        || term.starts_with("vt100")
        || term.starts_with("vt220")
        || term.starts_with("rxvt")
        || term.contains("color")
        || term.contains("ansi")
        || term.contains("cygwin")
        || term.contains("linux")
}

fn check_colorterm_16m(colorterm: &str) -> bool {
    colorterm == "truecolor" || colorterm == "24bit"
}

fn check_term_16m(term: &str) -> bool {
    term.ends_with("direct") || term.ends_with("truecolor")
}

fn check_256_color(term: &str) -> bool {
    term.ends_with("256") || term.ends_with("256color")
}

println!("Color support: {:#?}", translate_level(supports_color()));

println!("Run with -qq in Windows Terminal to suppress colored lines, type in something and see if first character gets swallowed");
let mut buffer = String::new();
io::stdin().lock().read_to_string(&mut buffer).unwrap();
println!("buffer={buffer:?}");
