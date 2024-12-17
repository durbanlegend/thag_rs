#![allow(clippy::uninlined_format_args)]
use crate::config::DependencyInference;
#[cfg(debug_assertions)]
use crate::debug_log;
use crate::{maybe_config, Dependencies};
use crate::{
    modified_since_compiled, vlog, DYNAMIC_SUBDIR, PACKAGE_NAME, REPL_SUBDIR, RS_SUFFIX,
    TEMP_DIR_NAME, TEMP_SCRIPT_NAME, TMPDIR, TOML_NAME, V,
};
use crate::{Cli, ProcFlags};
use crate::{ThagError, ThagResult};
use cargo_toml::Manifest;
use crossterm::event::Event;
use home::home_dir;
use mockall::automock;
use phf::phf_set;
use proc_macro2::TokenStream;
use quote::ToTokens;
use std::clone::Clone;
use std::{
    convert::Into,
    option::Option,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use strum::Display;
use syn::ItemUse;
use syn::{self, visit::Visit, ItemMod, TypePath, UseRename, UseTree};
use thag_core::{profile, profile_method};

static FILTER_WORDS: phf::Set<&'static str> = phf_set! {
    // Numeric primitives
    "f32", "f64",
    "i8", "i16", "i32", "i64", "i128", "isize",
    "u8", "u16", "u32", "u64", "u128", "usize",

    // Core types
    "bool", "str",

    // Common std modules that might appear in paths
    "error", "fs",

    // Rust keywords that might appear in paths
    "self", "super", "crate"
};

/// An abstract syntax tree wrapper for use with syn.
#[derive(Clone, Debug, Display)]
pub enum Ast {
    File(syn::File),
    Expr(syn::Expr),
    // None,
}

impl Ast {
    #[must_use]
    pub const fn is_file(&self) -> bool {
        match self {
            Self::File(_) => true,
            Self::Expr(_) => false,
        }
    }
}

/// Required to use quote! macro to generate code to resolve expression.
impl ToTokens for Ast {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        profile_method!("to_tokens");
        match self {
            Self::File(file) => file.to_tokens(tokens),
            Self::Expr(expr) => expr.to_tokens(tokens),
        }
    }
}

struct ExecutionFlags {
    is_repl: bool,
    is_dynamic: bool,
}

impl ExecutionFlags {
    const fn new(proc_flags: &ProcFlags, cli: &Cli) -> Self {
        let is_repl = proc_flags.contains(ProcFlags::REPL);
        let is_expr = cli.expression.is_some();
        let is_stdin = proc_flags.contains(ProcFlags::STDIN);
        let is_edit = proc_flags.contains(ProcFlags::EDIT);
        // let is_url = proc_flags.contains(ProcFlags::URL); // TODO reinstate
        let is_loop = proc_flags.contains(ProcFlags::LOOP);
        let is_dynamic = is_expr | is_stdin | is_edit | is_loop;

        Self {
            is_repl,
            is_dynamic,
        }
    }
}

struct BuildPaths {
    working_dir_path: PathBuf,
    source_path: PathBuf,
    source_dir_path: PathBuf,
    cargo_home: PathBuf,
    target_dir_path: PathBuf,
    target_path: PathBuf,
    cargo_toml_path: PathBuf,
}

#[derive(Clone, Debug, Default)]
pub struct CratesFinder {
    pub crates: Vec<String>,
    pub names_to_exclude: Vec<String>,
}

impl<'a> Visit<'a> for CratesFinder {
    fn visit_item_use(&mut self, node: &'a ItemUse) {
        profile_method!("visit_item_use");
        // Handle simple case `use a as b;`
        if let UseTree::Rename(use_rename) = &node.tree {
            let node_name = use_rename.ident.to_string();
            // debug_log!("item_use pushing {node_name} to crates");
            self.crates.push(node_name);
        } else {
            syn::visit::visit_item_use(self, node);
        }
    }

    fn visit_use_tree(&mut self, node: &'a UseTree) {
        profile_method!("visit_use_tree");
        match node {
            UseTree::Group(_) => {
                syn::visit::visit_use_tree(self, node);
            }
            UseTree::Path(p) => {
                let node_name = p.ident.to_string();
                if !should_filter_dependency(&node_name) && !self.crates.contains(&node_name) {
                    // debug_log!("use_tree pushing path name {node_name} to crates");
                    self.crates.push(node_name.clone());
                }
                let use_tree = &*p.tree;
                match use_tree {
                    UseTree::Path(child) => {
                        // if we have `use a::b::c;`, we want a to be recognised as
                        // a crate while b and c are excluded, This takes care of b
                        // when the parent node is a.
                        let child_name = child.ident.to_string();
                        if child_name != node_name  // e.g. the second quote in quote::quote
                            && !self.names_to_exclude.contains(&child_name)
                        {
                            // debug_log!(
                            //     "visit_use_tree pushing mid name {child_name} to names_to_exclude",
                            // );
                            self.names_to_exclude.push(child_name);
                        }
                    }
                    UseTree::Name(child) => {
                        // if we have `use a::b::c;`, we want a to be recognised as
                        // a crate while b and c are excluded, This takes care of c
                        // when the parent node is b.
                        let child_name = child.ident.to_string();
                        if child_name != node_name  // e.g. the second quote in quote::quote
                            && !self.names_to_exclude.contains(&child_name)
                        {
                            self.names_to_exclude.push(child_name);
                        }
                    }
                    UseTree::Group(group) => {
                        for child in &group.items {
                            // if we have `use a::{b, c};`, we want a to be recognised as
                            // a crate while b and c are excluded, This takes care of b and c
                            // when the parent node is a.
                            match child {
                                UseTree::Path(child) => {
                                    // if we have `use a::b::c;`, we want a to be recognised as
                                    // a crate while b and c are excluded, This takes care of b
                                    // when the parent node is a.
                                    let child_name = child.ident.to_string();
                                    if child_name != node_name  // e.g. the second quote in quote::quote
                                        && !self.names_to_exclude.contains(&child_name)
                                    {
                                        self.names_to_exclude.push(child_name);
                                    }
                                }
                                UseTree::Name(child) => {
                                    // if we have `use a::b::c;`, we want a to be recognised as
                                    // a crate while b and c are excluded, This takes care of c
                                    // when the parent node is b.
                                    let child_name = child.ident.to_string();
                                    if child_name != node_name  // e.g. the second quote in quote::quote
                                        && !self.names_to_exclude.contains(&child_name)
                                    {
                                        // debug_log!("visit_use_tree pushing grpend name {child_name} to names_to_exclude");
                                        self.names_to_exclude.push(child_name);
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                    _ => (),
                }
                syn::visit::visit_use_tree(self, node);
            }
            UseTree::Name(n) => {
                let node_name = n.ident.to_string();
                if !self.crates.contains(&node_name) {
                    // debug_log!("visit_use_tree pushing end name {node_name} to crates (2)");
                    self.crates.push(node_name);
                }
            }
            _ => (),
        }
    }

    fn visit_expr_path(&mut self, expr_path: &'a syn::ExprPath) {
        profile_method!("visit_expr_path");
        if expr_path.path.segments.len() > 1 {
            // must have the form a::b so not a variable
            if let Some(first_seg) = expr_path.path.segments.first() {
                let name = first_seg.ident.to_string();
                #[cfg(debug_assertions)]
                debug_log!("Found first seg {name} in expr_path={expr_path:#?}");
                if !should_filter_dependency(&name) && !self.crates.contains(&name) {
                    // debug_log!("visit_expr_path pushing {name} to crates");
                    self.crates.push(name);
                }
            }
        }
        syn::visit::visit_expr_path(self, expr_path);
    }

    fn visit_type_path(&mut self, type_path: &'a TypePath) {
        profile_method!("visit_type_path");
        if type_path.path.segments.len() > 1 {
            if let Some(first_seg) = type_path.path.segments.first() {
                let name = first_seg.ident.to_string();
                // #[cfg(debug_assertions)]
                // debug_log!("Found first seg {name} in type_path={type_path:#?}");
                if !should_filter_dependency(&name) && !self.crates.contains(&name) {
                    // #[cfg(debug_assertions)]
                    // debug_log!("visit_type_path pushing {name} to crates");
                    self.crates.push(name);
                }
            }
        }
        syn::visit::visit_type_path(self, type_path);
    }

    // Handle macro invocations
    fn visit_macro(&mut self, mac: &'a syn::Macro) {
        profile_method!("visit_macro");
        // Get the macro path (e.g., "serde_json::json" from "serde_json::json!()")
        if mac.path.segments.len() > 1 {
            if let Some(first_seg) = mac.path.segments.first() {
                let name = first_seg.ident.to_string();
                if !should_filter_dependency(&name) && !self.crates.contains(&name) {
                    // debug_log!("visit_macro pushing {name} to crates");
                    self.crates.push(name);
                }
            }
        }
        syn::visit::visit_macro(self, mac);
    }

    // Handle trait implementations
    fn visit_item_impl(&mut self, item: &'a syn::ItemImpl) {
        profile_method!("visit_item_impl");
        // Check the trait being implemented (if any)
        if let Some((_, path, _)) = &item.trait_ {
            if let Some(first_seg) = path.segments.first() {
                let name = first_seg.ident.to_string();
                if !should_filter_dependency(&name) && !self.crates.contains(&name) {
                    // debug_log!("visit_item_impl pushing {name} to crates (1)");
                    self.crates.push(name);
                }
            }
        }

        // Check the type being implemented for
        if let syn::Type::Path(type_path) = &*item.self_ty {
            if let Some(first_seg) = type_path.path.segments.first() {
                let name = first_seg.ident.to_string();
                if !should_filter_dependency(&name) && !self.crates.contains(&name) {
                    // debug_log!("visit_item_impl pushing {name} to crates (2)");
                    self.crates.push(name);
                }
            }
        }
        syn::visit::visit_item_impl(self, item);
    }

    // Handle associated types
    fn visit_item_type(&mut self, item: &'a syn::ItemType) {
        profile_method!("visit_item_type");
        if let syn::Type::Path(type_path) = &*item.ty {
            if let Some(first_seg) = type_path.path.segments.first() {
                let name = first_seg.ident.to_string();
                if !should_filter_dependency(&name) && !self.crates.contains(&name) {
                    // debug_log!("visit_item_type pushing {name} to crates (2)");
                    self.crates.push(name);
                }
            }
        }
        syn::visit::visit_item_type(self, item);
    }

    // Handle generic bounds
    fn visit_type_param_bound(&mut self, bound: &'a syn::TypeParamBound) {
        profile_method!("visit_type_param_bound");
        if let syn::TypeParamBound::Trait(trait_bound) = bound {
            if let Some(first_seg) = trait_bound.path.segments.first() {
                let name = first_seg.ident.to_string();
                if !should_filter_dependency(&name) && !self.crates.contains(&name) {
                    // debug_log!("visit_type_param_bound pushing first {name} to crates");
                    self.crates.push(name);
                }
            }
        }
        syn::visit::visit_type_param_bound(self, bound);
    }
}

#[derive(Clone, Debug, Default)]
pub struct MetadataFinder {
    pub extern_crates: Vec<String>,
    pub mods_to_exclude: Vec<String>,
    pub names_to_exclude: Vec<String>,
    pub main_count: usize,
}

impl<'a> Visit<'a> for MetadataFinder {
    fn visit_use_rename(&mut self, node: &'a UseRename) {
        profile_method!("visit_use_rename");
        // eprintln!(
        //     "visit_use_rename pushing {} to names_to_exclude",
        //     node.rename
        // );
        self.names_to_exclude.push(node.rename.to_string());
        syn::visit::visit_use_rename(self, node);
    }

    fn visit_item_extern_crate(&mut self, node: &'a syn::ItemExternCrate) {
        profile_method!("visit_item_extern_crate");
        let crate_name = node.ident.to_string();
        self.extern_crates.push(crate_name);
        syn::visit::visit_item_extern_crate(self, node);
    }

    fn visit_item_mod(&mut self, node: &'a ItemMod) {
        profile_method!("visit_item_mod");
        self.mods_to_exclude.push(node.ident.to_string());
        syn::visit::visit_item_mod(self, node);
    }

    fn visit_item_fn(&mut self, node: &'a syn::ItemFn) {
        profile_method!("visit_item_fn");
        if node.sig.ident == "main" {
            self.main_count += 1; // Increment counter instead of setting bool
        }
        syn::visit::visit_item_fn(self, node);
    }
}

#[must_use]
pub fn should_filter_dependency(name: &str) -> bool {
    // Filter out capitalized names
    if name.chars().next().map_or(false, char::is_uppercase) {
        return true;
    }

    FILTER_WORDS.contains(name)
}

#[must_use]
pub fn find_crates(syntax_tree: &Ast) -> CratesFinder {
    profile!("find_crates");
    let mut crates_finder = CratesFinder::default();

    match syntax_tree {
        Ast::File(ast) => crates_finder.visit_file(ast),
        Ast::Expr(ast) => crates_finder.visit_expr(ast),
    }

    crates_finder
}

#[must_use]
pub fn find_metadata(syntax_tree: &Ast) -> MetadataFinder {
    profile!("find_metadata");
    let mut metadata_finder = MetadataFinder::default();

    match syntax_tree {
        Ast::File(ast) => metadata_finder.visit_file(ast),
        Ast::Expr(ast) => metadata_finder.visit_expr(ast),
    }

    metadata_finder
}

/// A struct to encapsulate the attributes of the current build as needed by the various
/// functions co-operating in the generation, build and execution of the code.
#[derive(Clone, Debug, Default)]
pub struct BuildState {
    #[allow(dead_code)]
    pub working_dir_path: PathBuf,
    pub source_stem: String,
    pub source_name: String,
    #[allow(dead_code)]
    pub source_dir_path: PathBuf,
    pub source_path: PathBuf,
    pub cargo_home: PathBuf,
    pub target_dir_path: PathBuf,
    pub target_path: PathBuf,
    pub cargo_toml_path: PathBuf,
    pub rs_manifest: Option<Manifest>,
    pub cargo_manifest: Option<Manifest>,
    pub must_gen: bool,
    pub must_build: bool,
    pub build_from_orig_source: bool,
    pub ast: Option<Ast>,
    pub crates_finder: Option<CratesFinder>,
    pub metadata_finder: Option<MetadataFinder>,
    pub infer: DependencyInference,
    pub args: Vec<String>,
}

impl BuildState {
    /// Configures a new `BuildState` instance based on processing flags, CLI arguments, and script state.
    ///
    /// This function coordinates the complete setup process by:
    /// 1. Extracting and validating script information
    /// 2. Determining execution mode flags
    /// 3. Setting up all required directory paths
    /// 4. Creating the initial build state
    /// 5. Determining build requirements
    ///
    /// # Arguments
    /// * `proc_flags` - Processing flags that control build and execution behavior
    /// * `cli` - Command-line arguments parsed from the CLI
    /// * `script_state` - Current state of the script being processed
    ///
    /// # Returns
    /// * `ThagResult<Self>` - Configured `BuildState` instance if successful
    ///
    /// # Errors
    /// Returns a `ThagError` if:
    /// * No script is specified in the script state
    /// * Script filename is invalid or cannot be converted to a string
    /// * Unable to strip .rs suffix from script name
    /// * Cannot resolve working directory or home directory
    /// * Cannot resolve script directory path
    /// * Script file does not exist at the specified path
    /// * Cannot resolve parent directory of script
    /// * Cannot determine if source has been modified since last compilation
    ///
    /// # Example
    /// ```ignore
    /// let proc_flags = ProcFlags::default();
    /// let cli = Cli::parse();
    /// let script_state = ScriptState::new("example.rs");
    /// let build_state = BuildState::pre_configure(&proc_flags, &cli, &script_state)?;
    /// ```
    pub fn pre_configure(
        proc_flags: &ProcFlags,
        cli: &Cli,
        script_state: &ScriptState,
    ) -> ThagResult<Self> {
        profile_method!("pre_configure");

        // 1. Validate and extract basic script info
        let (source_name, source_stem) = Self::extract_script_info(script_state)?;

        // 2. Determine execution mode flags
        let execution_flags = ExecutionFlags::new(proc_flags, cli);

        // 3. Set up directory paths
        let paths = Self::set_up_paths(&execution_flags, script_state, &source_name, &source_stem)?;

        // 4. Create initial build state
        let mut build_state = Self::create_initial_state(paths, source_name, source_stem, cli);

        // 5. Determine build requirements
        build_state.determine_build_requirements(proc_flags, script_state, &execution_flags)?;

        // 6. Validate state (debug only)
        #[cfg(debug_assertions)]
        build_state.validate_state(proc_flags);

        Ok(build_state)
    }

    fn extract_script_info(script_state: &ScriptState) -> ThagResult<(String, String)> {
        profile!("extract_script_info");
        let script = script_state
            .get_script()
            .ok_or(ThagError::NoneOption("No script specified"))?;

        let path = Path::new(&script);
        let filename = path
            .file_name()
            .ok_or(ThagError::NoneOption("No filename specified"))?;

        let source_name = filename
            .to_str()
            .ok_or(ThagError::NoneOption(
                "Error converting filename to a string",
            ))?
            .to_string();

        let source_stem = source_name
            .strip_suffix(RS_SUFFIX)
            .ok_or_else(|| -> ThagError {
                format!("Error stripping suffix from {source_name}").into()
            })?
            .to_string();

        Ok((source_name, source_stem))
    }

    fn set_up_paths(
        flags: &ExecutionFlags,
        script_state: &ScriptState,
        source_name: &str,
        source_stem: &str,
    ) -> ThagResult<BuildPaths> {
        profile!("set_up_paths");
        // Working directory setup
        let working_dir_path = if flags.is_repl {
            TMPDIR.join(REPL_SUBDIR)
        } else {
            std::env::current_dir()?.canonicalize()?
        };

        // Script path setup
        let script_path = if flags.is_repl {
            script_state
                .get_script_dir_path()
                .ok_or("Missing script path")?
                .join(source_name)
        } else if flags.is_dynamic {
            script_state
                .get_script_dir_path()
                .ok_or("Missing script path")?
                .join(TEMP_SCRIPT_NAME)
        } else {
            working_dir_path.join(script_state.get_script().unwrap()) // Safe due to prior validation
        };

        // Source path setup and validation
        let source_path = script_path.canonicalize()?;
        if !source_path.exists() {
            return Err(format!(
                "No script named {source_stem} or {source_name} in path {source_path:?}"
            )
            .into());
        }

        // Source directory path
        let source_dir_path = source_path
            .parent()
            .ok_or("Problem resolving to parent directory")?
            .to_path_buf();

        // Cargo home setup
        let cargo_home = PathBuf::from(match std::env::var("CARGO_HOME") {
            Ok(string) if string != String::new() => string,
            _ => {
                let home_dir = home_dir().ok_or("Can't resolve home directory")?;
                home_dir.join(".cargo").display().to_string()
            }
        });

        // Target directory setup
        let target_dir_path = if flags.is_repl {
            script_state
                .get_script_dir_path()
                .ok_or("Missing ScriptState::NamedEmpty.repl_path")?
                .join(TEMP_DIR_NAME)
        } else if flags.is_dynamic {
            TMPDIR.join(DYNAMIC_SUBDIR)
        } else {
            TMPDIR.join(PACKAGE_NAME).join(source_stem)
        };

        // Target path setup
        let mut target_path = target_dir_path.join("target").join("debug");
        #[cfg(target_os = "windows")]
        {
            target_path = target_path.join(format!("{source_stem}.exe"));
        }
        #[cfg(not(target_os = "windows"))]
        {
            target_path = target_path.join(source_stem);
        }

        let cargo_toml_path = target_dir_path.join(TOML_NAME);

        Ok(BuildPaths {
            working_dir_path,
            source_path,
            source_dir_path,
            cargo_home,
            target_dir_path,
            target_path,
            cargo_toml_path,
        })
    }

    fn create_initial_state(
        paths: BuildPaths,
        source_name: String,
        source_stem: String,
        cli: &Cli,
    ) -> Self {
        profile!("create_initial_state");

        Self {
            working_dir_path: paths.working_dir_path,
            source_stem,
            source_name,
            source_dir_path: paths.source_dir_path,
            source_path: paths.source_path,
            cargo_home: paths.cargo_home,
            target_dir_path: paths.target_dir_path,
            target_path: paths.target_path,
            cargo_toml_path: paths.cargo_toml_path,
            ast: None,
            crates_finder: None,
            metadata_finder: None,
            infer: cli.infer.as_ref().map_or_else(
                || {
                    let config = maybe_config();
                    let binding = Dependencies::default();
                    let dep_config = config.as_ref().map_or(&binding, |c| &c.dependencies);
                    let infer = &dep_config.inference_level;
                    infer.clone()
                },
                Clone::clone,
            ),
            args: cli.args.clone(),
            ..Default::default()
        }
    }

    fn determine_build_requirements(
        &mut self,
        proc_flags: &ProcFlags,
        script_state: &ScriptState,
        flags: &ExecutionFlags,
    ) -> ThagResult<()> {
        profile_method!("determine_build_requirements");
        // Case 1: Force generation and building
        if flags.is_dynamic
            || flags.is_repl
            || proc_flags.contains(ProcFlags::FORCE)
            || proc_flags.contains(ProcFlags::CHECK)
        {
            self.must_gen = true;
            self.must_build = true;
            return Ok(());
        }

        // Case 2: No-run mode
        if proc_flags.contains(ProcFlags::NORUN) {
            self.must_build = proc_flags.contains(ProcFlags::BUILD)
                || proc_flags.contains(ProcFlags::EXECUTABLE)
                // For EXPAND and CARGO, "build" step (becoming a bit of a misnomer)
                // is needed to run their alternative Cargo commands
                || proc_flags.contains(ProcFlags::EXPAND)
                || proc_flags.contains(ProcFlags::CARGO);
            self.must_gen = self.must_build
                || proc_flags.contains(ProcFlags::GENERATE)
                || !self.cargo_toml_path.exists();
            return Ok(());
        }

        // Case 3: Check if build is needed due to state or modifications
        if matches!(script_state, ScriptState::NamedEmpty { .. })
            || !self.target_path.exists()
            || modified_since_compiled(self)?.is_some()
        {
            self.must_gen = true;
            self.must_build = true;
            return Ok(());
        }

        // Default case: no generation or building needed
        self.must_gen = false;
        self.must_build = false;
        Ok(())
    }

    #[cfg(debug_assertions)]
    fn validate_state(&self, proc_flags: &ProcFlags) {
        profile_method!("validate_state");
        // Validate build/check/executable/expand flags
        if proc_flags.contains(ProcFlags::BUILD)
            | proc_flags.contains(ProcFlags::CHECK)
            | proc_flags.contains(ProcFlags::EXECUTABLE)
            | proc_flags.contains(ProcFlags::EXPAND)
            | proc_flags.contains(ProcFlags::CARGO)
        {
            assert!(self.must_gen & self.must_build & proc_flags.contains(ProcFlags::NORUN));
        }

        // Validate force flag
        if proc_flags.contains(ProcFlags::FORCE) {
            assert!(self.must_gen & self.must_build);
        }

        // Validate expand and cargo flags
        if proc_flags.contains(ProcFlags::EXPAND) | proc_flags.contains(ProcFlags::CARGO) {
            assert!(self.must_gen & self.must_build & proc_flags.contains(ProcFlags::NORUN));
        }

        // Validate build dependency
        if self.must_build {
            assert!(self.must_gen);
        }

        // Log the final state in debug mode
        debug_log!("build_state={self:#?}");
    }
}

/// An enum to encapsulate the type of script in play.
#[derive(Debug)]
pub enum ScriptState {
    /// Repl with no script name provided by user
    #[allow(dead_code)]
    Anonymous,
    /// Repl with script name.
    NamedEmpty {
        script: String,
        script_dir_path: PathBuf,
    },
    /// Script name provided by user
    Named {
        script: String,
        script_dir_path: PathBuf,
    },
}

impl ScriptState {
    /// Return the script name wrapped in an Option.
    #[must_use]
    pub fn get_script(&self) -> Option<String> {
        profile_method!("get_script");
        match self {
            Self::Anonymous => None,
            Self::NamedEmpty { script, .. } | Self::Named { script, .. } => {
                Some(script.to_string())
            }
        }
    }
    /// Return the script's directory path wrapped in an Option.
    #[must_use]
    pub fn get_script_dir_path(&self) -> Option<PathBuf> {
        profile_method!("get_script_dir_path");
        match self {
            Self::Anonymous => None,
            Self::Named {
                script_dir_path, ..
            } => Some(script_dir_path.clone()),
            Self::NamedEmpty {
                script_dir_path: script_path,
                ..
            } => Some(script_path.clone()),
        }
    }
}

/// Developer method to log method timings.
#[inline]
#[cfg(debug_assertions)]
pub fn debug_timings(start: &Instant, process: &str) {
    profile!("debug_timings");
    let dur = start.elapsed();
    debug_log!("{} in {}.{}s", process, dur.as_secs(), dur.subsec_millis());
}

#[inline]
/// Display method timings when either the --verbose or --timings option is chosen.
pub fn display_timings(start: &Instant, process: &str, proc_flags: &ProcFlags) {
    profile!("display_timings");
    #[cfg(not(debug_assertions))]
    if !proc_flags.intersects(ProcFlags::DEBUG | ProcFlags::VERBOSE | ProcFlags::TIMINGS) {
        return;
    }
    let dur = start.elapsed();
    let msg = format!("{process} in {}.{}s", dur.as_secs(), dur.subsec_millis());

    #[cfg(debug_assertions)]
    debug_log!("{msg}");
    if proc_flags.intersects(ProcFlags::DEBUG | ProcFlags::VERBOSE | ProcFlags::TIMINGS) {
        vlog!(V::QQ, "{msg}");
    }
}

// Helper function to sort out the issues caused by Windows using the escape character as
// the file separator.
#[must_use]
#[inline]
#[cfg(target_os = "windows")]
pub fn escape_path_for_windows(path_str: &str) -> String {
    profile!("escape_path_for_windows");
    path_str.replace('\\', "/")
}

#[must_use]
#[cfg(not(target_os = "windows"))]
pub fn escape_path_for_windows(path_str: &str) -> String {
    profile!("escape_path_for_windows");
    path_str.to_string()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyDisplayLine {
    pub seq: usize,
    pub keys: &'static str, // Or String if you plan to modify the keys later
    pub desc: &'static str, // Or String for modifiability
}

impl PartialOrd for KeyDisplayLine {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // profile_method!("partial_cmp");
        Some(self.cmp(other))
    }
}

impl Ord for KeyDisplayLine {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // profile_method!("cmp");
        usize::cmp(&self.seq, &other.seq)
    }
}

impl KeyDisplayLine {
    #[must_use]
    pub const fn new(seq: usize, keys: &'static str, desc: &'static str) -> Self {
        Self { seq, keys, desc }
    }
}

/// A trait to allow mocking of the event reader for testing purposes.
#[automock]
pub trait EventReader {
    /// Read a terminal event.
    ///
    /// # Errors
    ///
    /// This function will bubble up any i/o, `ratatui` or `crossterm` errors encountered.
    fn read_event(&self) -> ThagResult<Event>;
    /// Poll for a terminal event.
    ///
    /// # Errors
    ///
    /// This function will bubble up any i/o, `ratatui` or `crossterm` errors encountered.
    fn poll(&self, timeout: Duration) -> ThagResult<bool>;
}

/// A struct to implement real-world use of the event reader, as opposed to use in testing.
#[derive(Debug)]
pub struct CrosstermEventReader;

impl EventReader for CrosstermEventReader {
    fn read_event(&self) -> ThagResult<Event> {
        profile_method!("read_event");
        crossterm::event::read().map_err(Into::<ThagError>::into)
    }

    fn poll(&self, timeout: Duration) -> ThagResult<bool> {
        profile_method!("poll");
        crossterm::event::poll(timeout).map_err(Into::<ThagError>::into)
    }
}

/// Control debug logging
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        // If the `debug-logs` feature is enabled, always log
        #[cfg(any(feature = "debug-logs", feature = "simplelog"))]
        {
            log::debug!($($arg)*);
        }

        // In all builds, log if runtime debug logging is enabled (e.g., via `-vv`)
        #[cfg(not(any(feature = "debug-logs", feature = "simplelog")))]
        {
            if $crate::logging::is_debug_logging_enabled() {
                log::debug!($($arg)*);
            } else {
                // Avoid unused variable warnings in release mode if logging isn't enabled
                let _ = format_args!($($arg)*);
            }
        }
    };
}

#[macro_export]
macro_rules! lazy_static_var {
    ($type:ty, $init_fn:expr, deref) => {{
        use std::sync::OnceLock;
        static GENERIC_LAZY: OnceLock<$type> = OnceLock::new();
        *GENERIC_LAZY.get_or_init(|| $init_fn)
    }};
    ($type:ty, $init_fn:expr) => {{
        use std::sync::OnceLock;
        static GENERIC_LAZY: OnceLock<$type> = OnceLock::new();
        GENERIC_LAZY.get_or_init(|| $init_fn)
    }};
}
