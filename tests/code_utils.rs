#[cfg(test)]
mod tests {
    use rs_script::code_utils::find_modules_source;
    use rs_script::code_utils::find_use_renames_source;
    use rs_script::code_utils::infer_deps_from_ast;
    use rs_script::code_utils::infer_deps_from_source;
    use rs_script::code_utils::path_to_str;
    use rs_script::code_utils::read_file_contents;
    use rs_script::code_utils::wrap_snippet;
    use rs_script::extract_manifest;

    use rs_script::Ast;
    use std::io::Write;
    use std::path::Path;
    use std::time::Instant;
    use tempfile::NamedTempFile;

    // Helper function to create a temporary file with given content
    fn create_temp_file(content: &str) -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", content).unwrap();
        temp_file
    }

    #[test]
    fn test_read_file_contents() {
        let temp_file = create_temp_file("Test content");
        let path = temp_file.path();

        let contents = read_file_contents(path).unwrap();
        assert_eq!(contents, "Test content");
    }

    #[test]
    fn test_infer_deps_from_ast() {
        // Example AST representing use and extern crate statements
        let ast = syn::parse_file(
            r#"
            extern crate foo;
            use bar::baz;
            use std::fmt;
            "#,
        )
        .unwrap();
        let ast = Ast::File(ast);

        let deps = infer_deps_from_ast(&ast);
        assert_eq!(deps, vec!["bar", "foo"]);
    }

    #[test]
    fn test_infer_deps_from_source() {
        let source_code = r#"
            extern crate foo;
            use bar::baz;
            use std::fmt;
            "#;

        let deps = infer_deps_from_source(source_code);
        assert_eq!(deps, vec!["bar", "foo"]);
    }

    #[test]
    fn test_extract_manifest() {
        let source_code = r#"
            /*[toml]
            [dependencies]
            foo = "0.1"
            bar = "0.2"
            */
            "#;
        let start_time = Instant::now();
        let manifest = extract_manifest(source_code, start_time).unwrap();

        let dependencies = manifest.dependencies;
        assert!(dependencies.contains_key("foo"));
        assert!(dependencies.contains_key("bar"));
    }

    #[test]
    fn test_path_to_str() {
        let path = Path::new("/some/test/path");
        let path_str = path_to_str(path).unwrap();
        assert_eq!(path_str, "/some/test/path");
    }

    #[test]
    fn test_wrap_snippet() {
        let source_code = r#"
            use std::io;
            fn example() {
                println!("Example function");
            }
            "#;

        let wrapped = wrap_snippet(source_code);
        assert!(wrapped.contains("fn main() -> Result<(), Box<dyn Error>>"));
    }

    #[test]
    fn test_find_use_renames_source() {
        let source_code = r#"
            use foo as bar;
            use std::fmt;
            "#;

        let use_renames = find_use_renames_source(source_code);
        assert_eq!(use_renames, vec!["bar"]);
    }

    #[test]
    fn test_find_modules_source() {
        let source_code = r#"
            mod foo;
            mod bar;
            "#;

        let modules = find_modules_source(source_code);
        assert_eq!(modules, vec!["foo", "bar"]);
    }
}
