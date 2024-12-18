use std::error::Error;

struct Example {
    url: String,
    args: Vec<String>,
    description: Option<String>,
}

impl Example {
    fn new(url: impl Into<String>, args: Vec<String>, description: Option<String>) -> Self {
        Self {
            url: url.into(),
            args,
            description,
        }
    }

    fn to_command(&self) -> String {
        if self.args.is_empty() {
            format!("thag_url {}", self.url)
        } else {
            format!("thag_url {} -- {}", self.url, self.args.join(" "))
        }
    }

    fn generate_markdown(&self) -> String {
        let mut md = String::new();
        md.push_str("Run this example:\n\n```bash\n");
        md.push_str(&self.to_command());
        md.push_str("\n```\n");
        if let Some(desc) = &self.description {
            md.push_str("\n<details>\n<summary>About this example</summary>\n\n");
            md.push_str(desc);
            md.push_str("\n</details>\n");
        }
        md
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let examples = vec![
        Example::new(
            "https://github.com/durbanlegend/thag_rs/blob/develop/demo/fib_matrix.rs",
            vec!["10".to_string()],
            Some("Matrix-based Fibonacci calculation example".to_string()),
        ),
        Example::new(
            "https://github.com/durbanlegend/thag_rs/blob/develop/demo/hello.rs",
            Vec::new(), // Empty Vec<String>
            Some("Simple hello world example".to_string()),
        ),
    ];

    for example in examples {
        println!("{}", example.generate_markdown());
        println!(); // Add blank line between examples
    }

    Ok(())
}
