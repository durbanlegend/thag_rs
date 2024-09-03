#![allow(clippy::implicit_return)]
use crate::config::{self};
use crate::debug_log;
use {crate::log, crate::logging::Verbosity};

use firestorm::profile_fn;
use lazy_static::lazy_static;
use nu_ansi_term::Style;
use serde::Deserialize;
#[cfg(target_os = "windows")]
use std::env;
use std::{fmt::Display, str::FromStr};
use strum::IntoEnumIterator;
use strum::{Display, EnumIter, EnumString};
#[cfg(not(target_os = "windows"))]
use supports_color::Stream;
#[cfg(not(target_os = "windows"))]
use termbg::Theme;

lazy_static! {
    pub static ref COLOR_SUPPORT: Option<ColorSupport> = {
        if std::env::var("TEST_ENV").is_ok() {
            #[cfg(debug_assertions)]
            debug_log!(
                "Avoiding supports_color for testing"
            );
            return Some(ColorSupport::Ansi16);
        }

        let color_support: Option<ColorSupport> = (*config::MAYBE_CONFIG).as_ref().map_or_else(get_color_level, |config| match config.colors.color_support {
            ColorSupport::Xterm256 | ColorSupport::Ansi16 | ColorSupport::None => Some(config.colors.color_support.clone()),
            ColorSupport::Default => get_color_level(),
        });
        color_support
    };

    #[derive(Debug)]
    pub static ref TERM_THEME: TermTheme = {
        if std::env::var("TEST_ENV").is_ok() {
            #[cfg(debug_assertions)]
            debug_log!(
                "Avoiding termbg for testing"
            );
            return TermTheme::Dark;
        }
        #[allow(clippy::option_if_let_else)]
        let term_theme: TermTheme = if let Some(config) = &*config::MAYBE_CONFIG {
            config.colors.term_theme.clone()
        } else {
            #[cfg(target_os = "windows")] {
            TermTheme::Dark }

            #[cfg(not(target_os = "windows"))] {
            #[cfg(debug_assertions)]
            debug_log!(
                "About to call termbg"
            );
            let timeout = std::time::Duration::from_millis(100);
            // #[cfg(debug_assertions)]
            // debug_log!("Check terminal background color");
            let theme = termbg::theme(timeout);
            // shared::clear_screen();
            match theme {
                Ok(Theme::Light) => TermTheme::Light,
                Ok(Theme::Dark) | Err(_) => TermTheme::Dark,
            }
            }
        };
        term_theme
    };
}

/// A struct of the color support details, borrowed from crate `supports-color` since we
/// can't import it because the `level` field is indispensable but private.
/// This type is returned from `supports_color::on`. See documentation for its fields for
/// more details.
#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColorLevel {
    level: usize,
    /// Basic ANSI colors are supported.
    pub has_basic: bool,
    /// 256-bit colors are supported.
    pub has_256: bool,
    /// 16 million (RGB) colors are supported.
    pub has_16m: bool,
}

#[cfg(target_os = "windows")]
fn get_color_level() -> Option<ColorSupport> {
    let color_level = translate_level(supports_color());
    match color_level {
        Some(color_level) => {
            if color_level.has_16m || color_level.has_256 {
                Some(ColorSupport::Xterm256)
            } else {
                Some(ColorSupport::Ansi16)
            }
        }
        None => None,
    }
}

#[cfg(not(target_os = "windows"))]
fn get_color_level() -> Option<ColorSupport> {
    #[cfg(debug_assertions)]
    debug_log!("About to call supports_color");
    let color_level = supports_color::on(Stream::Stdout);
    color_level.map(|color_level| {
        if color_level.has_16m || color_level.has_256 {
            ColorSupport::Xterm256
        } else {
            ColorSupport::Ansi16
        }
    })
}

#[cfg(target_os = "windows")]
fn env_force_color() -> usize {
    if let Ok(force) = env::var("FORCE_COLOR") {
        match force.as_ref() {
            "true" | "" => 1,
            "false" => 0,
            f => std::cmp::min(f.parse().unwrap_or(1), 3),
        }
    } else if let Ok(cli_clr_force) = env::var("CLICOLOR_FORCE") {
        usize::from(cli_clr_force != "0")
    } else {
        0
    }
}

#[cfg(target_os = "windows")]
fn env_no_color() -> bool {
    match as_str(&env::var("NO_COLOR")) {
        Ok("0") | Err(_) => false,
        Ok(_) => true,
    }
}

// same as Option::as_deref
#[cfg(target_os = "windows")]
fn as_str<E>(option: &Result<String, E>) -> Result<&str, &E> {
    match option {
        Ok(inner) => Ok(inner),
        Err(e) => Err(e),
    }
}

#[cfg(target_os = "windows")]
fn translate_level(level: usize) -> Option<ColorLevel> {
    if level == 0 {
        None
    } else {
        Some(ColorLevel {
            level,
            has_basic: true,
            has_256: level >= 2,
            has_16m: level >= 3,
        })
    }
}

#[cfg(target_os = "windows")]
fn supports_color() -> usize {
    profile_fn!(supports_color);
    let force_color = env_force_color();
    if force_color > 0 {
        force_color
    } else if env_no_color()
        || as_str(&env::var("TERM")) == Ok("dumb")
        || env::var("IGNORE_IS_TERMINAL").map_or(false, |v| v != "0")
    {
        0
    } else if env::var("COLORTERM").map(|colorterm| check_colorterm_16m(&colorterm)) == Ok(true)
        || env::var("TERM").map(|term| check_term_16m(&term)) == Ok(true)
        || as_str(&env::var("TERM_PROGRAM")) == Ok("iTerm.app")
    {
        3
    } else if as_str(&env::var("TERM_PROGRAM")) == Ok("Apple_Terminal")
        || env::var("TERM").map(|term| check_256_color(&term)) == Ok(true)
    {
        2
    } else {
        usize::from(
            env::var("COLORTERM").is_ok()
                || env::var("TERM").map(|term| check_ansi_color(&term)) == Ok(true)
                || env::consts::OS == "windows"
                || env::var("CLICOLOR").map_or(false, |v| v != "0"),
        )
    }
}

#[cfg(target_os = "windows")]
fn check_ansi_color(term: &str) -> bool {
    term.starts_with("screen")
        || term.starts_with("xterm")
        || term.starts_with("vt100")
        || term.starts_with("vt220")
        || term.starts_with("rxvt")
        || term.contains("color")
        || term.contains("ansi")
        || term.contains("cygwin")
        || term.contains("linux")
}

#[cfg(target_os = "windows")]
fn check_colorterm_16m(colorterm: &str) -> bool {
    colorterm == "truecolor" || colorterm == "24bit"
}

#[cfg(target_os = "windows")]
fn check_term_16m(term: &str) -> bool {
    term.ends_with("direct") || term.ends_with("truecolor")
}

#[cfg(target_os = "windows")]
fn check_256_color(term: &str) -> bool {
    term.ends_with("256") || term.ends_with("256color")
}

/// Retrieve whether the terminal theme is light or dark, to allow an appropriate colour
/// palette to be chosen.
#[must_use]
pub fn get_term_theme() -> &'static TermTheme {
    profile_fn!(get_term_theme);
    &TERM_THEME
}

/// A trait for common handling of the different colour palettes.
pub trait NuColor: Display {
    fn get_color(&self) -> nu_ansi_term::Color;
    /// Protection in case enum gets out of order, otherwise I think we could cast the variant to a number.
    fn get_fixed_code(&self) -> u8;
}

/// A version of println that prints an entire message in colour or otherwise styled.
/// Format: `nu_color_println!(style: Option<Style>, "Lorem ipsum dolor {} amet", content: &str);`
#[macro_export]
macro_rules! nu_color_println {
    ($style:expr, $($arg:tt)*) => {{
        let content = format!("{}", format_args!($($arg)*));
        let style = $style;
        // Qualified form to avoid imports in calling code.
        log!(Verbosity::Quiet, "{}\n", style.paint(content));
    }};
}

/// An enum to categorise the current terminal's level of colour support as detected, configured
/// or defaulted. We fold `TrueColor` into Xterm256 as we're not interested in more than 256
/// colours just for messages.
#[derive(Clone, Debug, Default, Deserialize, EnumString, Display, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum ColorSupport {
    Xterm256,
    Ansi16,
    None,
    #[default]
    Default,
}

/// An enum to categorise the current terminal's light or dark theme as detected, configured
/// or defaulted.
#[derive(Clone, Debug, Default, Deserialize, EnumString, Display, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum TermTheme {
    Light,
    #[default]
    Dark,
}

/// An enum to categorise the supported message types for display.
#[derive(Debug, Clone, Copy, EnumString, Display, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum MessageLevel {
    Error,
    Warning,
    Emphasis,
    Heading,
    Subheading,
    Normal,
    Debug,
    Ghost,
}

/// A trait to map a `MessageStyle` to a `nu_ansi_term::Style`.
pub trait NuThemeStyle: Display {
    fn get_style(&self) -> Style;
}

/// An enum of all the supported message styles for different levels of terminal colour support.
#[derive(Clone, Debug, Display, EnumIter, EnumString, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
#[strum(use_phf)]
pub enum MessageStyle {
    Ansi16LightError,
    Ansi16LightWarning,
    Ansi16LightEmphasis,
    Ansi16LightHeading,
    Ansi16LightSubheading,
    Ansi16LightNormal,
    Ansi16LightDebug,
    Ansi16LightGhost,

    Ansi16DarkError,
    Ansi16DarkWarning,
    Ansi16DarkEmphasis,
    Ansi16DarkHeading,
    Ansi16DarkSubheading,
    Ansi16DarkNormal,
    Ansi16DarkDebug,
    Ansi16DarkGhost,

    Xterm256LightError,
    Xterm256LightWarning,
    Xterm256LightEmphasis,
    Xterm256LightHeading,
    Xterm256LightSubheading,
    Xterm256LightNormal,
    Xterm256LightDebug,
    Xterm256LightGhost,

    Xterm256DarkError,
    Xterm256DarkWarning,
    Xterm256DarkEmphasis,
    Xterm256DarkHeading,
    Xterm256DarkSubheading,
    Xterm256DarkNormal,
    Xterm256DarkDebug,
    Xterm256DarkGhost,
}

/// Define the implementation of the `NuThemeStyle` trait for `MessageStyle` to facilitate
/// resolution of the `MessageStyle` variant to an `nu_ansi_term::Style`.
#[allow(clippy::match_same_arms)]
impl NuThemeStyle for MessageStyle {
    fn get_style(&self) -> Style {
        match self {
            Self::Ansi16LightError => nu_ansi_term::Color::Red.bold(),
            Self::Ansi16LightWarning => nu_ansi_term::Color::Magenta.bold(),
            Self::Ansi16LightEmphasis => nu_ansi_term::Color::Yellow.bold(),
            Self::Ansi16LightHeading => nu_ansi_term::Color::Blue.bold(),
            Self::Ansi16LightSubheading => nu_ansi_term::Color::Cyan.bold(),
            Self::Ansi16LightNormal => nu_ansi_term::Color::White.normal(),
            Self::Ansi16LightDebug => nu_ansi_term::Color::Cyan.normal(),
            Self::Ansi16LightGhost => nu_ansi_term::Color::Cyan.dimmed().italic(),
            Self::Ansi16DarkError => nu_ansi_term::Color::Red.bold(),
            Self::Ansi16DarkWarning => nu_ansi_term::Color::Magenta.bold(),
            Self::Ansi16DarkEmphasis => nu_ansi_term::Color::Yellow.bold(),
            Self::Ansi16DarkHeading => nu_ansi_term::Color::Cyan.bold(),
            Self::Ansi16DarkSubheading => nu_ansi_term::Color::Green.bold(),
            Self::Ansi16DarkNormal => nu_ansi_term::Color::White.normal(),
            Self::Ansi16DarkDebug => nu_ansi_term::Color::Cyan.normal(),
            Self::Ansi16DarkGhost => nu_ansi_term::Color::LightGray.dimmed().italic(),
            Self::Xterm256LightError => XtermColor::GuardsmanRed.get_color().bold(),
            Self::Xterm256LightWarning => XtermColor::DarkPurplePizzazz.get_color().bold(),
            Self::Xterm256LightEmphasis => XtermColor::Copperfield.get_color().bold(),
            Self::Xterm256LightHeading => XtermColor::MidnightBlue.get_color().bold(),
            Self::Xterm256LightSubheading => XtermColor::ScienceBlue.get_color().normal(),
            Self::Xterm256LightNormal => XtermColor::Black.get_color().normal(),
            Self::Xterm256LightDebug => XtermColor::LochmaraBlue.get_color().normal(),
            Self::Xterm256LightGhost => XtermColor::Boulder.get_color().normal().italic(),
            Self::Xterm256DarkError => XtermColor::GuardsmanRed.get_color().bold(),
            Self::Xterm256DarkWarning => XtermColor::DarkViolet.get_color().bold(),
            Self::Xterm256DarkEmphasis => XtermColor::Copperfield.get_color().bold(),
            Self::Xterm256DarkHeading => XtermColor::DarkMalibuBlue.get_color().bold(),
            Self::Xterm256DarkSubheading => XtermColor::CaribbeanGreen.get_color().normal(),
            Self::Xterm256DarkNormal => XtermColor::Silver.get_color().normal(),
            Self::Xterm256DarkDebug => XtermColor::BondiBlue.get_color().normal(),
            Self::Xterm256DarkGhost => XtermColor::Silver.get_color().normal().italic(),
        }
    }
}

/// Determine what message colour and style to use based on the current terminal's level of
/// colour support and light or dark theme, and the category of message to be displayed.
#[must_use]
pub fn nu_resolve_style(message_level: MessageLevel) -> Style {
    let maybe_color_support = COLOR_SUPPORT.as_ref();
    maybe_color_support.map_or_else(Style::default, |color_support| {
        let color_qual = color_support.to_string().to_lowercase();
        let theme_qual = TERM_THEME.to_string().to_lowercase();
        let msg_level_qual = message_level.to_string().to_lowercase();
        let message_style = MessageStyle::from_str(&format!(
            "{}_{}_{}",
            &color_qual, &theme_qual, &msg_level_qual
        ));
        #[cfg(debug_assertions)]
        debug_log!(
            "Called from_str on {color_qual}_{theme_qual}_{msg_level_qual}, found {message_style:#?}",
        );
        message_style.map_or_else(|_| Style::default(), |message_style| NuThemeStyle::get_style(&message_style))
    })
}

/// Main function for use by testing or the script runner.
#[allow(dead_code)]
pub fn main() {
    #[cfg(not(target_os = "windows"))]
    {
        let term = termbg::terminal();
        // shared::clear_screen();
        #[cfg(debug_assertions)]
        debug_log!("  Term : {term:?}");
    }

    let color_support = &*COLOR_SUPPORT;

    match color_support {
        None => {
            log!(Verbosity::Normal, "No colour support found for terminal");
        }
        Some(support) => {
            log!(
                Verbosity::Normal,
                "{}",
                nu_resolve_style(MessageLevel::Warning).paint("Colored Warning message\n")
            );

            for variant in MessageStyle::iter() {
                let variant_string: &str = &variant.to_string();
                log!(
                    Verbosity::Normal,
                    "My {} message",
                    variant.get_style().paint(variant_string)
                );
            }

            if matches!(support, ColorSupport::Xterm256) {
                log!(Verbosity::Normal, "");
                XtermColor::iter().for_each(|variant| {
                    let color = variant.get_color();
                    log!(Verbosity::Normal, "{}", color.paint(variant.to_string()));
                });
            }
        }
    }
}

/// An enum of the colours in a 256-colour palette.
#[cfg(debug_assertions)]
#[allow(dead_code)]
#[derive(Display, EnumIter)]
pub enum XtermColor {
    UserBlack,
    UserRed,
    UserGreen,
    UserYellow,
    UserBlue,
    UserMagenta,
    UserCyan,
    UserWhite,
    UserBrightBlack,
    UserBrightRed,
    UserBrightGreen,
    UserBrightYellow,
    UserBrightBlue,
    UserBrightMagenta,
    UserBrightCyan,
    UserBrightWhite,
    Black,
    StratosBlue,
    NavyBlue,
    MidnightBlue,
    DarkBlue,
    Blue,
    CamaroneGreen,
    BlueStone,
    OrientBlue,
    EndeavourBlue,
    ScienceBlue,
    BlueRibbon,
    JapaneseLaurel,
    DeepSeaGreen,
    Teal,
    DeepCerulean,
    LochmaraBlue,
    AzureRadiance,
    LightJapaneseLaurel,
    Jade,
    PersianGreen,
    BondiBlue,
    Cerulean,
    LightAzureRadiance,
    DarkGreen,
    Malachite,
    CaribbeanGreen,
    LightCaribbeanGreen,
    RobinEggBlue,
    Aqua,
    Green,
    DarkSpringGreen,
    SpringGreen,
    LightSpringGreen,
    BrightTurquoise,
    Cyan,
    Rosewood,
    PompadourMagenta,
    PigmentIndigo,
    DarkPurple,
    ElectricIndigo,
    ElectricPurple,
    VerdunGreen,
    ScorpionOlive,
    Lilac,
    ScampiIndigo,
    Indigo,
    DarkCornflowerBlue,
    DarkLimeade,
    GladeGreen,
    JuniperGreen,
    HippieBlue,
    HavelockBlue,
    CornflowerBlue,
    Limeade,
    FernGreen,
    SilverTree,
    Tradewind,
    ShakespeareBlue,
    DarkMalibuBlue,
    DarkBrightGreen,
    DarkPastelGreen,
    PastelGreen,
    DownyTeal,
    Viking,
    MalibuBlue,
    BrightGreen,
    DarkScreaminGreen,
    ScreaminGreen,
    DarkAquamarine,
    Aquamarine,
    LightAquamarine,
    Maroon,
    DarkFreshEggplant,
    LightFreshEggplant,
    Purple,
    ElectricViolet,
    LightElectricViolet,
    Brown,
    CopperRose,
    StrikemasterPurple,
    DelugePurple,
    DarkMediumPurple,
    DarkHeliotropePurple,
    Olive,
    ClayCreekOlive,
    DarkGray,
    WildBlueYonder,
    ChetwodeBlue,
    SlateBlue,
    LightLimeade,
    ChelseaCucumber,
    BayLeaf,
    GulfStream,
    PoloBlue,
    LightMalibuBlue,
    Pistachio,
    LightPastelGreen,
    DarkFeijoaGreen,
    VistaBlue,
    Bermuda,
    DarkAnakiwaBlue,
    ChartreuseGreen,
    LightScreaminGreen,
    DarkMintGreen,
    MintGreen,
    LighterAquamarine,
    AnakiwaBlue,
    BrightRed,
    DarkFlirt,
    Flirt,
    LightFlirt,
    DarkViolet,
    BrightElectricViolet,
    RoseofSharonOrange,
    MatrixPink,
    TapestryPink,
    FuchsiaPink,
    MediumPurple,
    Heliotrope,
    PirateGold,
    MuesliOrange,
    PharlapPink,
    Bouquet,
    Lavender,
    LightHeliotrope,
    BuddhaGold,
    OliveGreen,
    HillaryOlive,
    SilverChalice,
    WistfulLilac,
    MelroseLilac,
    RioGrandeGreen,
    ConiferGreen,
    Feijoa,
    PixieGreen,
    JungleMist,
    LightAnakiwaBlue,
    Lime,
    GreenYellow,
    LightMintGreen,
    Celadon,
    AeroBlue,
    FrenchPassLightBlue,
    GuardsmanRed,
    RazzmatazzCerise,
    MediumVioletRed,
    HollywoodCerise,
    DarkPurplePizzazz,
    BrighterElectricViolet,
    TennOrange,
    RomanOrange,
    CranberryPink,
    HopbushPink,
    Orchid,
    LighterHeliotrope,
    MangoTango,
    Copperfield,
    SeaPink,
    CanCanPink,
    LightOrchid,
    BrightHeliotrope,
    DarkCorn,
    DarkTachaOrange,
    TanBeige,
    ClamShell,
    ThistlePink,
    Mauve,
    Corn,
    TachaOrange,
    DecoOrange,
    PaleGoldenrod,
    AltoBeige,
    FogPink,
    ChartreuseYellow,
    Canary,
    Honeysuckle,
    ReefPaleYellow,
    SnowyMint,
    OysterBay,
    Red,
    DarkRose,
    Rose,
    LightHollywoodCerise,
    PurplePizzazz,
    Fuchsia,
    BlazeOrange,
    BittersweetOrange,
    WildWatermelon,
    DarkHotPink,
    HotPink,
    PinkFlamingo,
    FlushOrange,
    Salmon,
    VividTangerine,
    PinkSalmon,
    DarkLavenderRose,
    BlushPink,
    YellowSea,
    TexasRose,
    Tacao,
    Sundown,
    CottonCandy,
    LavenderRose,
    Gold,
    Dandelion,
    GrandisCaramel,
    Caramel,
    CosmosSalmon,
    PinkLace,
    Yellow,
    LaserLemon,
    DollyYellow,
    PortafinoYellow,
    Cumulus,
    White,
    DarkCodGray,
    CodGray,
    LightCodGray,
    DarkMineShaft,
    MineShaft,
    LightMineShaft,
    DarkTundora,
    Tundora,
    ScorpionGray,
    DarkDoveGray,
    DoveGray,
    Boulder,
    Gray,
    LightGray,
    DustyGray,
    NobelGray,
    DarkSilverChalice,
    LightSilverChalice,
    DarkSilver,
    Silver,
    DarkAlto,
    Alto,
    Mercury,
    GalleryGray,
}

impl NuColor for XtermColor {
    fn get_color(&self) -> nu_ansi_term::Color {
        nu_ansi_term::Color::Fixed(self.get_fixed_code())
    }

    #[allow(clippy::too_many_lines)]
    fn get_fixed_code(&self) -> u8 {
        match self {
            Self::UserBlack => 0,
            Self::UserRed => 1,
            Self::UserGreen => 2,
            Self::UserYellow => 3,
            Self::OrientBlue => 24,
            Self::EndeavourBlue => 25,
            Self::ScienceBlue => 26,
            Self::BlueRibbon => 27,
            Self::JapaneseLaurel => 28,
            Self::DeepSeaGreen => 29,
            Self::Teal => 30,
            Self::DeepCerulean => 31,
            Self::LochmaraBlue => 32,
            Self::AzureRadiance => 33,
            Self::LightJapaneseLaurel => 34,
            Self::Jade => 35,
            Self::PersianGreen => 36,
            Self::BondiBlue => 37,
            Self::Cerulean => 38,
            Self::LightAzureRadiance => 39,
            Self::DarkGreen => 40,
            Self::Malachite => 41,
            Self::CaribbeanGreen => 42,
            Self::LightCaribbeanGreen => 43,
            Self::RobinEggBlue => 44,
            Self::Aqua => 45,
            Self::Green => 46,
            Self::DarkSpringGreen => 47,
            Self::SpringGreen => 48,
            Self::LightSpringGreen => 49,
            Self::BrightTurquoise => 50,
            Self::Cyan => 51,
            Self::Rosewood => 52,
            Self::PompadourMagenta => 53,
            Self::PigmentIndigo => 54,
            Self::DarkPurple => 55,
            Self::ElectricIndigo => 56,
            Self::ElectricPurple => 57,
            Self::VerdunGreen => 58,
            Self::ScorpionOlive => 59,
            Self::Lilac => 60,
            Self::ScampiIndigo => 61,
            Self::Indigo => 62,
            Self::DarkCornflowerBlue => 63,
            Self::DarkLimeade => 64,
            Self::GladeGreen => 65,
            Self::JuniperGreen => 66,
            Self::HippieBlue => 67,
            Self::HavelockBlue => 68,
            Self::CornflowerBlue => 69,
            Self::Limeade => 70,
            Self::FernGreen => 71,
            Self::SilverTree => 72,
            Self::Tradewind => 73,
            Self::ShakespeareBlue => 74,
            Self::DarkMalibuBlue => 75,
            Self::DarkBrightGreen => 76,
            Self::DarkPastelGreen => 77,
            Self::PastelGreen => 78,
            Self::DownyTeal => 79,
            Self::Viking => 80,
            Self::MalibuBlue => 81,
            Self::BrightGreen => 82,
            Self::DarkScreaminGreen => 83,
            Self::ScreaminGreen => 84,
            Self::DarkAquamarine => 85,
            Self::Aquamarine => 86,
            Self::LightAquamarine => 87,
            Self::Maroon => 88,
            Self::DarkFreshEggplant => 89,
            Self::LightFreshEggplant => 90,
            Self::Purple => 91,
            Self::ElectricViolet => 92,
            Self::LightElectricViolet => 93,
            Self::Brown => 94,
            Self::CopperRose => 95,
            Self::StrikemasterPurple => 96,
            Self::DelugePurple => 97,
            Self::DarkMediumPurple => 98,
            Self::DarkHeliotropePurple => 99,
            Self::Olive => 100,
            Self::ClayCreekOlive => 101,
            Self::DarkGray => 102,
            Self::WildBlueYonder => 103,
            Self::ChetwodeBlue => 104,
            Self::SlateBlue => 105,
            Self::LightLimeade => 106,
            Self::ChelseaCucumber => 107,
            Self::BayLeaf => 108,
            Self::GulfStream => 109,
            Self::PoloBlue => 110,
            Self::LightMalibuBlue => 111,
            Self::Pistachio => 112,
            Self::LightPastelGreen => 113,
            Self::DarkFeijoaGreen => 114,
            Self::VistaBlue => 115,
            Self::Bermuda => 116,
            Self::DarkAnakiwaBlue => 117,
            Self::ChartreuseGreen => 118,
            Self::LightScreaminGreen => 119,
            Self::DarkMintGreen => 120,
            Self::MintGreen => 121,
            Self::LighterAquamarine => 122,
            Self::AnakiwaBlue => 123,
            Self::BrightRed => 124,
            Self::DarkFlirt => 125,
            Self::Flirt => 126,
            Self::LightFlirt => 127,
            Self::DarkViolet => 128,
            Self::BrightElectricViolet => 129,
            Self::RoseofSharonOrange => 130,
            Self::MatrixPink => 131,
            Self::UserBlue => 4,
            Self::UserMagenta => 5,
            Self::UserCyan => 6,
            Self::UserWhite => 7,
            Self::UserBrightBlack => 8,
            Self::UserBrightRed => 9,
            Self::UserBrightGreen => 10,
            Self::UserBrightYellow => 11,
            Self::UserBrightBlue => 12,
            Self::UserBrightMagenta => 13,
            Self::UserBrightCyan => 14,
            Self::UserBrightWhite => 15,
            Self::Black => 16,
            Self::StratosBlue => 17,
            Self::NavyBlue => 18,
            Self::MidnightBlue => 19,
            Self::DarkBlue => 20,
            Self::Blue => 21,
            Self::CamaroneGreen => 22,
            Self::BlueStone => 23,
            Self::TapestryPink => 132,
            Self::FuchsiaPink => 133,
            Self::MediumPurple => 134,
            Self::Heliotrope => 135,
            Self::PirateGold => 136,
            Self::MuesliOrange => 137,
            Self::PharlapPink => 138,
            Self::Bouquet => 139,
            Self::Lavender => 140,
            Self::LightHeliotrope => 141,
            Self::BuddhaGold => 142,
            Self::OliveGreen => 143,
            Self::HillaryOlive => 144,
            Self::SilverChalice => 145,
            Self::WistfulLilac => 146,
            Self::MelroseLilac => 147,
            Self::RioGrandeGreen => 148,
            Self::ConiferGreen => 149,
            Self::Feijoa => 150,
            Self::PixieGreen => 151,
            Self::JungleMist => 152,
            Self::LightAnakiwaBlue => 153,
            Self::Lime => 154,
            Self::GreenYellow => 155,
            Self::LightMintGreen => 156,
            Self::Celadon => 157,
            Self::AeroBlue => 158,
            Self::FrenchPassLightBlue => 159,
            Self::GuardsmanRed => 160,
            Self::RazzmatazzCerise => 161,
            Self::MediumVioletRed => 162,
            Self::HollywoodCerise => 163,
            Self::DarkPurplePizzazz => 164,
            Self::BrighterElectricViolet => 165,
            Self::TennOrange => 166,
            Self::RomanOrange => 167,
            Self::CranberryPink => 168,
            Self::HopbushPink => 169,
            Self::Orchid => 170,
            Self::LighterHeliotrope => 171,
            Self::MangoTango => 172,
            Self::Copperfield => 173,
            Self::SeaPink => 174,
            Self::CanCanPink => 175,
            Self::LightOrchid => 176,
            Self::BrightHeliotrope => 177,
            Self::DarkCorn => 178,
            Self::DarkTachaOrange => 179,
            Self::TanBeige => 180,
            Self::ClamShell => 181,
            Self::ThistlePink => 182,
            Self::Mauve => 183,
            Self::Corn => 184,
            Self::TachaOrange => 185,
            Self::DecoOrange => 186,
            Self::PaleGoldenrod => 187,
            Self::AltoBeige => 188,
            Self::FogPink => 189,
            Self::ChartreuseYellow => 190,
            Self::Canary => 191,
            Self::Honeysuckle => 192,
            Self::ReefPaleYellow => 193,
            Self::SnowyMint => 194,
            Self::OysterBay => 195,
            Self::Red => 196,
            Self::DarkRose => 197,
            Self::Rose => 198,
            Self::LightHollywoodCerise => 199,
            Self::PurplePizzazz => 200,
            Self::Fuchsia => 201,
            Self::BlazeOrange => 202,
            Self::BittersweetOrange => 203,
            Self::WildWatermelon => 204,
            Self::DarkHotPink => 205,
            Self::HotPink => 206,
            Self::PinkFlamingo => 207,
            Self::FlushOrange => 208,
            Self::Salmon => 209,
            Self::VividTangerine => 210,
            Self::PinkSalmon => 211,
            Self::DarkLavenderRose => 212,
            Self::BlushPink => 213,
            Self::YellowSea => 214,
            Self::TexasRose => 215,
            Self::Tacao => 216,
            Self::Sundown => 217,
            Self::CottonCandy => 218,
            Self::LavenderRose => 219,
            Self::Gold => 220,
            Self::Dandelion => 221,
            Self::GrandisCaramel => 222,
            Self::Caramel => 223,
            Self::CosmosSalmon => 224,
            Self::PinkLace => 225,
            Self::Yellow => 226,
            Self::LaserLemon => 227,
            Self::DollyYellow => 228,
            Self::PortafinoYellow => 229,
            Self::Cumulus => 230,
            Self::White => 231,
            Self::DarkCodGray => 232,
            Self::CodGray => 233,
            Self::LightCodGray => 234,
            Self::DarkMineShaft => 235,
            Self::MineShaft => 236,
            Self::LightMineShaft => 237,
            Self::DarkTundora => 238,
            Self::Tundora => 239,
            Self::ScorpionGray => 240,
            Self::DarkDoveGray => 241,
            Self::DoveGray => 242,
            Self::Boulder => 243,
            Self::Gray => 244,
            Self::LightGray => 245,
            Self::DustyGray => 246,
            Self::NobelGray => 247,
            Self::DarkSilverChalice => 248,
            Self::LightSilverChalice => 249,
            Self::DarkSilver => 250,
            Self::Silver => 251,
            Self::DarkAlto => 252,
            Self::Alto => 253,
            Self::Mercury => 254,
            Self::GalleryGray => 255,
        }
    }
}
