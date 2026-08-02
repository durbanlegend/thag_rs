use proc_macro::TokenStream;
use quote::quote;
use std::{
    env, fs, io,
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

    let source_dir = Path::new(&thag_dev).join("locales");

    if !source_dir.is_dir() {
        return Err(format!(
            "Source locales directory does not exist: {}",
            source_dir.display()
        ));
    }

    let dest_dir = Path::new(&manifest_dir).join("locales");

    // Not if identical file
    if source_dir.canonicalize().ok() == dest_dir.canonicalize().ok() {
        #[cfg(debug_assertions)]
        eprintln!("Source and destination directories are one and the same");
        return Ok(());
    }

    #[cfg(debug_assertions)]
    eprintln!(
        "copy_i18n_locales: {} -> {}",
        source_dir.display(),
        dest_dir.display()
    );

    copy_dir_recursive(&source_dir, &dest_dir).map_err(|e| {
        format!(
            "Failed to copy {} -> {}: {e}",
            source_dir.display(),
            dest_dir.display()
        )
    })?;

    Ok(())
}

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
