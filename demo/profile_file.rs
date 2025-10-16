/// An early profiling prototype that tries to profile a file with macros via injection
/// into its `syn` abstract syntax tree. The drawback is that this technique discards
/// valuable information like comments and formatting.
///
/// Note that the injected profiling code is no longer valid. this is a demonstration only
///
/// E.g.: `thag demo/profile_file.rs < demo/hello_main.rs > $TMPDIR/hello_main_profiled.rs`
///
//# Purpose: Debugging
//# Categories: AST, crates, demo, learning, profiling, technique
use quote::quote;
use std::io::{self, Read};
use syn::{self, visit_mut::VisitMut, Block, ImplItemFn, ItemFn, ItemImpl, Stmt, Type};

fn read_stdin() -> Result<String, io::Error> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    handle.read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = read_stdin().expect("Problem reading input");
    // eprintln!("[{:#?}]", content);
    let mut ast: syn::File = syn::parse_str(&content)?;
    // println!("{:#?}", syntax);
    inject_profiling(&mut ast);
    let formatted = prettyplease::unparse(&ast).replace("\r\n", "\n"); // Normalize line endings
    println!("{}", formatted);

    Ok(())
}

struct ProfileInjector {
    current_impl_type: Option<String>,
}

impl ProfileInjector {
    fn new() -> Self {
        Self {
            current_impl_type: None,
        }
    }
}

impl VisitMut for ProfileInjector {
    fn visit_item_fn_mut(&mut self, func: &mut ItemFn) {
        if !has_existing_profile(&func.block) {
            let fn_name = func.sig.ident.to_string();

            // Special handling for main function to also inject enable_profiling
            if fn_name == "main" {
                inject_enable_profiling(&mut func.block);
            }

            inject_profile_stmt(&mut func.block, &fn_name, false);
        }

        // Continue visiting the function body for nested items
        syn::visit_mut::visit_item_fn_mut(self, func);
    }

    fn visit_impl_item_fn_mut(&mut self, method: &mut ImplItemFn) {
        if !has_existing_profile(&method.block) {
            let method_name = if let Some(ref impl_type) = self.current_impl_type {
                format!("{}::{}", impl_type, method.sig.ident)
            } else {
                method.sig.ident.to_string()
            };

            inject_profile_stmt(&mut method.block, &method_name, true);
        }

        syn::visit_mut::visit_impl_item_fn_mut(self, method);
    }

    fn visit_item_impl_mut(&mut self, impl_: &mut ItemImpl) {
        let prev_type = self.current_impl_type.take();
        self.current_impl_type = Some(get_type_name(&impl_.self_ty));

        // Visit the impl items
        syn::visit_mut::visit_item_impl_mut(self, impl_);

        self.current_impl_type = prev_type;
    }
}

fn add_profiling_imports(ast: &mut syn::File) {
    let import =
        syn::parse_str("use thag_rs::{profile, profile_method};").expect("Failed to parse import");

    ast.items.insert(0, import);
}

pub fn inject_profiling(ast: &mut syn::File) {
    add_profiling_imports(ast);
    let mut injector = ProfileInjector::new();
    injector.visit_file_mut(ast);
}

fn get_type_name(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) if !type_path.path.segments.is_empty() => type_path
            .path
            .segments
            .last()
            .map(|seg| seg.ident.to_string())
            .unwrap_or_default(),
        _ => String::new(),
    }
}

fn inject_profile_stmt(block: &mut Block, name: &str, is_method: bool) {
    let profile_macro = if is_method {
        quote! { profile_method!(#name); }
    } else {
        quote! { profile!(#name); }
    };

    let profile_stmt: Stmt = syn::parse2(profile_macro).unwrap();

    // Find position after declarations
    let insert_pos = block
        .stmts
        .iter()
        .position(|stmt| !is_declaration(stmt))
        .unwrap_or(0);

    block.stmts.insert(insert_pos, profile_stmt);
}

fn is_declaration(stmt: &Stmt) -> bool {
    matches!(
        stmt,
        Stmt::Local(_) // let statements
                       // Add other declaration types if needed
    )
}

fn has_existing_profile(block: &Block) -> bool {
    block.stmts.iter().any(|stmt| {
        if let Stmt::Macro(stmt_macro) = stmt {
            let macro_path = &stmt_macro.mac.path;
            macro_path.segments.last().map_or(false, |seg| {
                let name = seg.ident.to_string();
                name == "profile" || name == "profile_method"
            })
        } else {
            false
        }
    })
}

fn inject_enable_profiling(block: &mut Block) {
    let enable_stmt = syn::parse2(quote! {
        thag_rs::profiling::enable_profiling(true).expect("Failed to enable profiling");
    })
    .unwrap();

    block.stmts.insert(0, enable_stmt);
}
