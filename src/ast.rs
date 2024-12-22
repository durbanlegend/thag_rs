use crate::{cprtln, debug_log, profile, profile_method, regex, BUILT_IN_CRATES};
use phf::phf_set;
use proc_macro2::TokenStream;
use quote::ToTokens;
use regex::Regex;
use std::collections::HashSet;
use std::ops::Deref;
use strum::Display;
use syn::{parse_file, visit::Visit, File, ItemMod, ItemUse, TypePath, UseRename, UseTree};

pub(crate) static FILTER_WORDS: phf::Set<&'static str> = phf_set! {
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
// #[cfg(any(feature = "ast", feature = "build"))]
pub enum Ast {
    File(syn::File),
    Expr(syn::Expr),
    // None,
}

// #[cfg(any(feature = "ast", feature = "build"))]
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
// #[cfg(any(feature = "ast", feature = "build"))]
impl ToTokens for Ast {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        profile_method!("to_tokens");
        match self {
            Self::File(file) => file.to_tokens(tokens),
            Self::Expr(expr) => expr.to_tokens(tokens),
        }
    }
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

/// Infer dependencies from AST-derived metadata to put in a Cargo.toml.
#[must_use]
#[allow(clippy::module_name_repetitions)]
pub fn infer_deps_from_ast(
    crates_finder: &CratesFinder,
    metadata_finder: &MetadataFinder,
) -> Vec<String> {
    profile!("infer_deps_from_ast");
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
    profile!("infer_deps_from_source");

    if code.trim().is_empty() {
        return vec![];
    }

    let maybe_ast = extract_and_wrap_uses(code);
    let mut dependencies = maybe_ast.map_or_else(
        |_| {
            cprtln!(
                &nu_ansi_term::Style::from(nu_ansi_term::Color::LightRed),
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

#[must_use]
pub fn should_filter_dependency(name: &str) -> bool {
    // Filter out capitalized names
    if name.chars().next().map_or(false, char::is_uppercase) {
        return true;
    }

    FILTER_WORDS.contains(name)
}

/// Identify mod statements for exclusion from Cargo.toml metadata.
/// Fallback version for when an abstract syntax tree cannot be parsed.
#[must_use]
pub fn find_modules_source(code: &str) -> Vec<String> {
    profile!("find_modules_source");
    let module_regex: &Regex = regex!(r"(?m)^[\s]*mod\s+([^;{\s]+)");
    debug_log!("In ast::find_use_renames_source");
    let mut modules: Vec<String> = vec![];
    for cap in module_regex.captures_iter(code) {
        let module = cap[1].to_string();
        debug_log!("module={module}");
        modules.push(module);
    }
    debug_log!("modules from source={modules:#?}");
    modules
}

/// Extract the `use` statements from source and parse them to a `syn::File` in order to
/// extract the dependencies..
///
/// # Errors
///
/// This function will return an error if `syn` fails to parse the `use` statements as a `syn::File`.
pub fn extract_and_wrap_uses(source: &str) -> Result<Ast, syn::Error> {
    profile!("extract_and_wrap_uses");
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
