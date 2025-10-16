let maybe_test_env = std::env::var("TEST_ENV");

println!(
    r#"std::env::var("TEST_ENV")={maybe_test_env:#?}"#,
);

assert!(maybe_test_env.is_ok());
