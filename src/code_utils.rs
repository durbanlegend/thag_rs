#![allow(
    clippy::uninlined_format_args,
    clippy::implicit_return,
    clippy::missing_trait_methods
)]
#[cfg(debug_assertions)]
use crate::debug_timings;
#[cfg(target_os = "windows")]
use crate::escape_path_for_windows;
use crate::{
    cvprtln, debug_log,
    shared::{find_crates, find_metadata},
    vlog, Ast, BuildState, Cli, Lvl, ThagError, ThagResult, BUILT_IN_CRATES, DYNAMIC_SUBDIR,
    TEMP_SCRIPT_NAME, TMPDIR, V,
};
use cargo_toml::{Edition, Manifest};
use firestorm::{profile_fn, profile_method, profile_section};
use regex::Regex;
use std::{
    any::Any,
    collections::{HashMap, HashSet},
    fs::{self, remove_dir_all, remove_file, OpenOptions},
    hash::BuildHasher,
    io::{self, BufRead, Write},
    ops::Deref,
    option::Option,
    path::{Path, PathBuf},
    process::{self, Command, Output},
    time::{Instant, SystemTime},
};
use syn::{
    parse_file,
    visit::Visit,
    visit_mut::{self, VisitMut},
    AttrStyle,
    BinOp::{
        AddAssign, BitAndAssign, BitOrAssign, BitXorAssign, DivAssign, MulAssign, RemAssign,
        ShlAssign, ShrAssign, SubAssign,
    },
    Expr, ExprBlock, File, Item, ReturnType, Stmt,
    Type::Tuple,
};

// From burntsushi at `https://github.com/rust-lang/regex/issues/709`
#[macro_export]
macro_rules! regex {
    ($re:literal $(,)?) => {{
        use {regex::Regex, std::sync::OnceLock};

        static RE: OnceLock<Regex> = OnceLock::new();
        RE.get_or_init(|| Regex::new($re).unwrap())
    }};
}

// To move inner attributes out of a syn AST for a snippet.
struct RemoveInnerAttributes {
    found: bool,
}

impl VisitMut for RemoveInnerAttributes {
    fn visit_expr_block_mut(&mut self, expr_block: &mut ExprBlock) {
        profile_method!(visit_expr_block_mut);
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
pub fn read_file_contents(path: &Path) -> ThagResult<String> {
    profile_fn!(read_file_contents);

    debug_log!("Reading from {path:?}");
    Ok(fs::read_to_string(path)?)
}

/// Infer dependencies from AST-derived metadata to put in a Cargo.toml.
#[must_use]
pub fn infer_deps_from_ast(
    crates_finder: &crate::shared::CratesFinder,
    metadata_finder: &crate::shared::MetadataFinder,
) -> Vec<String> {
    profile_fn!(infer_deps_from_ast);
    let mut dependencies = vec![];
    dependencies.extend_from_slice(&crates_finder.crates);
    let to_remove: HashSet<String> = crates_finder
        .names_to_exclude
        .iter()
        .cloned()
        .chain(metadata_finder.names_to_exclude.iter().cloned())
        .chain(metadata_finder.mods_to_exclude.iter().cloned())
        .chain(BUILT_IN_CRATES.iter().map(Deref::deref).map(String::from))
        .collect();
    // eprintln!("to_remove={to_remove:#?}");

    dependencies.retain(|e| !to_remove.contains(e));
    // eprintln!("dependencies (after)={dependencies:#?}");

    // Similar check for other regex pattern
    for crate_name in &metadata_finder.extern_crates {
        if !&to_remove.contains(crate_name) {
            dependencies.push(crate_name.to_owned());
        }
    }

    // Deduplicate the list of dependencies
    dependencies.sort();
    dependencies.dedup();

    dependencies
}

/// Infer dependencies from source code to put in a Cargo.toml.
/// Fallback version for when an abstract syntax tree cannot be parsed.
#[must_use]
pub fn infer_deps_from_source(code: &str) -> Vec<String> {
    profile_fn!(infer_deps_from_source);

    if code.trim().is_empty() {
        return vec![];
    }

    let maybe_ast = extract_and_wrap_uses(code);
    let mut dependencies = maybe_ast.map_or_else(
        |_| {
            cvprtln!(
                Lvl::ERR,
                V::QQ,
                "Could not parse code into an abstract syntax tree"
            );
            vec![]
        },
        |ast| {
            let crates_finder = find_crates(&ast);
            let metadata_finder = find_metadata(&ast);
            infer_deps_from_ast(&crates_finder, &metadata_finder)
        },
    );

    let macro_use_regex: &Regex = regex!(r"(?m)^[\s]*#\[macro_use\((\w+)\)");
    let extern_crate_regex: &Regex = regex!(r"(?m)^[\s]*extern\s+crate\s+([^;{]+)");

    let modules = find_modules_source(code);

    dependencies.retain(|e| !modules.contains(e));
    // eprintln!("dependencies (after)={dependencies:#?}");

    for cap in macro_use_regex.captures_iter(code) {
        let crate_name = cap[1].to_string();
        // eprintln!("macro-use crate_name={crate_name:#?}");
        if !modules.contains(&crate_name) {
            dependencies.push(crate_name);
        }
    }

    for cap in extern_crate_regex.captures_iter(code) {
        let crate_name = cap[1].to_string();
        // eprintln!("extern-crate crate_name={crate_name:#?}");
        if !modules.contains(&crate_name) {
            dependencies.push(crate_name);
        }
    }
    dependencies.sort();
    dependencies
}

// Function to extract use statements and wrap in braces for parsing
fn extract_and_wrap_uses(source: &str) -> Result<Ast, syn::Error> {
    profile_fn!(extract_and_wrap_uses);
    // Step 1: Capture `use` statements
    let use_simple_regex: &Regex = regex!(r"(?m)(^\s*use\s+[^;{]+;\s*$)");
    let use_nested_regex: &Regex = regex!(r"(?ms)(^\s*use\s+\{.*\};\s*$)");

    let mut use_statements: Vec<String> = vec![];

    for cap in use_simple_regex.captures_iter(source) {
        let use_string = cap[1].to_string();
        use_statements.push(use_string);
    }
    for cap in use_nested_regex.captures_iter(source) {
        let use_string = cap[1].to_string();
        use_statements.push(use_string);
    }

    // Step 2: Parse as `syn::File`
    let ast: File = parse_file(&use_statements.join("\n"))?;
    // eprintln!("ast={ast:#?}");

    // Return wrapped in `Ast::File`
    Ok(Ast::File(ast))
}

/// Identify use ... as statements for inclusion in / exclusion from Cargo.toml metadata.
///
/// Include the "from" name and exclude the "to" name.
/// Fallback version for when an abstract syntax tree cannot be parsed.
#[must_use]
pub fn find_use_renames_source(code: &str) -> (Vec<String>, Vec<String>) {
    profile_fn!(find_use_renames_source);

    debug_log!("In code_utils::find_use_renames_source");
    let use_as_regex: &Regex = regex!(r"(?m)^\s*use\s+(\w+).*? as\s+(\w+)");

    let mut use_renames_from: Vec<String> = vec![];
    let mut use_renames_to: Vec<String> = vec![];

    for cap in use_as_regex.captures_iter(code) {
        let from_name = cap[1].to_string();
        let to_name = cap[2].to_string();

        debug_log!("use_rename: from={from_name}, to={to_name}");
        use_renames_from.push(from_name);
        use_renames_to.push(to_name);
    }

    use_renames_from.sort();
    use_renames_from.dedup();

    debug_log!("use_renames from source: from={use_renames_from:#?}, to={use_renames_to:#?}");
    (use_renames_from, use_renames_to)
}

/// Identify mod statements for exclusion from Cargo.toml metadata.
/// Fallback version for when an abstract syntax tree cannot be parsed.
#[must_use]
pub fn find_modules_source(code: &str) -> Vec<String> {
    profile_fn!(find_modules_source);
    let module_regex: &Regex = regex!(r"(?m)^[\s]*mod\s+([^;{\s]+)");

    debug_log!("In code_utils::find_use_renames_source");

    let mut modules: Vec<String> = vec![];

    for cap in module_regex.captures_iter(code) {
        let module = cap[1].to_string();

        debug_log!("module={module}");
        modules.push(module);
    }

    debug_log!("modules from source={modules:#?}");
    modules
}

/// Extract embedded Cargo.toml metadata from a Rust source string.
/// # Errors
/// Will return `Err` if there is any error in parsing the toml data into a manifest.
pub fn extract_manifest(
    rs_full_source: &str,
    #[allow(unused_variables)] start_parsing_rs: Instant,
) -> ThagResult<Manifest> {
    profile_fn!(extract_manifest);
    let maybe_rs_toml = extract_toml_block(rs_full_source);

    profile_section!(parse_and_set_edition);
    let mut rs_manifest = if let Some(rs_toml_str) = maybe_rs_toml {
        // debug_log!("rs_toml_str={rs_toml_str}");
        Manifest::from_str(&rs_toml_str)?
    } else {
        Manifest::from_str("")?
    };

    {
        profile_section!(set_edition);
        if let Some(package) = rs_manifest.package.as_mut() {
            package.edition = cargo_toml::Inheritable::Set(Edition::E2021);
        }
    }

    // debug_log!("rs_manifest={rs_manifest:#?}");

    #[cfg(debug_assertions)]
    debug_timings(&start_parsing_rs, "extract_manifest parsed source");
    Ok(rs_manifest)
}

fn extract_toml_block(input: &str) -> Option<String> {
    profile_fn!(extract_toml_block);
    let re: &Regex = regex!(r"(?s)/\*\[toml\](.*?)\*/");
    re.captures(input)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

/// Parse a Rust expression source string into a syntax tree.
///
/// Although this is primarily intended for incomplete snippets and expressions, if it finds a fully-fledged program that
/// could equally be parsed with `syn::parse_file`, it should succeed anyway by wrapping it in braces. However that is the
/// snippet path and is not encouraged as it is likely to process and wrap the code unnecessarily.
/// # Errors
/// Will return `Err` if there is any error encountered by the `syn` crate trying to parse the source string into an AST.
pub fn extract_ast_expr(rs_source: &str) -> Result<Expr, syn::Error> {
    profile_fn!(extract_ast_expr);
    let mut expr: Result<Expr, syn::Error> = syn::parse_str::<Expr>(rs_source);
    if expr.is_err() && !(rs_source.starts_with('{') && rs_source.ends_with('}')) {
        // Try putting the expression in braces.
        let string = format!(r"{{{rs_source}}}");
        let str = string.as_str();
        // vlog!(V::N, "str={str}");

        expr = syn::parse_str::<Expr>(str);
    }
    expr
}

/// Convert a Path to a string value, assuming the path contains only valid characters.
/// # Errors
/// Will return `Err` if there is any error caused by invalid characters in the path name.
pub fn path_to_str(path: &Path) -> ThagResult<String> {
    profile_fn!(path_to_str);
    let string = path
        .to_path_buf()
        .into_os_string()
        .into_string()
        .map_err(ThagError::OsString)?;

    debug_log!("path_to_str={string}");
    Ok(string)
}

/// Reassemble an Iterator of lines from the disentangle function to a string of text.
#[inline]
pub fn reassemble<'a>(map: impl Iterator<Item = &'a str>) -> String {
    use std::fmt::Write;
    profile_fn!(reassemble);
    map.fold(String::new(), |mut output, b| {
        let _ = writeln!(output, "{b}");
        output
    })
}

/// Unescape \n markers to convert a string of raw text to readable lines.
#[inline]
#[must_use]
pub fn disentangle(text_wall: &str) -> String {
    profile_fn!(disentangle);
    reassemble(text_wall.lines())
}

#[warn(dead_code)]
/// Display output captured to `std::process::Output`.
/// # Errors
/// Will return `Err` if the stdout or stderr is not found captured as expected.
pub fn display_output(output: &Output) -> ThagResult<()> {
    profile_fn!(display_output);
    // Read the captured output from the pipe
    // let stdout = output.stdout;

    // Print the captured stdout
    vlog!(V::N, "Captured stdout:");
    for result in output.stdout.lines() {
        vlog!(V::N, "{}", result?);
    }

    // Print the captured stderr
    vlog!(V::N, "Captured stderr:");
    for result in output.stderr.lines() {
        vlog!(V::N, "{}", result?);
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
) -> ThagResult<Option<(&PathBuf, SystemTime)>> {
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
        vlog!(
            V::V,
            "The most recently modified file compared to {executable:#?} is: {file:#?}"
        );

        debug_log!("Executable modified time is{baseline_modified:#?}");
    } else {
        debug_log!("Neither file was modified more recently than {executable:#?}");
    }
    Ok(most_recent)
}

/// Parse the code into an abstract syntax tree for inspection
/// if possible (should work if the code will compile)
#[must_use]
pub fn to_ast(sourch_path_string: &str, source_code: &str) -> Option<Ast> {
    profile_fn!(to_ast);

    #[cfg(debug_assertions)]
    let start_ast = Instant::now();
    #[allow(clippy::option_if_let_else)]
    if let Ok(tree) = {
        profile_section!(to_ast_syn_parse_file);
        syn::parse_file(source_code)
    } {
        #[cfg(debug_assertions)]
        cvprtln!(&Lvl::EMPH, V::V, "Parsed to syn::File");

        #[cfg(debug_assertions)]
        debug_timings(&start_ast, "Completed successful AST parse to syn::File");
        Some(Ast::File(tree))
    } else if let Ok(tree) = {
        profile_section!(to_ast_syn_parse_expr);
        extract_ast_expr(source_code)
    } {
        #[cfg(debug_assertions)]
        cvprtln!(&Lvl::EMPH, V::V, "Parsed to syn::Expr");
        #[cfg(debug_assertions)]
        debug_timings(&start_ast, "Completed successful AST parse to syn::Expr");
        Some(Ast::Expr(tree))
    } else {
        cvprtln!(
            &Lvl::WARN,
            V::QQ,
            "Error parsing syntax tree for `{sourch_path_string}`. Using `rustfmt` to help you debug the script."
        );
        rustfmt(sourch_path_string);

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
    let inner_attrib_regex: &Regex = regex!(r"(?m)^[\s]*#!\[.+\]");

    profile_fn!(extract_inner_attribs);

    debug_log!("rs_source={rs_source}");

    let (inner_attribs, rest): Zipped = rs_source
        .lines()
        .map(|line| -> (Option<&str>, Option<&str>) {
            if inner_attrib_regex.is_match(line) {
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

    debug_log!("In wrap_snippet");

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

    debug_log!("wrapped_snippet={wrapped_snippet}");
    wrapped_snippet
}

/// Write the source to the destination source-code path.
/// # Errors
/// Will return `Err` if there is any error encountered opening or writing to the file.
pub fn write_source(to_rs_path: &PathBuf, rs_source: &str) -> ThagResult<fs::File> {
    profile_fn!(write_source);
    let mut to_rs_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(to_rs_path)?;

    // debug_log!("Writing out source to {to_rs_path:#?}:\n{}", {
    //     let lines = rs_source.lines();
    //     reassemble(lines)
    // });
    to_rs_file.write_all(rs_source.as_bytes())?;

    debug_log!("Done!");

    Ok(to_rs_file)
}

/// Create empty script file `temp.rs` to hold expression for --expr or --stdin options,
/// and open it for writing.
/// # Errors
/// Will return Err if it can't create the `rs_dyn` directory.
pub fn create_temp_source_file() -> ThagResult<PathBuf> {
    profile_fn!(create_temp_source_file);
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

    debug_log!("Created file: {path:#?}");
    Ok(path)
}

/// Combine the elements of a loop filter into a well-formed program.
#[must_use]
pub fn build_loop(args: &Cli, filter: String) -> String {
    profile_fn!(build_loop);
    let maybe_ast = extract_ast_expr(&filter);
    let returns_unit = maybe_ast.map_or_else(
        |_| {
            let expr_any: &dyn Any = &filter;
            dbg!(&filter);
            !expr_any.is::<()>()
        },
        |expr| is_unit_return_type(&expr),
    );
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
        loop_toml.as_ref().map_or_else(String::new, |toml| {
            vlog!(V::V, "toml={toml}");
            format!(
                r#"/*[toml]
{toml}
*/"#
            )
        }),
        loop_begin.as_ref().map_or("", |prelude| {
            vlog!(V::V, "prelude={prelude}");
            prelude
        }),
        loop_end.as_ref().map_or("", |postlude| {
            vlog!(V::V, "postlude={postlude}");
            postlude
        })
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

        vlog!(V::N, "Directory listing for {path:?}");
        for entry in entries {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let file_name = entry.file_name();
            vlog!(
                V::QQ,
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

/// Strip a set of curly braces off a Rust script, if present. This is intended to
/// undo the effect of adding them to create an expression that can be parsed into
/// an abstract syntax tree.
#[must_use]
pub fn strip_curly_braces(haystack: &str) -> Option<String> {
    profile_fn!(strip_curly_braces);
    // Define the regex pattern
    let re: &Regex = regex!(r"(?s)^\s*\{\s*(.*?)\s*\}\s*$");

    // Apply the regex to the input string
    re.captures(haystack)
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
        fn visit_item_fn(&mut self, i: &'ast syn::ItemFn) {
            profile_method!(extract_functions_visit_item_fn);
            // if is_debug_logging_enabled() {
            //     debug_log!("Node={:#?}", node);
            //     debug_log!("Ident={}", node.sig.ident);
            //     debug_log!("Output={:#?}", &node.sig.output);
            // }
            self.function_map
                .insert(i.sig.ident.to_string(), i.sig.output.clone());
        }
    }
    profile_fn!(extract_functions);

    let mut finder = FindFns::default();
    finder.visit_expr(expr);

    finder.function_map
}

/// Determine if the return type of the expression is unit (the empty tuple `()`),
/// otherwise we wrap it in a println! statement.
#[must_use]
#[inline]
pub fn is_unit_return_type(expr: &Expr) -> bool {
    profile_fn!(is_unit_return_type);

    #[cfg(debug_assertions)]
    let start = Instant::now();

    let function_map = extract_functions(expr);

    // debug_log!("function_map={function_map:#?}");
    let is_unit_type = is_last_stmt_unit_type(expr, &function_map);

    #[cfg(debug_assertions)]
    debug_timings(&start, "Determined probable snippet return type");
    is_unit_type
}

/// Finds the last statement in a given expression and determines if it
/// returns a unit type.
///
/// This function recursively alternates with function `is_stmt_unit_type` to drill down
/// through all the blocks, loops and if-conditions to find the last executable statement
/// so as to determine if it returns a unit type or a value worth printing.
///
/// # Panics
/// Will panic if an unexpected expression type is found in the elso branch of an if-statement.
#[allow(clippy::too_many_lines)]
#[must_use]
#[inline]
pub fn is_last_stmt_unit_type<S: BuildHasher>(
    expr: &Expr,
    function_map: &HashMap<String, ReturnType, S>,
) -> bool {
    profile_fn!(is_last_stmt_unit_type);

    // debug_log!("%%%%%%%% expr={expr:#?}");
    match expr {
        Expr::ForLoop(for_loop) => {
            // debug_log!("%%%%%%%% Expr::ForLoop(for_loop))");
            for_loop.body.stmts.last().map_or(false, |last_stmt| {
                is_stmt_unit_type(last_stmt, function_map)
            })
        }
        Expr::If(expr_if) => {
            // Cycle through if-else statements and return false if any one is found returning
            // a non-unit value;
            if let Some(last_stmt_in_then_branch) = expr_if.then_branch.stmts.last() {
                // debug_log!("%%%%%%%% Some(last_stmt) = expr_if.then_branch.stmts.last()");
                if !is_stmt_unit_type(last_stmt_in_then_branch, function_map) {
                    return false;
                };
                expr_if.else_branch.as_ref().map_or(true, |stmt| {
                    let expr_else = &*stmt.1;
                    // The else branch expression may only be an If or Block expression,
                    // not any of the other types of expression.
                    match expr_else {
                        // If it's a block, we're at the end of the if-else chain and can just
                        // decide according to the return type of the last statement in the block.
                        Expr::Block(expr_block) => {
                            let else_is_unit_type =
                                expr_block.block.stmts.last().map_or(false, |last_stmt_in_block| is_stmt_unit_type(last_stmt_in_block, function_map));
                            else_is_unit_type
                        }
                        // If it's another if-statement, simply recurse through this method.
                        Expr::If(_) => is_last_stmt_unit_type(expr_else, function_map),
                        expr => {
                            eprintln!("Possible logic error: expected else branch expression to be If or Block, found {:?}", expr);
                            process::exit(1);
                        }
                    }
                })
            } else {
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
            expr_block.block.stmts.last().map_or(false, |last_stmt| {
                is_stmt_unit_type(last_stmt, function_map)
            })
        }
        Expr::Match(expr_match) => {
            for arm in &expr_match.arms {
                // debug_log!("arm.body={:#?}", arm.body);
                let expr = &*arm.body;
                if is_last_stmt_unit_type(expr, function_map) {
                    continue;
                }
                return false;
            }

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
        Expr::Closure(ref expr_closure) => match &expr_closure.output {
            ReturnType::Default => is_last_stmt_unit_type(&expr_closure.body, function_map),
            ReturnType::Type(_, ty) => {
                if let Tuple(tuple) = &**ty {
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
            AddAssign(_)
                | SubAssign(_)
                | MulAssign(_)
                | DivAssign(_)
                | RemAssign(_)
                | BitXorAssign(_)
                | BitAndAssign(_)
                | BitOrAssign(_)
                | ShlAssign(_)
                | ShrAssign(_)
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
        Expr::Macro(ref expr_macro) => {
            if let Some(segment) = expr_macro.mac.path.segments.last() {
                let ident = &segment.ident.to_string();
                return ident.starts_with("print")
                    || ident.starts_with("write")
                    || ident.starts_with("debug");
            }
            false // default - because no intrinsic way of knowing?
        }
        Expr::Path(ref path) => {
            if let Some(value) = is_path_unit_type(path, function_map) {
                return value;
            }
            false
        }
        Expr::Return(ref expr_return) => {
            // debug_log!("%%%%%%%% expr_return={expr_return:#?}");
            expr_return.expr.is_none()
        }
        _ => {
            cvprtln!(
                &Lvl::WARN,
                V::Q,
                "Expression not catered for: {expr:#?}, wrapping expression in println!()"
            );
            false
        }
    }
}

/// Check if a path represents a function, and if so, whether it has a unit or non-unit
/// return type.
#[must_use]
#[inline]
pub fn is_path_unit_type<S: BuildHasher>(
    path: &syn::PatPath,
    function_map: &HashMap<String, ReturnType, S>,
) -> Option<bool> {
    profile_fn!(is_path_unit_type);
    if let Some(ident) = path.path.get_ident() {
        if let Some(return_type) = function_map.get(&ident.to_string()) {
            return Some(match return_type {
                ReturnType::Default => {
                    // debug_log!("%%%%%%%% ReturnType::Default");
                    true
                }
                ReturnType::Type(_, ty) => {
                    if let Tuple(tuple) = &**ty {
                        // debug_log!("%%%%%%%% Tuple ReturnType");
                        tuple.elems.is_empty()
                    } else {
                        // debug_log!("%%%%%%%% Non-unit return type");
                        false
                    }
                }
            });
        }
    }
    None
}

/// Determine whether the return type of a given statement is unit (the empty tuple `()`).
///
/// Recursively alternates with function `is_last_stmt_unit` to drill down through all
/// the blocks, loops and if-conditions to identify the last executable statement so as to
/// determine if it returns a unit type or a value worth printing.
pub fn is_stmt_unit_type<S: BuildHasher>(
    stmt: &Stmt,
    function_map: &HashMap<String, ReturnType, S>,
) -> bool {
    profile_fn!(is_stmt_unit_type);

    debug_log!("%%%%%%%% stmt={stmt:#?}");
    match stmt {
        Stmt::Expr(expr, None) => {
            // if is_debug_logging_enabled() {
            //     debug_log!("%%%%%%%% expr={expr:#?}");
            //     debug_log!("%%%%%%%% Stmt::Expr(_, None)");
            // }
            is_last_stmt_unit_type(expr, function_map)
        } // Expression without semicolon
        Stmt::Expr(expr, Some(_)) => {
            // debug_log!("%%%%%%%% Stmt::Expr(_, Some(_))");
            match expr {
                Expr::Return(expr_return) => {
                    debug_log!("%%%%%%%% expr_return={expr_return:#?}");
                    expr_return.expr.is_none()
                }
                Expr::Yield(expr_yield) => {
                    debug_log!("%%%%%%%% expr_yield={expr_yield:#?}");
                    expr_yield.expr.is_none()
                }
                _ => true,
            }
        } // Expression with semicolon usually returns unit, except sometimes return or yield.
        Stmt::Macro(m) => {
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
                // debug_log!("%%%%%%%% Item::Macro({m:#?}), m.semi_token.is_some()={is_some}");
                m.semi_token.is_some()
            }
            _ => false, // default
        },
    }
}

/// # Errors
/// Will return `Err` if there is any error parsing expressions
pub fn is_main_fn_returning_unit(file: &File) -> ThagResult<bool> {
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

#[must_use]
pub fn get_source_path(build_state: &BuildState) -> String {
    profile_fn!(get_source_path);
    let binding: &PathBuf = if build_state.build_from_orig_source {
        &build_state.source_path
    } else {
        &build_state.target_dir_path.join(&build_state.source_name)
    };

    #[cfg(target_os = "windows")]
    let src_path = escape_path_for_windows(binding.to_string_lossy().as_ref());

    #[cfg(not(target_os = "windows"))]
    let src_path = binding.to_string_lossy().into_owned();
    src_path
}

/// Format a Rust source file in situ using rustfmt. For user diagnostic assistance only.
/// # Panics
/// Will panic if the `rustfmt` failed.
fn rustfmt(source_path_str: &str) {
    profile_fn!(rustfmt);
    if Command::new("rustfmt").arg("--version").output().is_ok() {
        // Run rustfmt on the source file
        let mut command = Command::new("rustfmt");
        command.arg("--edition");
        command.arg("2021");
        command.arg(source_path_str);
        command
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit());
        let output = command.output().expect("Failed to run rustfmt");

        if output.status.success() {
            debug_log!("Successfully formatted {} with rustfmt.", source_path_str);
            debug_log!(
                "{}\n{}",
                source_path_str,
                String::from_utf8_lossy(&output.stdout)
            );
        } else {
            debug_log!(
                "Failed to format {} with rustfmt\n{}",
                source_path_str,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    } else {
        vlog!(
            V::QQ,
            "`rustfmt` not found. Please install it to use this script."
        );
    }
}
