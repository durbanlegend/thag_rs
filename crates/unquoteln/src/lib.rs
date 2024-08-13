#[macro_export]
macro_rules! unquoteln {
    ($val:expr) => {
        let formatted = format!("{:?}", $val);
        let trimmed = formatted.trim_matches('"');
        println!("{}", trimmed);
    };
}
