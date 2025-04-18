use crate::styling::{ColorSupport, TermTheme};
use crate::{debug_log, profile, ThagResult};
use crossterm::terminal;
use termbg::{theme, Theme};

#[cfg(target_os = "windows")]
use std::env;

#[cfg(not(target_os = "windows"))]
use supports_color::Stream;

/// A struct of the color support details, borrowed from crate `supports-color` since we
/// can't import it because the `level` field is indispensable but private.
/// This type is returned from `supports_color::on`. See documentation for its fields for
/// more details.
#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColorLevel {
    level: usize,
    /// Basic ANSI colors are supported.
    pub has_basic: bool,
    /// 256-bit colors are supported.
    pub has_256: bool,
    /// 16 million (RGB) colors are supported.
    pub has_16m: bool,
}

#[cfg(target_os = "windows")]
pub fn get_color_level() -> Option<ColorSupport> {
    let get_color_level = profile!("get_color_level", time);
    let color_level = translate_level(supports_color());
    match color_level {
        Some(color_level) => {
            if color_level.has_16m || color_level.has_256 {
                Some(ColorSupport::Xterm256)
            } else {
                Some(ColorSupport::Ansi16)
            }
        }
        None => None,
    }
}

#[cfg(not(target_os = "windows"))]
#[must_use]
pub fn get_color_level() -> Option<ColorSupport> {
    use crate::styling::ColorSupport;

    let get_color_level = profile!("get_color_level", time);
    #[cfg(debug_assertions)]
    debug_log!("About to call supports_color");
    let color_level = supports_color::on(Stream::Stdout);
    color_level.map(|color_level| {
        if color_level.has_16m || color_level.has_256 {
            ColorSupport::Xterm256
        } else {
            ColorSupport::Ansi16
        }
    })
}

#[cfg(target_os = "windows")]
fn env_force_color() -> usize {
    let env_force_color = profile!("env_force_color", time);
    if let Ok(force) = env::var("FORCE_COLOR") {
        match force.as_ref() {
            "true" | "" => 1,
            "false" => 0,
            f => std::cmp::min(f.parse().unwrap_or(1), 3),
        }
    } else if let Ok(cli_clr_force) = env::var("CLICOLOR_FORCE") {
        usize::from(cli_clr_force != "0")
    } else {
        0
    }
}

#[cfg(target_os = "windows")]
fn env_no_color() -> bool {
    let env_no_color = profile!("env_no_color", time);
    match as_str(&env::var("NO_COLOR")) {
        Ok("0") | Err(_) => false,
        Ok(_) => true,
    }
}

// same as Option::as_deref
#[cfg(target_os = "windows")]
fn as_str<E>(option: &Result<String, E>) -> Result<&str, &E> {
    match option {
        Ok(inner) => Ok(inner),
        Err(e) => Err(e),
    }
}

#[cfg(target_os = "windows")]
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

#[cfg(target_os = "windows")]
fn supports_color() -> usize {
    let supports_color = profile!("supports_color", time);
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
    } else {
        usize::from(
            env::var("COLORTERM").is_ok()
                || env::var("TERM").map(|term| check_ansi_color(&term)) == Ok(true)
                || env::consts::OS == "windows"
                || env::var("CLICOLOR").map_or(false, |v| v != "0"),
        )
    }
}

#[cfg(target_os = "windows")]
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

#[cfg(target_os = "windows")]
fn check_colorterm_16m(colorterm: &str) -> bool {
    colorterm == "truecolor" || colorterm == "24bit"
}

#[cfg(target_os = "windows")]
fn check_term_16m(term: &str) -> bool {
    term.ends_with("direct") || term.ends_with("truecolor")
}

#[cfg(target_os = "windows")]
fn check_256_color(term: &str) -> bool {
    term.ends_with("256") || term.ends_with("256color")
}

/// Calls `termbg` to resolve whether current terminal backfround is light or dark.
///
/// # Errors
///
/// This function will bubble up any errors returned by `termbg`.
pub fn resolve_term_theme() -> ThagResult<TermTheme> {
    let resolve_term_theme = profile!("resolve_term_theme", time);
    let raw_before = terminal::is_raw_mode_enabled()?;
    #[cfg(debug_assertions)]
    debug_log!("About to call termbg");
    let timeout = std::time::Duration::from_millis(1000);

    // #[cfg(debug_assertions)]
    // debug_log!("Check terminal background color");
    let theme = theme(timeout);
    debug_log!("Found theme {theme:?}");
    maybe_restore_raw_status(raw_before)?;

    match theme {
        Ok(Theme::Light) => Ok(TermTheme::Light),
        Ok(Theme::Dark) | Err(_) => Ok(TermTheme::Dark),
    }
}

fn maybe_restore_raw_status(raw_before: bool) -> ThagResult<()> {
    let maybe_restore_raw_status = profile!("maybe_restore_raw_status", time);
    let raw_after = terminal::is_raw_mode_enabled()?;
    if raw_before == raw_after {
        debug_log!("No need to restore raw status");
    } else {
        debug_log!("Restored raw status");
        restore_raw_status(raw_before)?;
    }
    Ok(())
}

/// Restore the raw or cooked terminal status as saved in the boolean argument.
///
/// # Errors
///
/// This function will bubble up any errors returned by `crossterm`.
pub fn restore_raw_status(raw_before: bool) -> ThagResult<()> {
    let restore_raw_status = profile!("restore_raw_status", time);
    if raw_before {
        terminal::enable_raw_mode()?;
    } else {
        terminal::disable_raw_mode()?;
    }
    Ok(())
}
