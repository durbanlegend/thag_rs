#![allow(clippy::module_name_repetitions)]
use proc_macro::TokenStream;
use quote::quote;

#[allow(clippy::too_many_lines)]
pub fn ansi_styling_support_impl(_input: TokenStream) -> TokenStream {
    let output = quote! {
        use std::fmt;

        // ANSI color codes
        #[derive(Clone, Copy)]
        pub enum Color {
            Black,
            Red,
            Green,
            Yellow,
            Blue,
            Magenta,
            Cyan,
            White,
        }

        // ANSI text effects
        #[derive(Clone, Copy)]
        pub enum Effect {
            Bold,
            Underline,
            Italic,
            Reversed,
        }

        pub struct Styled<'a> {
            text: &'a str,
            fg: Option<Color>,
            effects: Vec<Effect>,
        }

        pub trait AnsiStyleExt<'a> {
            fn style(self) -> Styled<'a>;
        }

        impl<'a> AnsiStyleExt<'a> for &'a str {
            fn style(self) -> Styled<'a> {
                Styled {
                    text: self,
                    fg: None,
                    effects: Vec::new(),
                }
            }
        }

        impl<'a> AnsiStyleExt<'a> for &'a String {
            fn style(self) -> Styled<'a> {
                self.as_str().style()
            }
        }

        impl<'a> Styled<'a> {
            pub fn fg(mut self, color: Color) -> Self {
                self.fg = Some(color);
                self
            }

            pub fn bold(mut self) -> Self {
                self.effects.push(Effect::Bold);
                self
            }

            pub fn underline(mut self) -> Self {
                self.effects.push(Effect::Underline);
                self
            }

            pub fn italic(mut self) -> Self {
                self.effects.push(Effect::Italic);
                self
            }

            pub fn reversed(mut self) -> Self {
                self.effects.push(Effect::Reversed);
                self
            }

            fn to_ansi_code(&self) -> String {
                let mut codes = Vec::new();

                for effect in &self.effects {
                    codes.push(match effect {
                        Effect::Bold => "1",
                        Effect::Underline => "4",
                        Effect::Italic => "3",
                        Effect::Reversed => "7",
                    });
                }

                if let Some(color) = self.fg {
                    codes.push(match color {
                        Color::Black => "30",
                        Color::Red => "31",
                        Color::Green => "32",
                        Color::Yellow => "33",
                        Color::Blue => "34",
                        Color::Magenta => "35",
                        Color::Cyan => "36",
                        Color::White => "37",
                    });
                }

                format!("\x1b[{}m", codes.join(";"))
            }

            fn to_ansi_reset_codes(&self) -> String {
                let mut codes = Vec::new();

                for effect in &self.effects {
                    codes.push(match effect {
                        Effect::Bold => "22",
                        Effect::Underline => "24",
                        Effect::Italic => "23",
                        Effect::Reversed => "27",
                    });
                }

                if self.fg.is_some() {
                    codes.push("39"); // Reset foreground
                }

                format!("\x1b[{}m", codes.join(";"))
            }

            pub fn embed(&self, inner: impl fmt::Display) -> String {
                format!(
                    "{}{}{}",
                    self.to_ansi_code(),
                    inner,
                    ""
                )
            }
        }

        impl fmt::Display for Styled<'_> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    f,
                    "{}{}{}",
                    self.to_ansi_code(),
                    self.text,
                    self.to_ansi_reset_codes()
                )
            }
        }
    };

    output.into()
}
