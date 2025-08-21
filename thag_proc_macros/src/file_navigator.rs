#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;

#[allow(clippy::too_many_lines)]
pub fn file_navigator_impl(_input: TokenStream) -> TokenStream {
    let output = quote! {
        use inquire::{InquireError, Select, Text};

        struct FileNavigator {
            current_dir: std::path::PathBuf,
            history: Vec<std::path::PathBuf>,
        }

        #[derive(Debug)]
        pub enum NavigationResult {
            SelectionComplete(std::path::PathBuf),
            NavigatedTo(std::path::PathBuf),
            NoSelection,
        }

        impl FileNavigator {
            fn new() -> Self {
                Self {
                    current_dir: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
                    history: Vec::new(),
                }
            }

            fn list_items(&self, include_ext: Option<&str>, hidden: bool, new_subdir_opt: bool, use_current_opt: bool) -> Vec<String> {
                let mut items = vec!["*TYPE PATH TO NAVIGATE*".to_string(), "..".to_string()];
                if new_subdir_opt {
                    items.insert(1, "*CREATE NEW SUBDIRECTORY*".to_string());
                }
                if use_current_opt {
                    items.insert(0, "*SELECT CURRENT DIRECTORY*".to_string());
                }

                // Add directories
                let mut dirs: Vec<_> = std::fs::read_dir(&self.current_dir)
                    .into_iter()
                    .flatten()
                    .flatten()
                    .filter(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
                    .filter(|entry| if hidden {true} else {!entry.file_name().to_string_lossy().starts_with('.')})
                    .map(|entry| entry.file_name().to_string_lossy().into_owned())
                    .collect();
                dirs.sort();
                items.extend(dirs.into_iter().map(|d| format!("📁 {d}")));

                // let includes: Vec<String> = include_ext.is_some_and(|incl| incl.split(",").map(String::from).collect());
                let includes = include_ext.map(|incl| incl.split(",").map(String::from).collect::<Vec<_>>());
                // Add .<include_ext> files
                let mut files: Vec<_> = std::fs::read_dir(&self.current_dir)
                    .into_iter()
                    .flatten()
                    .flatten()
                    .filter(|entry| {
                        entry.file_type().map(|ft| ft.is_file()).unwrap_or(false)
                            && entry.path().extension().is_some_and(|ext| includes.as_ref().is_some_and(|incl| incl.contains(&ext.to_string_lossy().to_string())))
                    })
                    .map(|entry| entry.file_name().to_string_lossy().into_owned())
                    .collect();
                files.sort();
                items.extend(files.into_iter().map(|f| format!("📄 {f}")));

                items
            }

            fn navigate(&mut self, selection: &str, select_dir: bool) -> NavigationResult {
                if selection == ".." {
                    if let Some(parent) = self.current_dir.parent() {
                        self.history.push(self.current_dir.clone());
                        self.current_dir = parent.to_path_buf();
                    }
                    NavigationResult::NoSelection
                } else if selection == "*SELECT CURRENT DIRECTORY*" && select_dir {
                    NavigationResult::SelectionComplete(self.current_dir.clone())
                } else if selection == "*TYPE PATH TO NAVIGATE*" {
                    NavigationResult::NoSelection
                } else {
                    let clean_name = selection.trim_start_matches(['📁', '📄', ' ']);
                    let new_path = self.current_dir.join(clean_name);

                    if new_path.is_dir() {
                        self.history.push(self.current_dir.clone());
                        self.current_dir.clone_from(&new_path);
                        if select_dir {
                            NavigationResult::NavigatedTo(new_path)
                        } else {
                            NavigationResult::NoSelection
                        }
                    } else if select_dir {
                        NavigationResult::NoSelection
                    } else {
                        NavigationResult::SelectionComplete(new_path)
                    }
                }
            }

            const fn current_path(&self) -> &std::path::PathBuf {
                &self.current_dir
            }

            fn expand_path(path: &str) -> std::path::PathBuf {
                let expanded = if path.starts_with('~') {
                    if let Some(home) = std::env::var_os("HOME") {
                        std::path::PathBuf::from(home).join(path.strip_prefix("~/").unwrap_or(""))
                    } else {
                        std::path::PathBuf::from(path)
                    }
                } else {
                    std::path::PathBuf::from(path)
                };

                // Expand environment variables
                let path_str = expanded.to_string_lossy();
                let mut result = String::new();
                let mut chars = path_str.chars().peekable();

                while let Some(ch) = chars.next() {
                    if ch == '$' {
                        if chars.peek() == Some(&'{') {
                            chars.next(); // consume '{'
                            let mut var_name = String::new();
                            while let Some(ch) = chars.next() {
                                if ch == '}' {
                                    break;
                                }
                                var_name.push(ch);
                            }
                            if let Ok(var_value) = std::env::var(&var_name) {
                                result.push_str(&var_value);
                            } else {
                                result.push_str(&format!("${{{var_name}}}"));
                            }
                        } else {
                            let mut var_name = String::new();
                            while let Some(&ch) = chars.peek() {
                                if ch.is_alphanumeric() || ch == '_' {
                                    var_name.push(chars.next().unwrap());
                                } else {
                                    break;
                                }
                            }
                            if !var_name.is_empty() {
                                if let Ok(var_value) = std::env::var(&var_name) {
                                    result.push_str(&var_value);
                                } else {
                                    result.push_str(&format!("${var_name}"));
                                }
                            } else {
                                result.push('$');
                            }
                        }
                    } else {
                        result.push(ch);
                    }
                }

                std::path::PathBuf::from(result)
            }

            fn navigate_to_path(&mut self, path: &str) -> Result<(), String> {
                let expanded_path = Self::expand_path(path);

                if expanded_path.is_dir() {
                    self.history.push(self.current_dir.clone());
                    self.current_dir = expanded_path;
                    Ok(())
                } else {
                    Err(format!("Path '{}' is not a valid directory", expanded_path.display()))
                }
            }
        }

        fn select_directory(navigator: &mut FileNavigator, hidden: bool) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
            println!("Select a directory (use arrow keys and Enter to navigate):");

            loop {
                let current_path = navigator.current_path().display();
                let items = navigator.list_items(None, hidden, true, true);
                let selection = Select::new(
                    &format!("Current directory: {current_path}", ),
                    items,
                )
                .with_help_message("Press Enter to navigate, select '*SELECT CURRENT DIRECTORY*' to choose current directory")
                .prompt()?;

                if selection == "*TYPE PATH TO NAVIGATE*" {
                    let path_input = Text::new("Enter path to navigate to (supports ~ and $VAR):")
                        .with_help_message("Examples: /tmp, ~/Documents, $HOME/projects")
                        .prompt()?;

                    match navigator.navigate_to_path(&path_input) {
                        Ok(()) => continue,
                        Err(err) => {
                            println!("\nFile navigator: {err}\n");
                            continue;
                        }
                    }
                }

                if selection == "*CREATE NEW SUBDIRECTORY*" {
                    let subdir = Text::new("Enter name of new subdirectory")
                        .with_help_message("Examples: demo, cat123")
                        .prompt()?;
                    let new_path = format!("{current_path}/{subdir}");
                    std::fs::create_dir_all(&new_path)?;

                    match navigator.navigate_to_path(&new_path) {
                        Ok(()) => continue,
                        Err(err) => {
                            println!("\nFile navigator: {err}\n");
                            continue;
                        }
                    }
                }

                match navigator.navigate(&selection, true) {
                    NavigationResult::SelectionComplete(path) => if inquire::Confirm::new(&format!("Use {}?", path.display()))
                                    .with_default(true)
                                    .prompt()?
                                {
                                    return Ok(path);
                                } else {
                                    continue;
                                }
                    NavigationResult::NavigatedTo(_) | NavigationResult::NoSelection => continue,
                }
            }
        }

        fn select_file(navigator: &mut FileNavigator, include_ext: Option<&str>, hidden: bool) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
            println!("Select a file (use arrow keys and Enter to navigate):");

            loop {
                let items = navigator.list_items(include_ext, hidden, false, false);
                let selection = Select::new(
                    &format!("Current directory: {}", navigator.current_path().display()),
                    items,
                )
                .with_help_message("Press Enter to navigate, select a file to load")
                .prompt()?;

                if selection == "*TYPE PATH TO NAVIGATE*" {
                    let path_input = Text::new("Enter path to navigate to (supports ~ and $VAR):")
                        .with_help_message("Examples: /tmp, ~/Documents, $HOME/projects")
                        .prompt()?;

                    match navigator.navigate_to_path(&path_input) {
                        Ok(()) => continue,
                        Err(err) => {
                            println!("\nFile navigator: {err}\n");
                            continue;
                        }
                    }
                }

                match navigator.navigate(&selection, false) {
                    NavigationResult::SelectionComplete(path) => if inquire::Confirm::new(&format!("Use {}?", path.display()))
                                    .with_default(true)
                                    .prompt()?
                                {
                                    return Ok(path);
                                } else {
                                    continue;
                                },
                    NavigationResult::NavigatedTo(_) | NavigationResult::NoSelection => continue,
                }
            }
        }

        fn save_to_file(content: String, default_name: &str, include_ext: Option<&str>, hidden: bool) -> std::io::Result<std::path::PathBuf> {
            let mut navigator = FileNavigator::new();

            println!("Select destination directory (use arrow keys and Enter to navigate):");

            let selected_dir = loop {
                let items = navigator.list_items(include_ext, hidden, true, true);
                let selection = Select::new(
                    &format!("Current directory: {}", navigator.current_path().display()),
                    items,
                )
                .with_help_message("Press Enter to navigate, Space to select current directory")
                .prompt();

                match selection {
                    Ok(sel) => {
                        if sel == "." || sel == "*SELECT CURRENT DIRECTORY*" {
                            break Some(navigator.current_path().clone());
                        } else if sel == "*TYPE PATH TO NAVIGATE*" {
                            let path_input = Text::new("Enter path to navigate to (supports ~ and $VAR):")
                                .with_help_message("Examples: /tmp, ~/Documents, $HOME/projects")
                                .prompt()
                                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

                            match navigator.navigate_to_path(&path_input) {
                                Ok(()) => continue,
                                Err(err) => {
                                    println!("\nFile navigator: {err}\n");
                                    continue;
                                }
                            }
                        } else if let NavigationResult::NavigatedTo(_path) = navigator.navigate(&sel, hidden) {
                            continue;
                        }
                    }
                    Err(inquire::error::InquireError::OperationCanceled | inquire::error::InquireError::OperationInterrupted) => {
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
                    .with_default(default_name)
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
