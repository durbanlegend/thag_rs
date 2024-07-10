#[cfg(test)]
mod tests {
    use rs_script::code_utils::find_modules_source;
    use rs_script::code_utils::find_use_renames_source;
    use rs_script::code_utils::infer_deps_from_ast;
    use rs_script::code_utils::infer_deps_from_source;
    use rs_script::code_utils::is_last_stmt_unit_type;
    use rs_script::code_utils::is_path_unit_type;
    use rs_script::code_utils::is_stmt_unit_type;
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

    use std::collections::HashMap;
    use syn::{parse_quote, Expr, ReturnType, Stmt};

    fn setup_function_map() -> HashMap<String, ReturnType> {
        let mut function_map = HashMap::new();
        function_map.insert("unit_fn".to_string(), ReturnType::Default);
        function_map.insert(
            "non_unit_fn".to_string(),
            ReturnType::Type(Default::default(), Box::new(syn::parse_quote!(i32))),
        );
        function_map
    }

    #[test]
    fn test_for_loop_expr() {
        let expr: Expr = parse_quote! {
            for i in 0..10 { println!("{}", i); }
        };
        let function_map = setup_function_map();
        assert!(is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_while_expr() {
        let expr: Expr = parse_quote! {
            while true { break; }
        };
        let function_map = setup_function_map();
        assert!(is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_if_expr() {
        let expr: Expr = parse_quote! {
            if true { 1 } else { 0 }
        };
        let function_map = setup_function_map();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_block_expr() {
        let expr: Expr = parse_quote! {
            { let x = 1; x + 1 }
        };
        let function_map = setup_function_map();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_match_expr() {
        let expr: Expr = parse_quote! {
            match x {
                1 => 2,
                _ => 3,
            }
        };
        let function_map = setup_function_map();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_call_expr() {
        let expr: Expr = parse_quote! {
            unit_fn()
        };
        let function_map = setup_function_map();
        assert!(is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_array_expr() {
        let expr: Expr = parse_quote! {
            [0, 1, 2]
        };
        let function_map = setup_function_map();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_return_unit_stmt() {
        let stmt: Stmt = parse_quote! {
            return;
        };
        let function_map = setup_function_map();
        assert!(is_stmt_unit_type(&stmt, &function_map));
    }

    #[test]
    fn test_return_non_unit_stmt() {
        let stmt: Stmt = parse_quote! {
            return 5;
        };
        let function_map = setup_function_map();
        assert!(!is_stmt_unit_type(&stmt, &function_map));
    }

    #[test]
    fn test_local_stmt() {
        let stmt: Stmt = parse_quote! {
            let x = 5;
        };
        let function_map = setup_function_map();
        assert!(is_stmt_unit_type(&stmt, &function_map));
    }

    #[test]
    fn test_item_const_stmt() {
        let stmt: Stmt = parse_quote! {
            const X: i32 = 5;
        };
        let function_map = setup_function_map();
        assert!(!is_stmt_unit_type(&stmt, &function_map));
    }

    #[test]
    fn test_expr_stmt_with_semicolon() {
        let stmt: Stmt = parse_quote! {
            42;
        };
        let function_map = setup_function_map();
        assert!(is_stmt_unit_type(&stmt, &function_map));
    }

    #[test]
    fn test_expr_without_semicolon() {
        let stmt: Expr = parse_quote! {
            42
        };
        let function_map = setup_function_map();
        assert!(!is_last_stmt_unit_type(&stmt, &function_map));
    }

    #[test]
    fn test_path_unit_type() {
        let expr: Expr = parse_quote! {
            unit_fn()
        };
        let function_map = setup_function_map();
        if let Expr::Call(call) = expr {
            if let Expr::Path(path) = *call.func {
                assert_eq!(is_path_unit_type(&path, &function_map), Some(true));
            } else {
                panic!("Expected Expr::Path");
            }
        } else {
            panic!("Expected Expr::Call");
        }
    }

    #[test]
    fn test_path_non_unit_type() {
        let expr: Expr = parse_quote! {
            non_unit_fn()
        };
        let function_map = setup_function_map();
        if let Expr::Call(call) = expr {
            if let Expr::Path(path) = *call.func {
                assert_eq!(is_path_unit_type(&path, &function_map), Some(false));
            } else {
                panic!("Expected Expr::Path");
            }
        } else {
            panic!("Expected Expr::Call");
        }
    }
}
