/// Exploring proc macro expansion. Expansion may be enabled via the `enable` feature (default = ["expand"]) in
/// `demo/proc_macros/Cargo.toml` and the expanded macro will be displayed in the compiler output.
///
/// In this example we use an attribute macro to annotate one function with "#[warn(unused_variables)]",
/// so we DO expect to get a compiler warning about its unused variable `warn_not_in_use`.
///
/// We use the same attribute macro to annotate another function with "#[allow(unused_variables)]",
/// so we DON'T expect to get a compiler warning about its unused variable `allow_not_in_use`.
///
//# Purpose: Sample model of a basic attribute proc macro.
//# Categories: proc_macros, technique
// "use thag_demo_proc_macros..." is a "magic" import that will be substituted by proc_macros.proc_macro_crate_path
// in your config file or defaulted to "demo/proc_macros" relative to your current directory.
use thag_demo_proc_macros::attribute_basic;

#[attribute_basic(warn(unused_variables))]
fn warn_about_unused_variables() {
    let warn_not_in_use = "Warn about not in use";
    println!("fn `warn_about_unused_variables` should get a compiler warning about unused variable `warn_not_in_use`");
}

#[attribute_basic(allow(unused_variables))]
fn allow_unused_variables() {
    let allow_not_in_use = "Allow not in use";
    println!("fn `allow_unused_variables` should NOT get a compiler warning about unused variable `allow_not_in_use`");
}

warn_about_unused_variables();

allow_unused_variables();
