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

    // Set environment variables before running tests
    fn set_up()() {
        std::env::set_var("TEST_ENV", "1");
        std::env::set_var("VISUAL", "cat");
        std::env::set_var("EDITOR", "cat");
    }

    // Helper function to create a temporary file with given content
    fn create_temp_file(content: &str) -> NamedTempFile {
        set_up()();
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", content).unwrap();
        temp_file
    }

    #[test]
    fn test_read_file_contents() {
        set_up()();
        let temp_file = create_temp_file("Test content");
        let path = temp_file.path();

        let contents = read_file_contents(path).unwrap();
        assert_eq!(contents, "Test content");
    }

    #[test]
    fn test_infer_deps_from_ast() {
        set_up()();
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
        set_up()();
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
        set_up()();
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
        set_up()();
        let path = Path::new("/some/test/path");
        let path_str = path_to_str(path).unwrap();
        assert_eq!(path_str, "/some/test/path");
    }

    #[test]
    fn test_wrap_snippet() {
        set_up()();
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
        set_up()();
        let source_code = r#"
            use foo as bar;
            use std::fmt;
            "#;

        let use_renames = find_use_renames_source(source_code);
        assert_eq!(use_renames, vec!["bar"]);
    }

    #[test]
    fn test_find_modules_source() {
        set_up()();
        let source_code = r#"
            mod foo;
            mod bar;
            "#;

        let modules = find_modules_source(source_code);
        assert_eq!(modules, vec!["foo", "bar"]);
    }

    use std::collections::HashMap;
    use syn::{parse_quote, Expr, ReturnType, Stmt};

    fn set_up()_function_map() -> HashMap<String, ReturnType> {
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
        set_up()();
        let expr: Expr = parse_quote! {
            for i in 0..10 { println!("{}", i); }
        };
        let function_map = set_up()_function_map();
        assert!(is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_while_expr() {
        set_up()();
        let expr: Expr = parse_quote! {
            while true { break; }
        };
        let function_map = set_up()_function_map();
        assert!(is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_loop_expr() {
        set_up()();
        let expr: Expr = parse_quote! {
            loop { break; }
        };
        let function_map = set_up()_function_map();
        assert!(is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_if_expr() {
        set_up()();
        let expr: Expr = parse_quote! {
            if true { 1 } else { 0 }
        };
        let function_map = set_up()_function_map();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_block_expr() {
        set_up()();
        let expr: Expr = parse_quote! {
            { let x = 1; x + 1 }
        };
        let function_map = set_up()_function_map();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_match_expr() {
        set_up()();
        let expr: Expr = parse_quote! {
            match x {
                1 => 2,
                _ => 3,
            }
        };
        let function_map = set_up()_function_map();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_call_expr() {
        set_up()();
        let expr: Expr = parse_quote! {
            unit_fn()
        };
        let function_map = set_up()_function_map();
        assert!(is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_closure_expr() {
        set_up()();
        let expr: Expr = parse_quote! {
            || { 42 }
        };
        let function_map = set_up()_function_map();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_method_call_expr() {
        set_up()();
        let expr: Expr = parse_quote! {
            foo.bar()
        };
        let function_map = set_up()_function_map();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_array_expr() {
        set_up()();
        let expr: Expr = parse_quote! {
            [0, 1, 2]
        };
        let function_map = set_up()_function_map();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_return_unit_stmt() {
        set_up()();
        let stmt: Stmt = parse_quote! {
            return;
        };
        let function_map = set_up()_function_map();
        assert!(is_stmt_unit_type(&stmt, &function_map));
    }

    #[test]
    fn test_return_non_unit_stmt() {
        set_up()();
        let stmt: Stmt = parse_quote! {
            return 5;
        };
        let function_map = set_up()_function_map();
        assert!(!is_stmt_unit_type(&stmt, &function_map));
    }

    #[test]
    fn test_yield_expr() {
        set_up()();
        let stmt: Stmt = parse_quote! {
            yield;
        };
        let function_map = set_up()_function_map();
        assert!(is_stmt_unit_type(&stmt, &function_map));
    }

    #[test]
    fn test_local_stmt() {
        set_up()();
        let stmt: Stmt = parse_quote! {
            let x = 5;
        };
        let function_map = set_up()_function_map();
        assert!(is_stmt_unit_type(&stmt, &function_map));
    }

    #[test]
    fn test_item_const_stmt() {
        set_up()();
        let stmt: Stmt = parse_quote! {
            const X: i32 = 5;
        };
        let function_map = set_up()_function_map();
        assert!(!is_stmt_unit_type(&stmt, &function_map));
    }

    #[test]
    fn test_expr_stmt_with_semicolon() {
        set_up()();
        let stmt: Stmt = parse_quote! {
            42;
        };
        let function_map = set_up()_function_map();
        assert!(is_stmt_unit_type(&stmt, &function_map));
    }

    #[test]
    fn test_expr_stmt_without_semicolon() {
        set_up()();
        let expr: Expr = parse_quote! {
            42
        };
        let function_map = set_up()_function_map();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_path_unit_type() {
        set_up()();
        let expr: Expr = parse_quote! {
            unit_fn()
        };
        let function_map = set_up()_function_map();
        if let Expr::Call(call) = expr {
            if let Expr::Path(path) = *call.func {
                assert!(is_path_unit_type(&path, &function_map).unwrap());
            } else {
                panic!("Expected Expr::Path");
            }
        } else {
            panic!("Expected Expr::Call");
        }
    }

    #[test]
    fn test_path_non_unit_type() {
        set_up()();
        let expr: Expr = parse_quote! {
            non_unit_fn()
        };
        let function_map = set_up()_function_map();
        if let Expr::Call(call) = expr {
            if let Expr::Path(path) = *call.func {
                assert!(!is_path_unit_type(&path, &function_map).unwrap());
            } else {
                panic!("Expected Expr::Path");
            }
        } else {
            panic!("Expected Expr::Call");
        }
    }

    #[test]
    fn test_macro_expr() {
        set_up()();
        let stmt: Stmt = parse_quote! {
            println!("Hello, world!");
        };
        let function_map = set_up()_function_map();
        assert!(is_stmt_unit_type(&stmt, &function_map));
    }

    #[test]
    fn test_async_expr() {
        set_up()();
        let expr: Expr = parse_quote! {
            async { 42 }
        };
        let function_map = set_up()_function_map();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_await_expr() {
        set_up()();
        let expr: Expr = parse_quote! {
            foo.await
        };
        let function_map = set_up()_function_map();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_binary_expr() {
        set_up()();
        let expr: Expr = parse_quote! {
            1 + 2
        };
        let function_map = set_up()_function_map();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_cast_expr() {
        set_up()();
        let expr: Expr = parse_quote! {
            1 as f64
        };
        let function_map = set_up()_function_map();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_index_expr() {
        set_up()();
        let expr: Expr = parse_quote! {
            arr[0]
        };
        let function_map = set_up()_function_map();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_tuple_expr() {
        set_up()();
        let expr: Expr = parse_quote! {
            (1, 2, 3)
        };
        let function_map = set_up()_function_map();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_unary_expr() {
        set_up()();
        let expr: Expr = parse_quote! {
            -42
        };
        let function_map = set_up()_function_map();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_paren_expr() {
        set_up()();
        let expr: Expr = parse_quote! {
            (42)
        };
        let function_map = set_up()_function_map();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_reference_expr() {
        set_up()();
        let expr: Expr = parse_quote! {
            &42
        };
        let function_map = set_up()_function_map();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_field_expr() {
        set_up()();
        let expr: Expr = parse_quote! {
            foo.bar
        };
        let function_map = set_up()_function_map();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_infer_expr() {
        set_up()();
        let expr: Expr = parse_quote! {
            _
        };
        let function_map = set_up()_function_map();
        assert!(is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_continue_expr() {
        set_up()();
        let expr: Expr = parse_quote! {
            continue
        };
        let function_map = set_up()_function_map();
        assert!(is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_break_expr() {
        set_up()();
        let expr: Expr = parse_quote! {
            break
        };
        let function_map = set_up()_function_map();
        assert!(is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_assign_expr() {
        set_up()();
        let expr: Expr = parse_quote! {
            x = 1
        };
        let function_map = set_up()_function_map();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_struct_expr() {
        set_up()();
        let expr: Expr = parse_quote! {
            Struct { field: 1 }
        };
        let function_map = set_up()_function_map();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_range_expr() {
        set_up()();
        let expr: Expr = parse_quote! {
            1..10
        };
        let function_map = set_up()_function_map();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_try_expr() {
        set_up()();
        let expr: Expr = parse_quote! {
            some_result?
        };
        let function_map = set_up()_function_map();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_verbatim_expr() {
        set_up()();
        let expr: Expr = parse_quote! {
            ver
        };
        let function_map = set_up()_function_map();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_assign_op_expr() {
        set_up()();
        let expr: Expr = parse_quote!(a += 1);
        let function_map = HashMap::new();
        assert!(is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_block_with_return_expr() {
        set_up()();
        let expr: Expr = parse_quote!({
            let x = 5;
            x
        });
        let function_map = HashMap::new();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_closure_non_unit_expr() {
        set_up()();
        let expr: Expr = parse_quote!(|x| x + 1);
        let function_map = HashMap::new();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_closure_unit_expr() {
        set_up()();
        let expr: Expr = parse_quote!(|| {});
        let function_map = HashMap::new();
        assert!(is_last_stmt_unit_type(&expr, &function_map));
    }

    // #[test]
    // fn test_continue_stmt() {
    //     set_up()();
    //     let stmt: Stmt = parse_quote!(continue);
    //     let function_map = HashMap::new();
    //     assert!(is_stmt_unit_type(&stmt, &function_map));
    // }

    #[test]
    fn test_if_let_expr() {
        set_up()();
        let expr: Expr = parse_quote!(if let Some(x) = y { x } else { 0 });
        let function_map = HashMap::new();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_let_expr() {
        set_up()();
        let expr: Expr = parse_quote!(let x = 5);
        let function_map = HashMap::new();
        assert!(is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_literal_expr() {
        set_up()();
        let expr: Expr = parse_quote!(42);
        let function_map = HashMap::new();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_loop_break_expr() {
        set_up()();
        let expr: Expr = parse_quote!(break);
        let function_map = HashMap::new();
        assert!(is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_macro_expr_with_semi() {
        set_up()();
        let stmt: Stmt = parse_quote!(println!("Hello"););
        let function_map = HashMap::new();
        assert!(is_stmt_unit_type(&stmt, &function_map));
    }

    #[test]
    fn test_macro_expr_without_semi() {
        set_up()();
        let expr: Expr = parse_quote!(println!("Hello"));
        let function_map = HashMap::new();
        assert!(is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_macro_with_debug() {
        set_up()();
        let expr: Expr = parse_quote!(debug!("debug message"));
        let function_map = HashMap::new();
        assert!(is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_macro_with_print() {
        set_up()();
        let expr: Expr = parse_quote!(print!("printed message"));
        let function_map = HashMap::new();
        assert!(is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_macro_with_write() {
        set_up()();
        let expr: Expr = parse_quote!(write!(std::io::stdout(), "written message"));
        let function_map = HashMap::new();
        assert!(is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_path_expr() {
        set_up()();
        let expr: Expr = parse_quote!(some_function());
        let function_map = {
            let mut map = HashMap::new();
            map.insert("some_function".to_string(), ReturnType::Default);
            map
        };
        assert!(is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_return_expr() {
        set_up()();
        let expr: Expr = parse_quote!(return);
        let function_map = HashMap::new();
        assert!(is_last_stmt_unit_type(&expr, &function_map));
    }

    #[test]
    fn test_stmt_expr() {
        set_up()();
        let expr: Expr = parse_quote!(5);
        let function_map = HashMap::new();
        assert!(!is_last_stmt_unit_type(&expr, &function_map));
    }
}
