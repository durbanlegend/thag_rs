use proc_macro::TokenStream;
use std::{env, fs, io, path::Path};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    LitStr, Token,
};

struct CopyArgs {
    env_var: LitStr,
    source_subdir: LitStr,
    dest_subdir: Option<LitStr>,
}

impl Parse for CopyArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let args = Punctuated::<LitStr, Token![,]>::parse_terminated(input)?;

        if !(2..=3).contains(&args.len()) {
            return Err(
                input.error("expected: copy_resource_dir!(\"ENV_VAR\", \"source\", \"dest\")")
            );
        }

        let mut args = args.into_iter();

        let env_var = args.next().unwrap();
        let source_subdir = args.next().unwrap();
        let dest_subdir = Some(args.next().unwrap_or_else(|| source_subdir.clone()));

        Ok(Self {
            env_var,
            source_subdir,
            dest_subdir,
        })
    }
}

pub fn copy_resource_dir_impl(input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(input as CopyArgs);

    let dest_subdir = args
        .dest_subdir
        .as_ref()
        .map(LitStr::value)
        .unwrap_or_else(|| args.source_subdir.value());

    if let Err(err) = do_copy(
        &args.env_var.value(),
        &args.source_subdir.value(),
        &dest_subdir,
    ) {
        return syn::Error::new(args.env_var.span(), err)
            .to_compile_error()
            .into();
    }

    quote::quote!().into()
}

fn do_copy(env_var: &str, source_subdir: &str, dest_subdir: &str) -> Result<(), String> {
    let source_root =
        env::var(env_var).map_err(|_| format!("Environment variable {env_var} is not set"))?;

    let source_dir = Path::new(&source_root).join(source_subdir);

    if !source_dir.is_dir() {
        return Err(format!(
            "Source directory does not exist: {}",
            source_dir.display()
        ));
    }

    let manifest_dir =
        env::var("CARGO_MANIFEST_DIR").map_err(|_| "CARGO_MANIFEST_DIR is not set".to_string())?;

    let dest_dir = Path::new(&manifest_dir).join(dest_subdir);

    // Not if identical file
    if source_dir.canonicalize().ok() == dest_dir.canonicalize().ok() {
        #[cfg(debug_assertions)]
        eprintln!("Source and destination directories are one and the same");
        return Ok(());
    }

    #[cfg(debug_assertions)]
    eprintln!(
        "copy_resource_dir: {} -> {}",
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
                #[cfg(debug_assertions)]
                eprintln!("copying: {} -> {}", src_path.display(), dst_path.display());

                fs::copy(&src_path, &dst_path)?;
            }
        }
    }

    Ok(())
}
