use crate::config::{self};
use crate::debug_log;
use {crate::log, crate::logging::Verbosity};

use firestorm::profile_fn;
use lazy_static::lazy_static;
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

        let color_support: Option<ColorSupport> = if let Some(config) = &*config::MAYBE_CONFIG {
            match config.colors.color_support {
                ColorSupport::Xterm256 | ColorSupport::Ansi16 | ColorSupport::None => Some(config.colors.color_support.clone()),
                ColorSupport::Default => get_color_level(),
            }
        } else {
            get_color_level()
        };
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
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
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
    color_level = translate_level(supports_color());
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
#[derive(Clone, Debug, Default, Deserialize, EnumString, Display, PartialEq)]
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
#[derive(Clone, Debug, Default, Deserialize, EnumString, Display, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum TermTheme {
    Light,
    #[default]
    Dark,
}

/// An enum to categorise the supported message types for display.
#[derive(Debug, Clone, Copy, EnumString, Display, PartialEq)]
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
    fn get_style(&self) -> nu_ansi_term::Style;
}

/// An enum of all the supported message styles for different levels of terminal colour support.
#[derive(Clone, Debug, Display, EnumIter, EnumString, PartialEq)]
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
    fn get_style(&self) -> nu_ansi_term::Style {
        match self {
            MessageStyle::Ansi16LightError => nu_ansi_term::Color::Red.bold(),
            MessageStyle::Ansi16LightWarning => nu_ansi_term::Color::Magenta.bold(),
            MessageStyle::Ansi16LightEmphasis => nu_ansi_term::Color::Yellow.bold(),
            MessageStyle::Ansi16LightHeading => nu_ansi_term::Color::Blue.bold(),
            MessageStyle::Ansi16LightSubheading => nu_ansi_term::Color::Cyan.bold(),
            MessageStyle::Ansi16LightNormal => nu_ansi_term::Color::White.normal(),
            MessageStyle::Ansi16LightDebug => nu_ansi_term::Color::Cyan.normal(),
            MessageStyle::Ansi16LightGhost => nu_ansi_term::Color::Cyan.dimmed().italic(),
            MessageStyle::Ansi16DarkError => nu_ansi_term::Color::Red.bold(),
            MessageStyle::Ansi16DarkWarning => nu_ansi_term::Color::Magenta.bold(),
            MessageStyle::Ansi16DarkEmphasis => nu_ansi_term::Color::Yellow.bold(),
            MessageStyle::Ansi16DarkHeading => nu_ansi_term::Color::Cyan.bold(),
            MessageStyle::Ansi16DarkSubheading => nu_ansi_term::Color::Green.bold(),
            MessageStyle::Ansi16DarkNormal => nu_ansi_term::Color::White.normal(),
            MessageStyle::Ansi16DarkDebug => nu_ansi_term::Color::Cyan.normal(),
            MessageStyle::Ansi16DarkGhost => nu_ansi_term::Color::LightGray.dimmed().italic(),
            MessageStyle::Xterm256LightError => XtermColor::GuardsmanRed.get_color().bold(),
            MessageStyle::Xterm256LightWarning => XtermColor::DarkPurplePizzazz.get_color().bold(),
            MessageStyle::Xterm256LightEmphasis => XtermColor::Copperfield.get_color().bold(),
            MessageStyle::Xterm256LightHeading => XtermColor::MidnightBlue.get_color().bold(),
            MessageStyle::Xterm256LightSubheading => XtermColor::ScienceBlue.get_color().normal(),
            MessageStyle::Xterm256LightNormal => XtermColor::Black.get_color().normal(),
            MessageStyle::Xterm256LightDebug => XtermColor::LochmaraBlue.get_color().normal(),
            MessageStyle::Xterm256LightGhost => XtermColor::Boulder.get_color().normal().italic(),
            MessageStyle::Xterm256DarkError => XtermColor::GuardsmanRed.get_color().bold(),
            MessageStyle::Xterm256DarkWarning => XtermColor::DarkViolet.get_color().bold(),
            MessageStyle::Xterm256DarkEmphasis => XtermColor::Copperfield.get_color().bold(),
            MessageStyle::Xterm256DarkHeading => XtermColor::DarkMalibuBlue.get_color().bold(),
            MessageStyle::Xterm256DarkSubheading => XtermColor::CaribbeanGreen.get_color().normal(),
            MessageStyle::Xterm256DarkNormal => XtermColor::Silver.get_color().normal(),
            MessageStyle::Xterm256DarkDebug => XtermColor::BondiBlue.get_color().normal(),
            MessageStyle::Xterm256DarkGhost => XtermColor::Silver.get_color().normal().italic(),
        }
    }
}

/// Determine what message colour and style to use based on the current terminal's level of
/// colour support and light or dark theme, and the category of message to be displayed.
#[must_use]
pub fn nu_resolve_style(message_level: MessageLevel) -> nu_ansi_term::Style {
    let maybe_color_support = COLOR_SUPPORT.as_ref();
    if let Some(color_support) = maybe_color_support {
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
        match message_style {
            Ok(message_style) => NuThemeStyle::get_style(&message_style),
            Err(_) => nu_ansi_term::Style::default(),
        }
    } else {
        nu_ansi_term::Style::default()
    }
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
            XtermColor::UserBlack => 0,
            XtermColor::UserRed => 1,
            XtermColor::UserGreen => 2,
            XtermColor::UserYellow => 3,
            XtermColor::OrientBlue => 24,
            XtermColor::EndeavourBlue => 25,
            XtermColor::ScienceBlue => 26,
            XtermColor::BlueRibbon => 27,
            XtermColor::JapaneseLaurel => 28,
            XtermColor::DeepSeaGreen => 29,
            XtermColor::Teal => 30,
            XtermColor::DeepCerulean => 31,
            XtermColor::LochmaraBlue => 32,
            XtermColor::AzureRadiance => 33,
            XtermColor::LightJapaneseLaurel => 34,
            XtermColor::Jade => 35,
            XtermColor::PersianGreen => 36,
            XtermColor::BondiBlue => 37,
            XtermColor::Cerulean => 38,
            XtermColor::LightAzureRadiance => 39,
            XtermColor::DarkGreen => 40,
            XtermColor::Malachite => 41,
            XtermColor::CaribbeanGreen => 42,
            XtermColor::LightCaribbeanGreen => 43,
            XtermColor::RobinEggBlue => 44,
            XtermColor::Aqua => 45,
            XtermColor::Green => 46,
            XtermColor::DarkSpringGreen => 47,
            XtermColor::SpringGreen => 48,
            XtermColor::LightSpringGreen => 49,
            XtermColor::BrightTurquoise => 50,
            XtermColor::Cyan => 51,
            XtermColor::Rosewood => 52,
            XtermColor::PompadourMagenta => 53,
            XtermColor::PigmentIndigo => 54,
            XtermColor::DarkPurple => 55,
            XtermColor::ElectricIndigo => 56,
            XtermColor::ElectricPurple => 57,
            XtermColor::VerdunGreen => 58,
            XtermColor::ScorpionOlive => 59,
            XtermColor::Lilac => 60,
            XtermColor::ScampiIndigo => 61,
            XtermColor::Indigo => 62,
            XtermColor::DarkCornflowerBlue => 63,
            XtermColor::DarkLimeade => 64,
            XtermColor::GladeGreen => 65,
            XtermColor::JuniperGreen => 66,
            XtermColor::HippieBlue => 67,
            XtermColor::HavelockBlue => 68,
            XtermColor::CornflowerBlue => 69,
            XtermColor::Limeade => 70,
            XtermColor::FernGreen => 71,
            XtermColor::SilverTree => 72,
            XtermColor::Tradewind => 73,
            XtermColor::ShakespeareBlue => 74,
            XtermColor::DarkMalibuBlue => 75,
            XtermColor::DarkBrightGreen => 76,
            XtermColor::DarkPastelGreen => 77,
            XtermColor::PastelGreen => 78,
            XtermColor::DownyTeal => 79,
            XtermColor::Viking => 80,
            XtermColor::MalibuBlue => 81,
            XtermColor::BrightGreen => 82,
            XtermColor::DarkScreaminGreen => 83,
            XtermColor::ScreaminGreen => 84,
            XtermColor::DarkAquamarine => 85,
            XtermColor::Aquamarine => 86,
            XtermColor::LightAquamarine => 87,
            XtermColor::Maroon => 88,
            XtermColor::DarkFreshEggplant => 89,
            XtermColor::LightFreshEggplant => 90,
            XtermColor::Purple => 91,
            XtermColor::ElectricViolet => 92,
            XtermColor::LightElectricViolet => 93,
            XtermColor::Brown => 94,
            XtermColor::CopperRose => 95,
            XtermColor::StrikemasterPurple => 96,
            XtermColor::DelugePurple => 97,
            XtermColor::DarkMediumPurple => 98,
            XtermColor::DarkHeliotropePurple => 99,
            XtermColor::Olive => 100,
            XtermColor::ClayCreekOlive => 101,
            XtermColor::DarkGray => 102,
            XtermColor::WildBlueYonder => 103,
            XtermColor::ChetwodeBlue => 104,
            XtermColor::SlateBlue => 105,
            XtermColor::LightLimeade => 106,
            XtermColor::ChelseaCucumber => 107,
            XtermColor::BayLeaf => 108,
            XtermColor::GulfStream => 109,
            XtermColor::PoloBlue => 110,
            XtermColor::LightMalibuBlue => 111,
            XtermColor::Pistachio => 112,
            XtermColor::LightPastelGreen => 113,
            XtermColor::DarkFeijoaGreen => 114,
            XtermColor::VistaBlue => 115,
            XtermColor::Bermuda => 116,
            XtermColor::DarkAnakiwaBlue => 117,
            XtermColor::ChartreuseGreen => 118,
            XtermColor::LightScreaminGreen => 119,
            XtermColor::DarkMintGreen => 120,
            XtermColor::MintGreen => 121,
            XtermColor::LighterAquamarine => 122,
            XtermColor::AnakiwaBlue => 123,
            XtermColor::BrightRed => 124,
            XtermColor::DarkFlirt => 125,
            XtermColor::Flirt => 126,
            XtermColor::LightFlirt => 127,
            XtermColor::DarkViolet => 128,
            XtermColor::BrightElectricViolet => 129,
            XtermColor::RoseofSharonOrange => 130,
            XtermColor::MatrixPink => 131,
            XtermColor::UserBlue => 4,
            XtermColor::UserMagenta => 5,
            XtermColor::UserCyan => 6,
            XtermColor::UserWhite => 7,
            XtermColor::UserBrightBlack => 8,
            XtermColor::UserBrightRed => 9,
            XtermColor::UserBrightGreen => 10,
            XtermColor::UserBrightYellow => 11,
            XtermColor::UserBrightBlue => 12,
            XtermColor::UserBrightMagenta => 13,
            XtermColor::UserBrightCyan => 14,
            XtermColor::UserBrightWhite => 15,
            XtermColor::Black => 16,
            XtermColor::StratosBlue => 17,
            XtermColor::NavyBlue => 18,
            XtermColor::MidnightBlue => 19,
            XtermColor::DarkBlue => 20,
            XtermColor::Blue => 21,
            XtermColor::CamaroneGreen => 22,
            XtermColor::BlueStone => 23,
            XtermColor::TapestryPink => 132,
            XtermColor::FuchsiaPink => 133,
            XtermColor::MediumPurple => 134,
            XtermColor::Heliotrope => 135,
            XtermColor::PirateGold => 136,
            XtermColor::MuesliOrange => 137,
            XtermColor::PharlapPink => 138,
            XtermColor::Bouquet => 139,
            XtermColor::Lavender => 140,
            XtermColor::LightHeliotrope => 141,
            XtermColor::BuddhaGold => 142,
            XtermColor::OliveGreen => 143,
            XtermColor::HillaryOlive => 144,
            XtermColor::SilverChalice => 145,
            XtermColor::WistfulLilac => 146,
            XtermColor::MelroseLilac => 147,
            XtermColor::RioGrandeGreen => 148,
            XtermColor::ConiferGreen => 149,
            XtermColor::Feijoa => 150,
            XtermColor::PixieGreen => 151,
            XtermColor::JungleMist => 152,
            XtermColor::LightAnakiwaBlue => 153,
            XtermColor::Lime => 154,
            XtermColor::GreenYellow => 155,
            XtermColor::LightMintGreen => 156,
            XtermColor::Celadon => 157,
            XtermColor::AeroBlue => 158,
            XtermColor::FrenchPassLightBlue => 159,
            XtermColor::GuardsmanRed => 160,
            XtermColor::RazzmatazzCerise => 161,
            XtermColor::MediumVioletRed => 162,
            XtermColor::HollywoodCerise => 163,
            XtermColor::DarkPurplePizzazz => 164,
            XtermColor::BrighterElectricViolet => 165,
            XtermColor::TennOrange => 166,
            XtermColor::RomanOrange => 167,
            XtermColor::CranberryPink => 168,
            XtermColor::HopbushPink => 169,
            XtermColor::Orchid => 170,
            XtermColor::LighterHeliotrope => 171,
            XtermColor::MangoTango => 172,
            XtermColor::Copperfield => 173,
            XtermColor::SeaPink => 174,
            XtermColor::CanCanPink => 175,
            XtermColor::LightOrchid => 176,
            XtermColor::BrightHeliotrope => 177,
            XtermColor::DarkCorn => 178,
            XtermColor::DarkTachaOrange => 179,
            XtermColor::TanBeige => 180,
            XtermColor::ClamShell => 181,
            XtermColor::ThistlePink => 182,
            XtermColor::Mauve => 183,
            XtermColor::Corn => 184,
            XtermColor::TachaOrange => 185,
            XtermColor::DecoOrange => 186,
            XtermColor::PaleGoldenrod => 187,
            XtermColor::AltoBeige => 188,
            XtermColor::FogPink => 189,
            XtermColor::ChartreuseYellow => 190,
            XtermColor::Canary => 191,
            XtermColor::Honeysuckle => 192,
            XtermColor::ReefPaleYellow => 193,
            XtermColor::SnowyMint => 194,
            XtermColor::OysterBay => 195,
            XtermColor::Red => 196,
            XtermColor::DarkRose => 197,
            XtermColor::Rose => 198,
            XtermColor::LightHollywoodCerise => 199,
            XtermColor::PurplePizzazz => 200,
            XtermColor::Fuchsia => 201,
            XtermColor::BlazeOrange => 202,
            XtermColor::BittersweetOrange => 203,
            XtermColor::WildWatermelon => 204,
            XtermColor::DarkHotPink => 205,
            XtermColor::HotPink => 206,
            XtermColor::PinkFlamingo => 207,
            XtermColor::FlushOrange => 208,
            XtermColor::Salmon => 209,
            XtermColor::VividTangerine => 210,
            XtermColor::PinkSalmon => 211,
            XtermColor::DarkLavenderRose => 212,
            XtermColor::BlushPink => 213,
            XtermColor::YellowSea => 214,
            XtermColor::TexasRose => 215,
            XtermColor::Tacao => 216,
            XtermColor::Sundown => 217,
            XtermColor::CottonCandy => 218,
            XtermColor::LavenderRose => 219,
            XtermColor::Gold => 220,
            XtermColor::Dandelion => 221,
            XtermColor::GrandisCaramel => 222,
            XtermColor::Caramel => 223,
            XtermColor::CosmosSalmon => 224,
            XtermColor::PinkLace => 225,
            XtermColor::Yellow => 226,
            XtermColor::LaserLemon => 227,
            XtermColor::DollyYellow => 228,
            XtermColor::PortafinoYellow => 229,
            XtermColor::Cumulus => 230,
            XtermColor::White => 231,
            XtermColor::DarkCodGray => 232,
            XtermColor::CodGray => 233,
            XtermColor::LightCodGray => 234,
            XtermColor::DarkMineShaft => 235,
            XtermColor::MineShaft => 236,
            XtermColor::LightMineShaft => 237,
            XtermColor::DarkTundora => 238,
            XtermColor::Tundora => 239,
            XtermColor::ScorpionGray => 240,
            XtermColor::DarkDoveGray => 241,
            XtermColor::DoveGray => 242,
            XtermColor::Boulder => 243,
            XtermColor::Gray => 244,
            XtermColor::LightGray => 245,
            XtermColor::DustyGray => 246,
            XtermColor::NobelGray => 247,
            XtermColor::DarkSilverChalice => 248,
            XtermColor::LightSilverChalice => 249,
            XtermColor::DarkSilver => 250,
            XtermColor::Silver => 251,
            XtermColor::DarkAlto => 252,
            XtermColor::Alto => 253,
            XtermColor::Mercury => 254,
            XtermColor::GalleryGray => 255,
        }
    }
}
