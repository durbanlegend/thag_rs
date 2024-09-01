#![allow(clippy::uninlined_format_args)]
use crate::builder::gen_build_run;
use crate::cmd_args::{Cli, ProcFlags};
use crate::errors::ThagError;
use crate::log;
use crate::logging::Verbosity;
use crate::shared::{debug_timings, Ast, BuildState};
use crate::{debug_log, nu_resolve_style};
use crate::{DYNAMIC_SUBDIR, REPL_SUBDIR, TEMP_SCRIPT_NAME, TMPDIR};

use cargo_toml::{Edition, Manifest};
use firestorm::profile_fn;
use lazy_static::lazy_static;
use regex::Regex;
use std::any::Any;
use std::collections::HashMap;
use std::fs;
use std::fs::{remove_dir_all, remove_file, OpenOptions};
use std::io::{self, BufRead, Write};
use std::option::Option;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Instant, SystemTime};
use syn::{
    visit::Visit,
    visit_mut::{self, VisitMut},
};
use syn::{
    AttrStyle, Expr, ExprBlock, File, Item, ItemExternCrate, ItemMod, ReturnType, Stmt, UsePath,
    UseRename,
};

// To move inner attributes out of a syn AST for a snippet.
struct RemoveInnerAttributes {
    found: bool,
}

impl VisitMut for RemoveInnerAttributes {
    fn visit_expr_block_mut(&mut self, expr_block: &mut ExprBlock) {
        profile_fn!(visit_expr_block_mut);
        // Count inner attributes
        self.found = expr_block
            .attrs
            .iter()
            .filter(|attr| attr.style == AttrStyle::Inner(syn::token::Not::default()))
            .count()
            > 0;
        if self.found {
            // Filter out inner attributes
            expr_block
                .attrs
                .retain(|attr| attr.style != AttrStyle::Inner(syn::token::Not::default()));
        };

        // Delegate to the default impl to visit nested expressions.
        visit_mut::visit_expr_block_mut(self, expr_block);
    }
}

/// Remove inner attributes (`#![...]`) from the part of the AST that will be wrapped in
/// `fn main`, as they need to be promoted to the crate level.
pub fn remove_inner_attributes(expr: &mut syn::ExprBlock) -> bool {
    profile_fn!(remove_inner_attributes);
    let remove_inner_attributes = &mut RemoveInnerAttributes { found: false };
    remove_inner_attributes.visit_expr_block_mut(expr);
    remove_inner_attributes.found
}

/// Read the contents of a file. For reading the Rust script.
/// # Errors
/// Will return `Err` if there is any file system error reading from the file path.
pub fn read_file_contents(path: &Path) -> Result<String, ThagError> {
    profile_fn!(read_file_contents);
    #[cfg(debug_assertions)]
    debug_log!("Reading from {path:?}");
    Ok(fs::read_to_string(path)?)
}

/// Infer dependencies from the abstract syntax tree to put in a Cargo.toml.
#[must_use]
pub fn infer_deps_from_ast(syntax_tree: &Ast) -> Vec<String> {
    profile_fn!(infer_deps_from_ast);
    let use_crates = find_use_crates_ast(syntax_tree);
    let extern_crates = find_extern_crates_ast(syntax_tree);
    let use_renames = find_use_renames_ast(syntax_tree);
    let modules = find_modules_ast(syntax_tree);

    let mut dependencies = Vec::new();
    let built_in_crates = ["std", "core", "alloc", "collections", "fmt", "crate"];

    for crate_name in use_crates {
        filter_deps_ast(
            &crate_name,
            &built_in_crates,
            &use_renames,
            &modules,
            &mut dependencies,
        );
    }

    for crate_name in extern_crates {
        filter_deps_ast(
            &crate_name,
            &built_in_crates,
            &use_renames,
            &modules,
            &mut dependencies,
        );
    }

    // Deduplicate the list of dependencies
    dependencies.sort();
    dependencies.dedup();

    dependencies
}

/// Filter out crates that don't need to be added as dependencies: abstract syntax tree-based version.
fn filter_deps_ast(
    crate_name: &str,
    built_in_crates: &[&str; 6],
    use_renames: &[String],
    modules: &[String],
    dependencies: &mut Vec<String>,
) {
    profile_fn!(filter_deps_ast);
    let crate_name_string = crate_name.to_string();
    // Filter out "crate" entries
    if !built_in_crates.contains(&crate_name)
        && !use_renames.contains(&crate_name_string)
        && !modules.contains(&crate_name_string)
    {
        dependencies.push(crate_name_string);
    }
}

/// Identify use ... as statements for exclusion from Cargo.toml metadata: abstract syntax tree-based version.
fn find_use_renames_ast(syntax_tree: &Ast) -> Vec<String> {
    #[derive(Default)]
    struct FindCrates {
        use_renames: Vec<String>,
    }
    impl<'a> Visit<'a> for FindCrates {
        fn visit_use_rename(&mut self, node: &'a UseRename) {
            profile_fn!(visit_use_rename);
            self.use_renames.push(node.rename.to_string());
        }
    }

    profile_fn!(find_use_renames_ast);
    let mut finder = FindCrates::default();

    match syntax_tree {
        Ast::File(ast) => finder.visit_file(ast),
        Ast::Expr(ast) => finder.visit_expr(ast),
    }

    #[cfg(debug_assertions)]
    debug_log!("use_renames from ast={:#?}", finder.use_renames);
    finder.use_renames
}

/// Identify modules for filtering use statements from Cargo.toml metadata: abstract syntax tree-based version.
fn find_modules_ast(syntax_tree: &Ast) -> Vec<String> {
    #[derive(Default)]
    struct FindMods {
        modules: Vec<String>,
    }
    impl<'a> Visit<'a> for FindMods {
        fn visit_item_mod(&mut self, node: &'a ItemMod) {
            profile_fn!(visit_item_mod);
            self.modules.push(node.ident.to_string());
        }
    }

    profile_fn!(find_modules_ast);
    let mut finder = FindMods::default();

    match syntax_tree {
        Ast::File(ast) => finder.visit_file(ast),
        Ast::Expr(ast) => finder.visit_expr(ast),
    }

    #[cfg(debug_assertions)]
    debug_log!("modules from ast={:#?}", finder.modules);
    finder.modules
}

/// Identify use crate statements for inclusion in Cargo.toml metadata: abstract syntax tree-based version.
fn find_use_crates_ast(syntax_tree: &Ast) -> Vec<String> {
    #[derive(Default)]
    struct FindCrates {
        use_crates: Vec<String>,
    }
    impl<'a> Visit<'a> for FindCrates {
        fn visit_use_path(&mut self, node: &'a UsePath) {
            profile_fn!(visit_use_path);
            let node_name = node.ident.to_string();
            // See for instance Constraint and Keyword in demo/tui_scrollview.rs.
            if let Some(c) = node_name.chars().nth(0) {
                if c.is_uppercase() {
                    #[cfg(debug_assertions)]
                    debug_log!("Assuming capitalised use name {} is not a crate", node_name);
                    return;
                }
            }
            self.use_crates.push(node_name);
        }
    }

    profile_fn!(find_use_crates_ast);
    let mut finder = FindCrates::default();

    match syntax_tree {
        Ast::File(ast) => finder.visit_file(ast),
        Ast::Expr(ast) => finder.visit_expr(ast),
    }

    #[cfg(debug_assertions)]
    debug_log!("use_crates from ast={:#?}", finder.use_crates);
    finder.use_crates
}

/// Identify extern crate stxatements for inclusion in Cargo.toml metadata: abstract syntax tree-based version.
fn find_extern_crates_ast(syntax_tree: &Ast) -> Vec<String> {
    #[derive(Default)]
    struct FindCrates {
        extern_crates: Vec<String>,
    }

    impl<'a> Visit<'a> for FindCrates {
        fn visit_item_extern_crate(&mut self, node: &'a ItemExternCrate) {
            profile_fn!(visit_item_extern_crate);
            self.extern_crates.push(node.ident.to_string());
        }
    }

    profile_fn!(find_extern_crates_ast);
    let mut finder = FindCrates::default();

    match syntax_tree {
        Ast::File(ast) => finder.visit_file(ast),
        Ast::Expr(ast) => finder.visit_expr(ast),
    }
    #[cfg(debug_assertions)]
    debug_log!("extern_crates from ast={:#?}", finder.extern_crates);
    finder.extern_crates
}

/// Infer dependencies from source code to put in a Cargo.toml.
/// Fallback version for when an abstract syntax tree cannot be parsed.
#[must_use]
pub fn infer_deps_from_source(code: &str) -> Vec<String> {
    lazy_static! {
        static ref USE_REGEX: Regex = Regex::new(r"(?m)^[\s]*use\s+([^;{]+)").unwrap();
        static ref MACRO_USE_REGEX: Regex = Regex::new(r"(?m)^[\s]*#\[macro_use\((\w+)\)").unwrap();
        static ref EXTERN_CRATE_REGEX: Regex =
            Regex::new(r"(?m)^[\s]*extern\s+crate\s+([^;{]+)").unwrap();
    }
    profile_fn!(infer_deps_from_source);

    #[cfg(debug_assertions)]
    debug_log!("In code_utils::infer_deps_from_source");
    let use_renames = find_use_renames_source(code);
    let modules = find_modules_source(code);

    let mut dependencies = Vec::new();

    let built_in_crates = [
        "std",
        "core",
        "alloc",
        "collections",
        "fmt",
        "crate",
        "super",
    ];

    for cap in USE_REGEX.captures_iter(code) {
        let crate_name = cap[1].to_string();
        #[cfg(debug_assertions)]
        debug_log!("dependency={crate_name}");
        filter_deps_source(
            &crate_name,
            &built_in_crates,
            &use_renames,
            &modules,
            &mut dependencies,
        );
    }

    // Similar checks for other regex patterns

    for cap in MACRO_USE_REGEX.captures_iter(code) {
        let crate_name = cap[1].to_string();
        filter_deps_source(
            &crate_name,
            &built_in_crates,
            &use_renames,
            &modules,
            &mut dependencies,
        );
    }

    for cap in EXTERN_CRATE_REGEX.captures_iter(code) {
        let crate_name = cap[1].to_string();
        filter_deps_source(
            &crate_name,
            &built_in_crates,
            &use_renames,
            &modules,
            &mut dependencies,
        );
    }
    // Deduplicate the list of dependencies
    dependencies.sort();
    dependencies.dedup();

    #[cfg(debug_assertions)]
    debug_log!("dependencies from source={dependencies:#?}");
    dependencies
}

/// Filter out crates that don't need to be added as dependencies: fallback version using regex on source code.
fn filter_deps_source(
    crate_name: &str,
    built_in_crates: &[&str; 7],
    use_renames: &[String],
    modules: &[String],
    dependencies: &mut Vec<String>,
) {
    profile_fn!(filter_deps_source);
    #[cfg(debug_assertions)]
    debug_log!("crate_name={crate_name}");
    let dep = if crate_name.contains(':') {
        crate_name.split_once(':').unwrap().0
    } else {
        crate_name
    };

    let dep_string = dep.to_owned();
    #[cfg(debug_assertions)]
    debug_log!("dep_string={dep_string}, built_in_crates={built_in_crates:#?}, use_renames={use_renames:#?}, modules={modules:#?}");
    if !built_in_crates.contains(&dep)
        && !use_renames.contains(&dep_string)
        && !modules.contains(&dep_string)
    {
        dependencies.push(dep_string);
    }
}

/// Identify use ... as statements for exclusion from Cargo.toml metadata.
/// Fallback version for when an abstract syntax tree cannot be parsed.
#[must_use]
pub fn find_use_renames_source(code: &str) -> Vec<String> {
    profile_fn!(find_use_renames_source);
    lazy_static! {
        static ref USE_AS_REGEX: Regex = Regex::new(r"(?m)^\s*use\s+.+as\s+(\w+)").unwrap();
    }

    #[cfg(debug_assertions)]
    debug_log!("In code_utils::find_use_renames_source");

    let mut use_renames: Vec<String> = vec![];

    for cap in USE_AS_REGEX.captures_iter(code) {
        let use_rename = cap[1].to_string();
        #[cfg(debug_assertions)]
        debug_log!("use_rename={use_rename}");
        use_renames.push(use_rename);
    }

    #[cfg(debug_assertions)]
    debug_log!("use_renames from source={use_renames:#?}");
    use_renames
}

/// Identify mod statements for exclusion from Cargo.toml metadata.
/// Fallback version for when an abstract syntax tree cannot be parsed.
#[must_use]
pub fn find_modules_source(code: &str) -> Vec<String> {
    lazy_static! {
        static ref MODULE_REGEX: Regex = Regex::new(r"(?m)^[\s]*mod\s+([^;{\s]+)").unwrap();
    }

    #[cfg(debug_assertions)]
    debug_log!("In code_utils::find_use_renames_source");

    let mut modules: Vec<String> = vec![];

    for cap in MODULE_REGEX.captures_iter(code) {
        let module = cap[1].to_string();
        #[cfg(debug_assertions)]
        debug_log!("module={module}");
        modules.push(module);
    }

    #[cfg(debug_assertions)]
    debug_log!("modules from source={modules:#?}");
    modules
}

/// Extract embedded Cargo.toml metadata from a Rust source string.
/// # Errors
/// Will return `Err` if there is any error in parsing the toml data into a manifest.
pub fn extract_manifest(
    rs_full_source: &str,
    start_parsing_rs: Instant,
) -> Result<Manifest, ThagError> {
    let maybe_rs_toml = extract_toml_block(rs_full_source);

    let mut rs_manifest = if let Some(rs_toml_str) = maybe_rs_toml {
        // #[cfg(debug_assertions)]
        // debug_log!("rs_toml_str={rs_toml_str}");
        Manifest::from_str(&rs_toml_str)?
    } else {
        Manifest::from_str("")?
    };

    if let Some(package) = rs_manifest.package.as_mut() {
        package.edition = cargo_toml::Inheritable::Set(Edition::E2021);
    }
    // #[cfg(debug_assertions)]
    // debug_log!("rs_manifest={rs_manifest:#?}");

    #[cfg(debug_assertions)]
    debug_timings(&start_parsing_rs, "extract_manifest parsed source");
    Ok(rs_manifest)
}

fn extract_toml_block(input: &str) -> Option<String> {
    let re = Regex::new(r"(?s)/\*\[toml\](.*?)\*/").unwrap();
    re.captures(input)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

/// Parse a Rust expression source string into a syntax tree.
/// Although this is primarily intended for incomplete snippets and expressions, if it finds a fully-fledged program that
/// could equally be parsed with `syn::parse_file`, it should succeed anyway by wrapping it in braces. However that is the
/// snippet path and is not encouraged as it is likely to process and wrap the code unnecessarily.
/// # Errors
/// Will return `Err` if there is any error encountered by the `syn` crate trying to parse the source string into an AST.
pub fn extract_ast_expr(rs_source: &str) -> Result<Expr, syn::Error> {
    let mut expr: Result<Expr, syn::Error> = syn::parse_str::<Expr>(rs_source);
    if expr.is_err() && !(rs_source.starts_with('{') && rs_source.ends_with('}')) {
        // Try putting the expression in braces.
        let string = format!(r"{{{rs_source}}}");
        let str = string.as_str();
        // log!(Verbosity::Normal, "str={str}");

        expr = syn::parse_str::<Expr>(str);
    }
    expr
}

/// Process a Rust expression
/// # Errors
/// Will return `Err` if there is any errors encountered opening or writing to the file.
pub fn process_expr(
    expr_ast: Expr,
    build_state: &mut BuildState,
    rs_source: &str,
    args: &mut Cli,
    proc_flags: &ProcFlags,
    start: &Instant,
) -> Result<(), ThagError> {
    let syntax_tree = Some(Ast::Expr(expr_ast));
    write_source(&build_state.source_path, rs_source)?;
    let result = gen_build_run(args, proc_flags, build_state, syntax_tree, start);
    log!(Verbosity::Normal, "{result:?}");
    Ok(())
}

/// Convert a Path to a string value, assuming the path contains only valid characters.
/// # Errors
/// Will return `Err` if there is any error caused by invalid characters in the path name.
pub fn path_to_str(path: &Path) -> Result<String, ThagError> {
    let string = path
        .to_path_buf()
        .into_os_string()
        .into_string()
        .map_err(ThagError::OsString)?;
    #[cfg(debug_assertions)]
    debug_log!("path_to_str={string}");
    Ok(string)
}

/// Reassemble an Iterator of lines from the disentangle function to a string of text.
#[inline]
pub fn reassemble<'a>(map: impl Iterator<Item = &'a str>) -> String {
    use std::fmt::Write;
    map.fold(String::new(), |mut output, b| {
        let _ = writeln!(output, "{b}");
        output
    })
}

/// Unescape \n markers to convert a string of raw text to readable lines.
#[inline]
#[must_use]
pub fn disentangle(text_wall: &str) -> String {
    reassemble(text_wall.lines())
}

#[cfg(debug_assertions)]
#[warn(dead_code)]
/// Display output captured to `std::process::Output`.
/// # Errors
/// Will return `Err` if the stdout or stderr is not found captured as expected.
pub fn display_output(output: &Output) -> Result<(), ThagError> {
    // Read the captured output from the pipe
    // let stdout = output.stdout;

    // Print the captured stdout
    log!(Verbosity::Normal, "Captured stdout:");
    for result in output.stdout.lines() {
        log!(Verbosity::Normal, "{}", result?);
    }

    // Print the captured stderr
    log!(Verbosity::Normal, "Captured stderr:");
    for result in output.stderr.lines() {
        log!(Verbosity::Normal, "{}", result?);
    }
    Ok(())
}

/// Check if executable is stale, i.e. if raw source script or individual Cargo.toml
/// has a more recent modification date and time
/// # Errors
/// Will return `Err` if either the executable or the Cargo.toml for the script is missing,
/// or if there is a logic error wrapping the path and modified time.
pub fn modified_since_compiled(
    build_state: &BuildState,
) -> Result<Option<(&PathBuf, SystemTime)>, ThagError> {
    profile_fn!(modified_since_compiled);

    let executable = &build_state.target_path;
    executable.try_exists()?;
    let Ok(metadata) = fs::metadata(executable) else {
        return Ok(None);
    };

    let baseline_modified = metadata.modified()?;

    let files = [&build_state.source_path, &build_state.cargo_toml_path];
    let mut most_recent: Option<(&PathBuf, SystemTime)> = None;
    for &file in &files {
        let Ok(metadata) = fs::metadata(file) else {
            continue;
        };

        let modified_time = metadata.modified()?;
        #[cfg(debug_assertions)]
        debug_log!("File: {file:?} modified time is {modified_time:#?}");

        if modified_time < baseline_modified {
            continue;
        }

        if most_recent.is_none()
            || modified_time
                > most_recent
                    .ok_or("Logic error unwrapping what we wrapped ourselves")?
                    .1
        {
            most_recent = Some((file, modified_time));
        }
    }
    if let Some(file) = most_recent {
        log!(
            Verbosity::Verbose,
            "The most recently modified file compared to {executable:#?} is: {file:#?}"
        );
        #[cfg(debug_assertions)]
        debug_log!("Executable modified time is{baseline_modified:#?}");
    } else {
        #[cfg(debug_assertions)]
        debug_log!("Neither file was modified more recently than {executable:#?}");
    }
    Ok(most_recent)
}

/// Count the number of `main()` methods in an abstract syntax tree.
#[must_use]
pub fn count_main_methods(syntax_tree: &Ast) -> usize {
    #[derive(Default)]
    struct FindMainFns {
        main_method_count: usize,
    }

    impl<'a> Visit<'a> for FindMainFns {
        fn visit_item_fn(&mut self, node: &'a syn::ItemFn) {
            profile_fn!(visit_item_fn);
            if node.sig.ident == "main" && node.sig.inputs.is_empty() {
                self.main_method_count += 1;
            }
        }
    }

    profile_fn!(count_main_methods);
    let mut finder = FindMainFns::default();

    match syntax_tree {
        Ast::File(ast) => finder.visit_file(ast),
        Ast::Expr(ast) => finder.visit_expr(ast),
    }

    debug_log!(
        "In count_main_methods: finder.main_method_count={}",
        finder.main_method_count
    );

    finder.main_method_count
}

/// Parse the code into an abstract syntax tree for inspection
/// if possible (should work if the code will compile)
#[must_use]
pub fn to_ast(source_code: &str) -> Option<Ast> {
    profile_fn!(to_ast);
    let start_ast = Instant::now();
    if let Ok(tree) = syn::parse_file(source_code) {
        // Highlight the unexpected
        log!(
            Verbosity::Quiet,
            "{}",
            nu_resolve_style(crate::MessageLevel::Warning).paint("Parsed to syn::File")
        );
        #[cfg(debug_assertions)]
        debug_timings(&start_ast, "Completed successful AST parse to syn::File");
        Some(Ast::File(tree))
    } else if let Ok(tree) = extract_ast_expr(source_code) {
        #[cfg(debug_assertions)]
        debug_timings(&start_ast, "Completed successful AST parse to syn::Expr");
        log!(
            Verbosity::Quiet,
            "{}",
            nu_resolve_style(crate::MessageLevel::Emphasis).paint("Parsed to syn::Expr")
        );
        Some(Ast::Expr(tree))
    } else {
        log!(
            Verbosity::Quieter,
            "{}",
            nu_resolve_style(crate::MessageLevel::Warning)
                .paint("Error parsing syntax tree. Using regex to help you debug the script.")
        );
        #[cfg(debug_assertions)]
        debug_timings(&start_ast, "Completed unsuccessful AST parse");
        None
    }
}

type Zipped<'a> = (Vec<Option<&'a str>>, Vec<Option<&'a str>>);

/// Prepare a snippet for wrapping in `fn main` by separating out any inner attributes,
/// as they need to be promoted to crate level.
#[must_use]
pub fn extract_inner_attribs(rs_source: &str) -> (String, String) {
    use std::fmt::Write;

    lazy_static! {
        static ref INNER_ATTRIB_REGEX: Regex = Regex::new(r"(?m)^[\s]*#!\[.+\]").unwrap();
    }

    profile_fn!(extract_inner_attribs);

    #[cfg(debug_assertions)]
    debug_log!("rs_source={rs_source}");

    let (inner_attribs, rest): Zipped = rs_source
        .lines()
        .map(|line| -> (Option<&str>, Option<&str>) {
            if INNER_ATTRIB_REGEX.is_match(line) {
                (Some(line), None)
            } else {
                (None, Some(line))
            }
        })
        .unzip();

    // eprintln!("inner_attribs={inner_attribs:#?}\nrest={rest:#?}");
    let inner_attribs = inner_attribs
        .iter()
        .flatten()
        .fold(String::new(), |mut output, &b| {
            let _ = writeln!(output, "{b}");
            output
        });

    let rest = rest.iter().flatten().fold(String::new(), |mut output, &b| {
        let _ = writeln!(output, "    {b}");
        output
    });
    (inner_attribs, rest)
}

/// Convert a Rust code snippet into a program by wrapping it in a main method and other scaffolding.
#[must_use]
pub fn wrap_snippet(inner_attribs: &str, body: &str) -> String {
    profile_fn!(wrap_snippet);
    #[cfg(debug_assertions)]
    debug_log!("In wrap_snippet");

    #[cfg(debug_assertions)]
    debug_log!("In wrap_snippet: inner_attribs={inner_attribs:#?}");
    let wrapped_snippet = format!(
        r##"#![allow(unused_imports,unused_macros,unused_variables,dead_code)]
{inner_attribs}
use std::error::Error;
use std::io;
use std::io::prelude::*;

#[doc = "Wrapped snippet in main method to make it a program."]
fn main() -> Result<(), Box<dyn Error>> {{
{body}
Ok(())
}}
"##
    );
    #[cfg(debug_assertions)]
    debug_log!("wrapped_snippet={wrapped_snippet}");
    wrapped_snippet
}

/// Write the source to the destination source-code path.
/// # Errors
/// Will return `Err` if there is any error encountered opening or writing to the file.
pub fn write_source(to_rs_path: &PathBuf, rs_source: &str) -> Result<fs::File, ThagError> {
    profile_fn!(write_source);
    let mut to_rs_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(to_rs_path)?;
    // #[cfg(debug_assertions)]
    // debug_log!("Writing out source to {to_rs_path:#?}:\n{}", {
    //     let lines = rs_source.lines();
    //     reassemble(lines)
    // });
    to_rs_file.write_all(rs_source.as_bytes())?;

    #[cfg(debug_assertions)]
    debug_log!("Done!");

    Ok(to_rs_file)
}

/// Create the next sequential REPL file according to the `repl_nnnnnn.rs` standard used by this crate.
/// # Errors
/// Will return `Err` if there is any error encountered creating the next `repl_nnnnnnn/repl_nnnnnn.rs` in `$TMPDIR/REPL_SUBDIR`.
pub fn create_next_repl_file() -> Result<PathBuf, ThagError> {
    profile_fn!(create_next_repl_file);
    // Create a directory inside of `std::env::temp_dir()`
    let gen_repl_temp_dir_path = TMPDIR.join(REPL_SUBDIR);

    #[cfg(debug_assertions)]
    debug_log!("repl_temp_dir = std::env::temp_dir() = {gen_repl_temp_dir_path:?}");

    // Ensure REPL subdirectory exists
    fs::create_dir_all(&gen_repl_temp_dir_path)?;

    // Find existing dirs with the pattern repl_<nnnnnn>
    let mut existing_dirs = Vec::new();

    for entry in fs::read_dir(&gen_repl_temp_dir_path)? {
        let entry = entry?; // Bubbles up I/O errors
        let path = entry.path();

        if path.is_dir() {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if stem.starts_with("repl_") {
                    let num_str = stem.trim_start_matches("repl_");
                    if num_str.len() == 6 && num_str.chars().all(char::is_numeric) {
                        if let Ok(num) = num_str.parse::<u32>() {
                            existing_dirs.push(num);
                        }
                    }
                }
            }
        }
    }

    let next_file_num = match existing_dirs.as_slice() {
        [] => 0, // No existing files, start with 000000
        _ if existing_dirs.contains(&999_999) => {
            // Wrap around and find the first gap
            for i in 0..999_999 {
                if !existing_dirs.contains(&i) {
                    return create_repl_file(&gen_repl_temp_dir_path, i);
                }
            }
            return Err(format!("Cannot create new file: all possible subdirs `repl_nnnnnn` already exist in {TMPDIR:?}/{REPL_SUBDIR}.").into());
        }
        _ => {
            existing_dirs
                .iter()
                .max()
                .ok_or("Can't get max value iterating existing repl_nnnnnn subdirectories")?
                + 1
        } // Increment from highest existing number
    };

    create_repl_file(&gen_repl_temp_dir_path, next_file_num)
}

/// Create a REPL file on disk, given the path and sequence number.
/// # Errors
/// Will return `Err` if it fails to create the next `repl_nnnnnnn/repl_nnnnnn.rs` in `$TMPDIR/REPL_SUBDIR`.
pub fn create_repl_file(gen_repl_temp_dir_path: &Path, num: u32) -> Result<PathBuf, ThagError> {
    profile_fn!(create_repl_file);
    let padded_num = format!("{:06}", num);
    let dir_name = format!("repl_{padded_num}");
    let target_dir_path = gen_repl_temp_dir_path.join(dir_name);
    fs::create_dir_all(&target_dir_path)?;

    let filename = format!("repl_{padded_num}.rs");
    let path = target_dir_path.join(filename);
    fs::File::create(&path)?;
    #[cfg(debug_assertions)]
    debug_log!("Created file: {path:#?}");
    Ok(path)
}

/// Create empty script file `temp.rs` to hold expression for --expr or --stdin options,
/// and open it for writing.
/// # Errors
/// Will return Err if it can't create the `rs_dyn` directory.
pub fn create_temp_source_file() -> Result<PathBuf, ThagError> {
    // Create a directory inside of `std::env::temp_dir()`
    let gen_expr_temp_dir_path = TMPDIR.join(DYNAMIC_SUBDIR);

    // Ensure REPL subdirectory exists
    fs::create_dir_all(&gen_expr_temp_dir_path)?;

    let filename = TEMP_SCRIPT_NAME;
    let path = gen_expr_temp_dir_path.join(filename);
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;
    #[cfg(debug_assertions)]
    debug_log!("Created file: {path:#?}");
    Ok(path)
}

/// Combine the elements of a loop filter into a well-formed program.
#[must_use]
pub fn build_loop(args: &Cli, filter: String) -> String {
    profile_fn!(build_loop);
    let maybe_ast = extract_ast_expr(&filter);
    let returns_unit = if let Ok(expr) = maybe_ast {
        is_unit_return_type(&expr)
    } else {
        let expr_any: &dyn Any = &filter;
        dbg!(&filter);
        !expr_any.is::<()>()
    };
    let loop_toml = &args.toml;
    let loop_begin = &args.begin;
    let loop_end = &args.end;
    let filter = if returns_unit {
        filter
    } else {
        format!(r#"println!("{{:?}}", {});"#, filter)
    };
    // dbg!(&filter);

    format!(
        r#"{}
use std::io::{{self, BufRead}};
fn main() -> Result<(), Box<dyn std::error::Error>> {{
    {}
    // Read from stdin and execute main loop for each line
    #[allow(unused_variables)]
    let mut i = 0;
    let stdin = io::stdin();
    for line in stdin.lock().lines() {{
        let line = line?;
        i += 1;
        {filter}
    }}
    {}
    Ok(())
}}
"#,
        if let Some(ref toml) = &loop_toml {
            log!(Verbosity::Verbose, "toml={toml}");
            format!(
                r#"/*[toml]
{toml}
*/"#
            )
        } else {
            String::new()
        },
        if let Some(prelude) = loop_begin {
            log!(Verbosity::Verbose, "prelude={prelude}");
            prelude
        } else {
            ""
        },
        if let Some(postlude) = loop_end {
            log!(Verbosity::Verbose, "postlude={postlude}");
            postlude
        } else {
            ""
        }
    )
}

/// Clean up temporary files.
/// # Errors
/// Will return `Err` if there is any error deleting the file.
pub fn clean_up(source_path: &PathBuf, target_dir_path: &PathBuf) -> io::Result<()> {
    profile_fn!(clean_up);
    // Delete the file
    remove_file(source_path)?;

    // Remove the directory and its contents recursively
    remove_dir_all(target_dir_path)
}

/// Display the contents of a given directory.
/// # Errors
/// Will return `Err` if there is any error reading the directory.
pub fn display_dir_contents(path: &PathBuf) -> io::Result<()> {
    profile_fn!(display_dir_contents);
    if path.is_dir() {
        let entries = fs::read_dir(path)?;

        log!(Verbosity::Normal, "Directory listing for {:?}", path);
        for entry in entries {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let file_name = entry.file_name();
            log!(
                Verbosity::Quieter,
                "  {file_name:?} ({})",
                if file_type.is_dir() {
                    "Directory"
                } else {
                    "File"
                }
            );
        }
    }
    Ok(())
}

/// Format a Rust source file in situ using rustfmt.
/// # Errors
/// Will return `Err` if there is any error accessing path to source file
/// # Panics
/// Will panic if the `rustfmt` failed.
pub fn rustfmt(build_state: &BuildState) -> Result<(), ThagError> {
    profile_fn!(rustfmt);
    let target_rs_path = build_state.target_dir_path.join(&build_state.source_name);
    let source_path_str = target_rs_path
        .to_str()
        .ok_or(String::from("Error accessing path to source file"))?;

    if Command::new("rustfmt").arg("--version").output().is_ok() {
        // Run rustfmt on the source file
        let mut command = Command::new("rustfmt");
        command.arg("--verbose");
        command.arg("--edition");
        command.arg("2021");
        command.arg(source_path_str);
        let output = command.output()?;

        #[cfg(debug_assertions)]
        if output.status.success() {
            debug_log!("Successfully formatted {} with rustfmt.", source_path_str);
            debug_log!(
                "{source_path_str}\n{}",
                String::from_utf8_lossy(&output.stdout)
            );
        } else {
            debug_log!(
                "Failed to format {source_path_str} with rustfmt\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    } else {
        log!(
            Verbosity::Quieter,
            "`rustfmt` not found. Please install it to use this script."
        );
    }
    Ok(())
}

/// Strip a set of curly braces off a Rust script, if present. This is intended to
/// undo the effect of adding them to create an expression that can be parsed into
/// an abstract syntax tree.
#[must_use]
pub fn strip_curly_braces(haystack: &str) -> Option<String> {
    profile_fn!(strip_curly_braces);
    // Define the regex pattern
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?s)^\s*\{\s*(.*?)\s*\}\s*$").unwrap();
    }

    // Apply the regex to the input string
    RE.captures(haystack)
        .map(|captures| captures[1].to_string())
}

/// Cache any functions that we may find in a snippet expression in a Hashmap, so
/// that if the last statement in the expression is a function call, we can look
/// up its return type and determine whether to wrap it in a println! statement.
fn extract_functions(expr: &syn::Expr) -> HashMap<String, ReturnType> {
    #[derive(Default)]
    struct FindFns {
        function_map: HashMap<String, ReturnType>,
    }

    impl<'ast> Visit<'ast> for FindFns {
        fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
            // #[cfg(debug_assertions)]
            // {
            //     debug_log!("Node={:#?}", node);
            //     debug_log!("Ident={}", node.sig.ident);
            //     debug_log!("Output={:#?}", &node.sig.output);
            // }
            self.function_map
                .insert(node.sig.ident.to_string(), node.sig.output.clone());
        }
    }

    let mut finder = FindFns::default();
    finder.visit_expr(expr);

    finder.function_map
}

/// Determine if the return type of the expression is unit (the empty tuple `()`),
/// otherwise we wrap it in a println! statement.
#[must_use]
pub fn is_unit_return_type(expr: &Expr) -> bool {
    profile_fn!(is_unit_return_type);
    let start = Instant::now();

    let function_map = extract_functions(expr);
    #[cfg(debug_assertions)]
    // debug_log!("function_map={function_map:#?}");
    let is_last_stmt_unit_type = is_last_stmt_unit_type(expr, &function_map);
    #[cfg(debug_assertions)]
    debug_timings(&start, "Determined probable snippet return type");
    is_last_stmt_unit_type
}

/// Recursively alternate with function `is_stmt_unit_type` until we drill down through
/// all the blocks, loops and if-conditions to find the last executable statement and
/// determine if it returns a unit type or a value worth printing.
///
/// This function finds the last statement in a given expression and determines if it
/// returns a unit type.
/// # Panics
/// Will panic if an unexpected expression type is found in the elso branch of an if-statement.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn is_last_stmt_unit_type<S: ::std::hash::BuildHasher>(
    expr: &Expr,
    function_map: &HashMap<String, ReturnType, S>,
) -> bool {
    profile_fn!(is_last_stmt_unit_type);
    #[cfg(debug_assertions)]
    debug_log!("%%%%%%%% expr={expr:#?}");
    match expr {
        Expr::ForLoop(for_loop) => {
            // #[cfg(debug_assertions)]
            // debug_log!("%%%%%%%% Expr::ForLoop(for_loop))");
            if let Some(last_stmt) = for_loop.body.stmts.last() {
                is_stmt_unit_type(last_stmt, function_map)
            } else {
                // #[cfg(debug_assertions)]
                // debug_log!("%%%%%%%% Not if let Some(last_stmt) = for_loop.body.stmts.last()");
                false
            }
        }
        Expr::If(expr_if) => {
            // Cycle through if-else statements and return false if any one is found returning
            // a non-unit value;
            if let Some(last_stmt) = expr_if.then_branch.stmts.last() {
                // #[cfg(debug_assertions)]
                // debug_log!("%%%%%%%% Some(last_stmt) = expr_if.then_branch.stmts.last()");
                if !is_stmt_unit_type(last_stmt, function_map) {
                    return false;
                };
                if let Some(ref stmt) = expr_if.else_branch {
                    let expr_else = &*stmt.1;
                    // The else branch expression may only be an If or Block expression,
                    // not any of the other types of expression.
                    match expr_else {
                        // If it's a block, we're at the end of the if-else chain and can just
                        // decide according to the return type of the last statement in the block.
                        Expr::Block(expr_block) => {
                            let else_is_unit_type =
                                if let Some(last_stmt) = expr_block.block.stmts.last() {
                                    // #[cfg(debug_assertions)]
                                    // debug_log!("%%%%%%%% If let Some(last_stmt) = expr_block.block.stmts.last()");
                                    is_stmt_unit_type(last_stmt, function_map)
                                } else {
                                    // #[cfg(debug_assertions)]
                                    // debug_log!("%%%%%%%% Not if let Some(last_stmt) = expr_block.block.stmts.last()");
                                    false
                                };
                            else_is_unit_type
                        }
                        // If it's another if-statement, simply recurse through this method.
                        Expr::If(_) => is_last_stmt_unit_type(expr_else, function_map),
                        _ => panic!("Expected else branch expression to be If or Block"),
                    }
                } else {
                    true
                }
            } else {
                // #[cfg(debug_assertions)]
                // debug_log!(
                //     "%%%%%%%% Not if let Some(last_stmt) = expr_if.then_branch.stmts.last()"
                // );
                false
            }
        }
        Expr::Block(expr_block) => {
            if expr_block.block.stmts.is_empty() {
                return true;
            }
            if let Some(last_stmt) = expr_block.block.stmts.last() {
                // #[cfg(debug_assertions)]
                // debug_log!("%%%%%%%% if let Some(last_stmt) = expr_block.block.stmts.last()");
                is_stmt_unit_type(last_stmt, function_map)
            } else {
                // #[cfg(debug_assertions)]
                // debug_log!("%%%%%%%% Not if let Some(last_stmt) = expr_block.block.stmts.last()");
                false
            }
        }
        Expr::Match(expr_match) => {
            for arm in &expr_match.arms {
                // #[cfg(debug_assertions)]
                // debug_log!("arm.body={:#?}", arm.body);
                let expr = &*arm.body;
                if is_last_stmt_unit_type(expr, function_map) {
                    continue;
                }
                return false;
            }
            // #[cfg(debug_assertions)]
            // debug_log!("%%%%%%%% Match arm returns non-unit type");
            true
        }
        Expr::Call(expr_call) => {
            if let Expr::Path(path) = &*expr_call.func {
                if let Some(value) = is_path_unit_type(path, function_map) {
                    return value;
                }
            }

            false
        }
        Expr::Closure(expr_closure) => match &expr_closure.output {
            ReturnType::Default => is_last_stmt_unit_type(&expr_closure.body, function_map),
            ReturnType::Type(_, ty) => {
                if let syn::Type::Tuple(tuple) = &**ty {
                    tuple.elems.is_empty()
                } else {
                    false
                }
            }
        },
        Expr::MethodCall(expr_method_call) => {
            is_last_stmt_unit_type(&expr_method_call.receiver, function_map)
        }
        Expr::Binary(expr_binary) => matches!(
            expr_binary.op,
            syn::BinOp::AddAssign(_)
                | syn::BinOp::SubAssign(_)
                | syn::BinOp::MulAssign(_)
                | syn::BinOp::DivAssign(_)
                | syn::BinOp::RemAssign(_)
                | syn::BinOp::BitXorAssign(_)
                | syn::BinOp::BitAndAssign(_)
                | syn::BinOp::BitOrAssign(_)
                | syn::BinOp::ShlAssign(_)
                | syn::BinOp::ShrAssign(_)
        ),
        Expr::While(_)
        | Expr::Loop(_)
        | Expr::Break(_)
        | Expr::Continue(_)
        | Expr::Infer(_)
        | Expr::Let(_) => true,
        Expr::Array(_)
        | Expr::Assign(_)
        | Expr::Async(_)
        | Expr::Await(_)
        | Expr::Cast(_)
        | Expr::Const(_)
        | Expr::Field(_)
        | Expr::Group(_)
        | Expr::Index(_)
        | Expr::Lit(_)
        | Expr::Paren(_)
        | Expr::Range(_)
        | Expr::Reference(_)
        | Expr::Repeat(_)
        | Expr::Struct(_)
        | Expr::Try(_)
        | Expr::TryBlock(_)
        | Expr::Tuple(_)
        | Expr::Unary(_)
        | Expr::Unsafe(_)
        | Expr::Verbatim(_)
        | Expr::Yield(_) => false,
        Expr::Macro(expr_macro) => {
            if let Some(segment) = expr_macro.mac.path.segments.last() {
                let ident = &segment.ident.to_string();
                return ident.starts_with("print")
                    || ident.starts_with("write")
                    || ident.starts_with("debug");
            }
            false // default - because no intrinsic way of knowing?
        }
        Expr::Path(path) => {
            if let Some(value) = is_path_unit_type(path, function_map) {
                return value;
            }
            false
        }
        Expr::Return(expr_return) => {
            #[cfg(debug_assertions)]
            debug_log!("%%%%%%%% expr_return={expr_return:#?}");
            expr_return.expr.is_none()
        }
        _ => {
            println!(
                "%%%%%%%% Expression not catered for: {expr:#?}, wrapping expression in println!()"
            );
            false
        }
    }
}

/// Check if a path represents a function, and if so, whether it has a unit or non-unit
/// return type.
#[must_use]
pub fn is_path_unit_type<S: ::std::hash::BuildHasher>(
    path: &syn::PatPath,
    function_map: &HashMap<String, ReturnType, S>,
) -> Option<bool> {
    profile_fn!(is_path_unit_type);
    if let Some(ident) = path.path.get_ident() {
        if let Some(return_type) = function_map.get(&ident.to_string()) {
            return Some(match return_type {
                ReturnType::Default => {
                    // #[cfg(debug_assertions)]
                    // debug_log!("%%%%%%%% ReturnType::Default");
                    true
                }
                ReturnType::Type(_, ty) => {
                    if let syn::Type::Tuple(tuple) = &**ty {
                        // #[cfg(debug_assertions)]
                        // debug_log!("%%%%%%%% Tuple ReturnType");
                        tuple.elems.is_empty()
                    } else {
                        // #[cfg(debug_assertions)]
                        // debug_log!("%%%%%%%% Non-unit return type");
                        false
                    }
                }
            });
        }
    }
    None
}

/// Recursively alternate with function `is_last_stmt_unit` until we drill down through
/// all the blocks, loops and if-conditions to find the last executable statement and
/// determine if it returns a unit type or a value worth printing.
pub fn is_stmt_unit_type<S: ::std::hash::BuildHasher>(
    stmt: &Stmt,
    function_map: &HashMap<String, ReturnType, S>,
) -> bool {
    profile_fn!(is_stmt_unit_type);
    #[cfg(debug_assertions)]
    debug_log!("%%%%%%%% stmt={stmt:#?}");
    match stmt {
        Stmt::Expr(expr, None) => {
            // #[cfg(debug_assertions)]
            // {
            //     debug_log!("%%%%%%%% expr={expr:#?}");
            //     debug_log!("%%%%%%%% Stmt::Expr(_, None)");
            // }
            is_last_stmt_unit_type(expr, function_map)
        } // Expression without semicolon
        Stmt::Expr(expr, Some(_)) => {
            // #[cfg(debug_assertions)]
            // debug_log!("%%%%%%%% Stmt::Expr(_, Some(_))");
            match expr {
                Expr::Return(expr_return) => {
                    #[cfg(debug_assertions)]
                    debug_log!("%%%%%%%% expr_return={expr_return:#?}");
                    expr_return.expr.is_none()
                }
                Expr::Yield(expr_yield) => {
                    #[cfg(debug_assertions)]
                    debug_log!("%%%%%%%% expr_yield={expr_yield:#?}");
                    expr_yield.expr.is_none()
                }
                _ => true,
            }
        } // Expression with semicolon usually returns unit, except sometimes return or yield.
        Stmt::Macro(m) => {
            // #[cfg(debug_assertions)]
            // debug_log!("%%%%%%%% Stmt::Macro({m:#?}), m.semi_token.is_some()={is_some}");
            m.semi_token.is_some()
        }
        Stmt::Local(_) => true,
        Stmt::Item(item) => match item {
            Item::ExternCrate(_)
            | Item::Fn(_)
            | Item::ForeignMod(_)
            | Item::Impl(_)
            | Item::Struct(_)
            | Item::Trait(_)
            | Item::TraitAlias(_)
            | Item::Type(_)
            | Item::Union(_)
            | Item::Use(_)
            | Item::Mod(_) => true,
            Item::Macro(m) => {
                // #[cfg(debug_assertions)]
                // debug_log!("%%%%%%%% Item::Macro({m:#?}), m.semi_token.is_some()={is_some}");
                m.semi_token.is_some()
            }
            _ => false, // default
        },
    }
}

// /// Check if an expression returns a unit value, so that we can avoid trying to print it out.
// #[must_use]
// pub fn returns_unit(expr: &Expr) -> bool {
//     profile_fn!(returns_unit);
//     let is_unit_type = matches!(expr, Expr::Tuple(tuple) if tuple.elems.is_empty());
//     nu_color_println!(
//         nu_resolve_style(crate::MessageLevel::Emphasis),
//         "is_unit_type={is_unit_type}"
//     );
//     is_unit_type
// }

/// Convert a `syn::File`'s `fn main` to a `syn::Expr`, for the purpose of determining the return type.
/// # Errors
/// Will return `Err` if there is any error parsing expressions
// #[cfg(debug_assertions)]
// #[allow(dead_code)]
// pub fn extract_expr_from_file(file: &File) -> Result<Expr, ThagError> {
//     profile_fn!(extract_expr_from_file);

//     for item in &file.items {
//         if let Item::Fn(func) = item {
//             if func.sig.ident == "main" {
//                 // Convert the function's block into an Expr::Block
//                 let expr_block = Expr::Block(ExprBlock {
//                     attrs: Vec::new(),          // Add attributes if needed
//                     label: None,                // No label for the block
//                     block: *func.block.clone(), // Clone the block from the function
//                 });

//                 return Ok(expr_block);
//             }
//         }
//     }

//     Err("No main function found".into())
// }

/// # Errors
/// Will return `Err` if there is any error parsing expressions
pub fn is_main_fn_returning_unit(file: &File) -> Result<bool, ThagError> {
    profile_fn!(is_main_fn_returning_unit);

    // Traverse the file to find the main function
    for item in &file.items {
        if let Item::Fn(func) = item {
            if func.sig.ident == "main" {
                // Check if the return type is the unit type
                let is_unit_return_type = matches!(func.sig.output, ReturnType::Default);

                return Ok(is_unit_return_type);
            }
        }
    }

    Err("No main function found".into())
}
