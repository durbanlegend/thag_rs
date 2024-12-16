pub enum Theme {
    Light,
    Dark,
}

pub enum TerminalType {
    Xterm256,
    Ansi16,
}

pub enum Xterm256LightStyle {
    Error,
    Warning,
    Emphasis,
    Heading,
    Subheading,
    Normal,
    Debug,
    Ghost,
}

pub enum Xterm256DarkStyle {
    Error,
    Warning,
    Emphasis,
    Heading,
    Subheading,
    Normal,
    Debug,
    Ghost,
}

pub enum Ansi16LightStyle {
    Error,
    Warning,
    Emphasis,
    Heading,
    Subheading,
    Normal,
    Debug,
    Ghost,
}

pub enum Ansi16DarkStyle {
    Error,
    Warning,
    Emphasis,
    Heading,
    Subheading,
    Normal,
    Debug,
    Ghost,
}

// Call macro to implement any additional functionality
generate_styles!(
    Xterm256LightStyle,
    Theme::Light,
    TerminalType::Xterm256,
    Xterm256DarkStyle,
    Theme::Dark,
    TerminalType::Xterm256,
    Ansi16LightStyle,
    Theme::Light,
    TerminalType::Ansi16,
    Ansi16DarkStyle,
    Theme::Dark,
    TerminalType::Ansi16
);

#[macro_export]
macro_rules! generate_styles {
    (
        $($style_enum:ident, $theme:expr, $term_type:expr),* $(,)?
    ) => {
        $(
            impl $style_enum {
                pub fn to_color(&self) -> &str {
                    match ($theme, $term_type) {
                        (Theme::Light, TerminalType::Xterm256) => "light xterm256 color",
                        (Theme::Dark, TerminalType::Xterm256) => "dark xterm256 color",
                        (Theme::Light, TerminalType::Ansi16) => "light ansi16 color",
                        (Theme::Dark, TerminalType::Ansi16) => "dark ansi16 color",
                        // _ => "default color",
                    }
                }
            }
        )*
    };
}
