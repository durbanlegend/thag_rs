#[path = "../build_utils.rs"]
mod build_utils;
use build_utils::*;

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::build_utils::validate_theme_file;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    fn create_test_theme(dir: &Path, name: &str, content: &str) -> BuildResult<PathBuf> {
        let path = dir.join(format!("{}.toml", name));
        fs::write(&path, content)?;
        Ok(path)
    }

    #[test]
    fn test_cargo_build_valid_theme() -> BuildResult<()> {
        let temp_dir = TempDir::new().unwrap();

        let valid_theme = r#"
name = "Test Theme"
description = "A test theme"
term_bg_luma = "dark"
min_color_support = "basic"

[palette]
heading1 = { basic = ["\\x1b[31m", "1"], style = ["bold"] }
heading2 = { basic = ["\\x1b[32m", "2"], style = ["bold"] }
heading3 = { basic = ["\\x1b[33m", "3"], style = ["bold"] }
error = { basic = ["\\x1b[31m", "1"] }
warning = { basic = ["\\x1b[33m", "3"] }
success = { basic = ["\\x1b[32m", "2"] }
info = { basic = ["\\x1b[36m", "6"] }
emphasis = { basic = ["\\x1b[35m", "5"], style = ["bold"] }
code = { basic = ["\\x1b[34m", "4"] }
normal = { basic = ["\\x1b[0m", "0"] }
subtle = { basic = ["\\x1b[37m", "7"] }
hint = { basic = ["\\x1b[36m", "6"], style = ["italic"] }
debug = { basic = ["\\x1b[36m", "6"] }
link = { basic = ["\\x1b[31m", "1"] }
quote = { basic = ["\\x1b[37m", "7"] }
commentary = { basic = ["\\x1b[90m", "8"] }
"#;

        let theme_path = create_test_theme(temp_dir.path(), "valid", valid_theme)?;
        validate_theme_file(&theme_path)
    }

    #[test]
    fn test_cargo_build_missing_required_field() {
        let temp_dir = TempDir::new().unwrap();

        let invalid_theme = r#"
name = "Test Theme"
# missing description
term_bg_luma = "dark"
min_color_support = "basic"

[palette]
heading1 = { basic = ["\\x1b[31m", "1"], style = ["bold"] }
"#;

        let theme_path = create_test_theme(temp_dir.path(), "invalid", invalid_theme).unwrap();
        assert!(validate_theme_file(&theme_path).is_err());
    }

    #[test]
    fn test_cargo_build_missing_palette_style() {
        let temp_dir = TempDir::new().unwrap();

        let invalid_theme = r#"
name = "Test Theme"
description = "A test theme"
term_bg_luma = "dark"
min_color_support = "basic"

[palette]
heading1 = { basic = ["\\x1b[31m", "1"], style = ["bold"] }
# missing required styles
"#;

        let theme_path = create_test_theme(temp_dir.path(), "invalid", invalid_theme).unwrap();
        assert!(validate_theme_file(&theme_path).is_err());
    }

    #[test]
    fn test_cargo_build_invalid_color_support() {
        let temp_dir = TempDir::new().unwrap();

        let invalid_theme = r#"
name = "Test Theme"
description = "A test theme"
term_bg_luma = "dark"
min_color_support = "invalid_value"  # invalid value

[palette]
heading1 = { basic = ["\\x1b[31m", "1"], style = ["bold"] }
"#;

        let theme_path = create_test_theme(temp_dir.path(), "invalid", invalid_theme).unwrap();
        assert!(validate_theme_file(&theme_path).is_err());
    }

    #[test]
    fn test_cargo_build_invalid_term_bg_luma() {
        let temp_dir = TempDir::new().unwrap();

        let invalid_theme = r#"
name = "Test Theme"
description = "A test theme"
term_bg_luma = "medium"  # invalid value
min_color_support = "basic"

[palette]
heading1 = { basic = ["\\x1b[31m", "1"], style = ["bold"] }
"#;

        let theme_path = create_test_theme(temp_dir.path(), "invalid", invalid_theme).unwrap();
        assert!(validate_theme_file(&theme_path).is_err());
    }
}
