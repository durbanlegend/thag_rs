use crate::cmd_args::ProcFlags;
use crate::errors::BuildRunError;
use crate::toml_utils::CargoManifest;
use crate::{cmd_args, toml_utils::rs_extract_manifest};
use log::debug;
use regex::Regex;
use std::time::Instant;
use std::{
    collections::HashSet,
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

#[allow(dead_code, clippy::uninlined_format_args)]
fn main() {
    let code_snippet = r#"
  use std::io;

  #[macro_use]
  extern crate serde_derive;

  fn main() {
    println!("Hello, world!");
  }
  "#;

    let dependencies = infer_dependencies(code_snippet);
    println!("Potential dependencies: {dependencies:?}");
}
pub(crate) fn read_file_contents(path: &Path) -> Result<String, BuildRunError> {
    debug!("Reading from {path:?}");
    Ok(fs::read_to_string(path)?)
}

pub(crate) fn rs_extract_src(rs_contents: &str) -> String {
    use std::fmt::Write;
    let rs_source = rs_contents
        .lines()
        .map(str::trim_start)
        .filter(|&line| !line.starts_with("//!"))
        .fold(String::new(), |mut output, b| {
            let _ = writeln!(output, "{b}");
            output
        });
    debug!("Rust source string (rs_source) = {rs_source}");
    rs_source
}

// Make a best effort to help the user by inferring dependencies from the source code.
pub(crate) fn infer_dependencies(code: &str) -> HashSet<String> {
    let mut dependencies = HashSet::new();

    let use_regex = Regex::new(r"(?i)use\s+([^;{]+)").unwrap();
    let macro_use_regex = Regex::new(r"(?i)#\[macro_use\]\s+::\s+([^;{]+)").unwrap();
    let extern_crate_regex = Regex::new(r"(?i)extern\s+crate\s+([^;{]+)").unwrap();

    let built_in_crates = &["std", "core", "alloc", "collections", "fmt"];

    for cap in use_regex.captures_iter(code) {
        let dependency = cap[1].to_string();
        if !built_in_crates
            .iter()
            .any(|builtin| dependency.starts_with(builtin))
        {
            if let Some((dep, _)) = dependency.split_once(':') {
                dependencies.insert(dep.to_owned());
            }
        }
    }

    // Similar checks for other regex patterns

    for cap in macro_use_regex.captures_iter(code) {
        let dependency = cap[1].to_string();
        if !built_in_crates
            .iter()
            .any(|builtin| dependency.starts_with(builtin))
        {
            dependencies.insert(dependency);
        }
    }

    for cap in extern_crate_regex.captures_iter(code) {
        let dependency = cap[1].to_string();
        if !built_in_crates
            .iter()
            .any(|builtin| dependency.starts_with(builtin))
        {
            dependencies.insert(dependency);
        }
    }

    dependencies
}

pub(crate) fn build_code_path(source_stem: &str) -> Result<PathBuf, Box<dyn Error>> {
    let source_name = format!("{source_stem}.rs");
    let project_dir = env::var("PWD")?;
    let project_path = PathBuf::from(project_dir);
    let mut code_path: PathBuf = project_path.join("examples");
    // let default_toml_path = code_path.join("default_cargo.toml");
    code_path.push(source_name);
    Ok(code_path)
}

/// Set up the processing flags from the command line arguments and pass them back.
pub(crate) fn get_proc_flags(options: &cmd_args::Opt) -> Result<ProcFlags, Box<dyn Error>> {
    let flags = {
        if options.all && options.no_run {
            // println!(
            //     "Conflicting options {} and {} specified",
            //     options.all, options.no_run
            // );
            return Err(Box::new(BuildRunError::Command(format!(
                "Conflicting options {} and {} specified",
                options.all, options.no_run
            ))));
        }
        let mut flags = ProcFlags::empty();
        flags.set(ProcFlags::GEN_SRC, options.gen_src | options.all);
        flags.set(ProcFlags::GEN_TOML, options.gen_cargo | options.all);
        flags.set(ProcFlags::BUILD, options.build | options.all);
        flags.set(ProcFlags::VERBOSE, options.verbose);
        flags.set(ProcFlags::TIMINGS, options.timings);
        flags.set(ProcFlags::RUN, !options.no_run);
        flags.set(ProcFlags::ALL, options.all);
        if !(flags.contains(ProcFlags::ALL)) {
            flags.set(
                ProcFlags::ALL,
                options.gen_src & options.gen_cargo & options.build && !options.no_run,
            );
        }

        let formatted = flags.to_string();
        let parsed = formatted
            .parse::<ProcFlags>()
            .map_err(|e| BuildRunError::FromStr(e.to_string()))?;

        assert_eq!(flags, parsed);

        Ok::<cmd_args::ProcFlags, BuildRunError>(flags)
    }?;
    Ok(flags)
}

pub(crate) fn parse_source(
    options: &cmd_args::Opt,
) -> Result<(String, String, CargoManifest, String), Box<dyn Error>> {
    let start_parsing_rs = Instant::now();

    let rs_suffix = ".rs";
    debug!(
        "options.script={}; options.script.ends_with(rs_suffix)? {})",
        options.script,
        options.script.ends_with(rs_suffix)
    );
    let (source_stem, source_name) = if options.script.ends_with(rs_suffix) {
        let stem = String::from(options.script.strip_suffix(rs_suffix).ok_or_else(|| {
            BuildRunError::NoneOption(format!("Failed to strip {rs_suffix} suffix"))
        })?);
        (stem, options.script.clone())
    } else {
        (options.script.clone(), options.script.clone() + rs_suffix)
    };
    debug!("source_stem={source_stem}; source_name={source_name}");
    let code_path = build_code_path(&source_stem)?;
    let source_path = code_path.clone();
    debug!("code_path={code_path:?}");
    if !source_path.exists() {
        return Err(Box::new(BuildRunError::Command(format!(
            "No script named {source_stem} or {source_name} in path {code_path:?}"
        ))));
    }
    let rs_full_source = read_file_contents(&code_path)?;
    let rs_manifest = rs_extract_manifest(&rs_full_source)?;
    let rs_source = rs_extract_src(&rs_full_source);

    let dur = start_parsing_rs.elapsed();
    debug!(
        "Parsed source in {}.{}s",
        dur.as_secs(),
        dur.subsec_millis()
    );
    Ok((source_stem, source_name, rs_manifest, rs_source))
}
