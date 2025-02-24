#[cfg(test)]
mod tests {
    use std::sync::Once;
    use thag_rs::ast::{
        find_crates, find_metadata, infer_deps_from_ast, infer_deps_from_source,
        should_filter_dependency,
    };
    use thag_rs::Ast;

    // Example AST representing use and extern crate statements
    const IMPORTS: &str = r#"
        extern crate foo;
        use bar::baz;
        mod glorp;
        use {
            crokey::{
                crossterm::{
                    event::{read, Event},
                    style::Stylize,
                    terminal,
                },
                key, KeyCombination, KeyCombinationFormat,
            },
            glorp::thagomize,
            serde::Deserialize,
            std::collections::HashMap,
            toml,
        };
        use owo_ansi::xterm as owo_xterm;
        use owo_ansi::{Blue, Cyan, Green, Red, White, Yellow};
        use owo_colors::colors::{self as owo_ansi, Magenta};
        use owo_colors::{AnsiColors, Style, XtermColors};
        use owo_xterm::Black;
        use snarf as qux;
        use std::fmt;
        use qux::corge;
        "#;

    const EXPECTED_CRATES: &[&str] = &[
        "bar",
        "crokey",
        "foo",
        "owo_colors",
        "serde",
        "snarf",
        "toml",
    ];

    // Set environment variables before running tests
    fn set_up() {
        static INIT: Once = Once::new();
        INIT.call_once(|| unsafe {
            std::env::set_var("TEST_ENV", "1");
            std::env::set_var("VISUAL", "cat");
            std::env::set_var("EDITOR", "cat");
        });
    }

    #[test]
    fn test_ast_infer_deps_from_ast() {
        set_up();
        // Example AST representing use and extern crate statements
        let ast = syn::parse_file(
            r#"
            extern crate foo;
            use bar::baz;
            use std::fmt;
            use glorp;
            "#,
        )
        .unwrap();
        let ast = Ast::File(ast);
        let crates_finder = Some(find_crates(&ast)).unwrap();
        let metadata_finder = Some(find_metadata(&ast)).unwrap();
        let deps = infer_deps_from_ast(&crates_finder, &metadata_finder);
        assert_eq!(deps, vec!["bar", "foo", "glorp"]);
    }

    #[test]
    fn test_ast_infer_deps_from_nested_ast() {
        set_up();
        // Example AST representing use and extern crate statements
        let file = syn::parse_file(IMPORTS).unwrap();
        let ast = Ast::File(file);
        let crates_finder = Some(find_crates(&ast)).unwrap();
        let metadata_finder = Some(find_metadata(&ast)).unwrap();
        let deps = infer_deps_from_ast(&crates_finder, &metadata_finder);
        assert_eq!(&deps, EXPECTED_CRATES);
    }

    #[test]
    fn test_ast_infer_deps_from_source() {
        set_up();
        let source_code = r#"
            extern crate foo;
            use bar::baz;
            use std::fmt;
            mod glorp;
            use snarf;
            "#;

        let deps = infer_deps_from_source(source_code);
        assert_eq!(deps, vec!["bar", "foo", "snarf"]);
    }

    #[test]
    fn test_ast_infer_deps_from_nested_source() {
        set_up();
        let deps = infer_deps_from_source(IMPORTS);
        assert_eq!(&deps, EXPECTED_CRATES);
    }

    #[test]
    fn test_ast_dep_filter_numeric_primitives() {
        assert!(should_filter_dependency("f32"));
        assert!(should_filter_dependency("i64"));
        assert!(should_filter_dependency("usize"));
    }

    #[test]
    fn test_ast_dep_filter_core_types() {
        assert!(should_filter_dependency("bool"));
        assert!(should_filter_dependency("str"));
    }

    #[test]
    fn test_ast_dep_filter_keywords() {
        assert!(should_filter_dependency("self"));
        assert!(should_filter_dependency("super"));
        assert!(should_filter_dependency("crate"));
    }

    #[test]
    fn test_ast_dep_filter_real_crates_not_filtered() {
        assert!(!should_filter_dependency("serde"));
        assert!(!should_filter_dependency("tokio"));
        assert!(!should_filter_dependency("rand"));
    }

    #[test]
    fn test_ast_dep_filter_capitalized_names() {
        assert!(should_filter_dependency("String"));
        assert!(should_filter_dependency("Result"));
        assert!(should_filter_dependency("Option"));
    }
}
