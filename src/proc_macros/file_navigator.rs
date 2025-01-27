use proc_macro::TokenStream;
use quote::quote;

pub fn file_navigator_impl(_input: TokenStream) -> TokenStream {
    let output = quote! {
        struct FileNavigator {
            current_dir: std::path::PathBuf,
            history: Vec<std::path::PathBuf>,
        }

        impl FileNavigator {
            fn new() -> Self {
                Self {
                    current_dir: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
                    history: Vec::new(),
                }
            }

            fn list_items(&self, include: &str) -> Vec<String> {
                let mut items = vec!["*SELECT CURRENT DIRECTORY*".to_string(), "..".to_string()];

                // Add directories
                let mut dirs: Vec<_> = std::fs::read_dir(&self.current_dir)
                    .into_iter()
                    .flatten()
                    .flatten()
                    .filter(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
                    .filter(|entry| !entry.file_name().to_string_lossy().starts_with('.'))
                    .map(|entry| entry.file_name().to_string_lossy().into_owned())
                    .collect();
                dirs.sort();
                items.extend(dirs.into_iter().map(|d| format!("📁 {d}")));

                // Add .<include> files
                let mut files: Vec<_> = std::fs::read_dir(&self.current_dir)
                    .into_iter()
                    .flatten()
                    .flatten()
                    .filter(|entry| {
                        entry.file_type().map(|ft| ft.is_file()).unwrap_or(false)
                            && entry.path().extension().is_some_and(|ext| ext == include)
                    })
                    .map(|entry| entry.file_name().to_string_lossy().into_owned())
                    .collect();
                files.sort();
                items.extend(files.into_iter().map(|f| format!("📄 {f}")));

                items
            }

            fn navigate(&mut self, selection: &str) -> Option<std::path::PathBuf> {
                if selection == ".." {
                    if let Some(parent) = self.current_dir.parent() {
                        self.history.push(self.current_dir.clone());
                        self.current_dir = parent.to_path_buf();
                    }
                    None
                } else {
                    let clean_name = selection.trim_start_matches(['📁', '📄', ' ']);
                    let new_path = self.current_dir.join(clean_name);

                    if new_path.is_dir() {
                        self.history.push(self.current_dir.clone());
                        self.current_dir = new_path;
                        None
                    } else {
                        Some(new_path)
                    }
                }
            }

            fn current_path(&self) -> &std::path::PathBuf {
                &self.current_dir
            }
        }

        fn save_to_file(content: String, default_name: String, ext: &str) -> std::io::Result<std::path::PathBuf> {
            use inquire::{Select, Text};
            let mut navigator = FileNavigator::new();

            println!("Select destination directory (use arrow keys and Enter to navigate):");

            let selected_dir = loop {
                let items = navigator.list_items(ext);
                let selection = Select::new(
                    &format!("Current directory: {}", navigator.current_path().display()),
                    items,
                )
                .with_help_message("Press Enter to navigate, Space to select current directory")
                .prompt();

                match selection {
                    Ok(sel) => {
                        if sel == "." || sel == "*SELECT CURRENT DIRECTORY*" {
                            break Some(navigator.current_path().to_path_buf());
                        } else if let Some(_path) = navigator.navigate(&sel) {
                            continue;
                        }
                    }
                    Err(inquire::error::InquireError::OperationCanceled)
                    | Err(inquire::error::InquireError::OperationInterrupted) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Selection cancelled",
                        ));
                    }
                    Err(_) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Unexpected error",
                        ))
                    }
                }
            };

            if let Some(dir) = selected_dir {
                let filename = Text::new("Enter filename:")
                    .with_default(&default_name)
                    .prompt()
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

                let full_path = dir.join(filename);
                std::fs::write(&full_path, content)?;
                Ok(full_path)
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "No directory selected",
                ))
            }
        }
    };

    output.into()
}
