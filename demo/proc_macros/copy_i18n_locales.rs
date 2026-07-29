use proc_macro::TokenStream;
use quote::quote;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

pub fn copy_i18n_locales_impl(_input: TokenStream) -> TokenStream {
    if let Err(err) = do_copy() {
        return syn::Error::new(proc_macro2::Span::call_site(), err)
            .to_compile_error()
            .into();
    }

    TokenStream::from(quote! {})
}

fn do_copy() -> Result<(), String> {
    let thag_dev = env::var("THAG_DEV_PATH")
        .map_err(|_| "Environment variable THAG_DEV_PATH is not set".to_string())?;

    let manifest_dir =
        env::var("CARGO_MANIFEST_DIR").map_err(|_| "CARGO_MANIFEST_DIR is not set".to_string())?;

    if manifest_dir.eq(&thag_dev) {
        return Ok(());
    }

    let source = Path::new(&thag_dev).join("locales").join("app.yaml");

    if !source.exists() {
        return Err(format!(
            "Source locale file does not exist: {}",
            source.display()
        ));
    }

    let dest_dir = Path::new(&manifest_dir).join("locales");

    fs::create_dir_all(&dest_dir)
        .map_err(|e| format!("Failed to create {}: {e}", dest_dir.display()))?;

    let dest = dest_dir.join("app.yaml");

    // Optional optimisation: don't rewrite an identical file.
    let should_copy = match (fs::read(&source), fs::read(&dest)) {
        (Ok(src), Ok(dst)) => src != dst,
        _ => true,
    };

    if should_copy {
        // Not if identical file
        if source.canonicalize().ok() != dest.canonicalize().ok() {
            #[cfg(debug_assertions)]
            eprintln!(
                "copy_i18n_locales: {} -> {}",
                source.display(),
                dest.display()
            );
            fs::copy(&source, &dest).map_err(|e| {
                format!(
                    "Failed to copy {} -> {}: {e}",
                    source.display(),
                    dest.display()
                )
            })?;
        }
    }

    Ok(())
}

use std::{fs, io, path::Path};

fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;

        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else if file_type.is_file() {
            // Don't rewrite identical files.
            let should_copy = match (fs::read(&src_path), fs::read(&dst_path)) {
                (Ok(src), Ok(dst)) => src != dst,
                _ => true,
            };

            if should_copy {
                fs::copy(&src_path, &dst_path)?;
            }
        }
    }

    Ok(())
}
