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
            // 256-color support
            Color256(u8),
            // RGB support
            Rgb(u8, u8, u8),
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
                let mut codes: Vec<String> = Vec::new();

                for effect in &self.effects {
                    codes.push(match effect {
                        Effect::Bold => "1".to_string(),
                        Effect::Underline => "4".to_string(),
                        Effect::Italic => "3".to_string(),
                        Effect::Reversed => "7".to_string(),
                    });
                }

                if let Some(color) = self.fg {
                    match color {
                        Color::Black => codes.push("30".to_string()),
                        Color::Red => codes.push("31".to_string()),
                        Color::Green => codes.push("32".to_string()),
                        Color::Yellow => codes.push("33".to_string()),
                        Color::Blue => codes.push("34".to_string()),
                        Color::Magenta => codes.push("35".to_string()),
                        Color::Cyan => codes.push("36".to_string()),
                        Color::White => codes.push("37".to_string()),
                        Color::Color256(index) => {
                            codes.push(format!("38;5;{}", index));
                        },
                        Color::Rgb(r, g, b) => {
                            codes.push(format!("38;2;{};{};{}", r, g, b));
                        },
                    }
                }

                format!("\x1b[{}m", codes.join(";"))
            }

            fn to_ansi_reset_codes(&self) -> String {
                let mut codes: Vec<String> = Vec::new();

                for effect in &self.effects {
                    codes.push(match effect {
                        Effect::Bold => "22".to_string(),
                        Effect::Underline => "24".to_string(),
                        Effect::Italic => "23".to_string(),
                        Effect::Reversed => "27".to_string(),
                    });
                }

                if self.fg.is_some() {
                    codes.push("39".to_string()); // Reset foreground
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
