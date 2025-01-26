/// Second prototype of building an enum from a macro and using it thereafter.
//# Purpose: explore a technique for resolving mappings from a message level enum to corresponding
//# Categories: macros, technique
// message styles at compile time instead of dynamically while logging. This involves using macros
// to build impls for 4 enums representing the 4 combinations of light vs dark theme and 16 vs 256
// colour palette, and selecting the appropriate enum at the start of execution according to the
// user's choice of theme and the capabilities of the terminal.
#[macro_export]
// Macro to generate enums and init_styles function
macro_rules! generate_styles {
    (
        $(
            ($style_enum:ident, $term_bg_luma:ident, $color_support:ident)
        ),*
    ) => {
        $(
            #[derive(Debug)]
            pub enum $style_enum {
                Error,
                Warning,
                Info,
            }
        )*
    };
}

// Call the macro early in your codebase to generate the enums and functions
generate_styles!(
    (Xterm256LightStyle, Light, Xterm256),
    (Xterm256DarkStyle, Dark, Xterm256),
    (Ansi16LightStyle, Light, Ansi16),
    (Ansi16DarkStyle, Dark, Ansi16)
);

let x = Xterm256LightStyle::Warning;
println!("Xterm256LightStyle::Warning={x:?}");
println!("Ansi16DarkStyle::Info={:?}", Ansi16DarkStyle::Info);
