/*[toml]
[dependencies]
thag_rs = { version = "1, thag-auto", default-features = false, features = ["color_detect", "core", "simplelog"] }

[features]
default = ["strum/phf"]     # Because `strum` omits to publish "phf" feature for discovery by cargo-lookup.
*/
/// A version of `thag_rs`'s  now defunct `colors` module to style messages according to their type. Like the `stdin`
/// module, `colors` was originally developed here as a separate script and integrated as a module later.
///
/// The `colors` module was superseded by `styling`. See `demo/styling_demo.rs`
///
/// E.g. `thag demo/colors_old.rs`
//# Purpose: Demo using `thag_rs` to develop a module outside of the project.
//# Categories: prototype, reference, testing
use log::debug;
use std::{fmt::Display, str::FromStr, sync::LazyLock};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};
use supports_color::Stream;
use termbg::Theme;
use thag_rs::{Verbosity, vprtln};

pub static COLOR_SUPPORT: LazyLock<Option<ColorSupport>> =
    LazyLock::new(|| match supports_color::on(Stream::Stdout) {
        Some(color_support) => {
            if color_support.has_16m || color_support.has_256 {
                Some(ColorSupport::Xterm256)
            } else {
                Some(ColorSupport::Ansi16)
            }
        }
        None => None,
    });

pub static TERM_THEME: LazyLock<TermBgLuma> = LazyLock::new(|| {
    let timeout = std::time::Duration::from_millis(100);
    debug!("Check terminal background color");
    let theme = termbg::theme(timeout);
    // clear_screen();
    match theme {
        Ok(Theme::Light) => TermBgLuma::Light,
        Ok(Theme::Dark) | Err(_) => TermBgLuma::Dark,
    }
});

pub trait NuColor: Display {
    fn get_color(&self) -> nu_ansi_term::Color;
    // Protection in case enum gets out of order, otherwise I think we could cast the variant to a number.
    fn get_fixed_code(&self) -> u8;
}

#[macro_export]
macro_rules! nu_color_println {
    ($style:expr, $($arg:tt)*) => {{
        let content = format!("{}", format_args!($($arg)*));
     let style = $style;
    // Qualified form to avoid imports in calling code.
    #[cfg(windows)] {vprtln!(Verbosity::Quiet, "{}\r", style.paint(content));} else {vprtln!(Verbosity::Quiet, "{}", style.paint(content)); }
    }};
}

#[derive(Clone, EnumString, Display, PartialEq, Eq)]
// We include `TrueColor` in Xterm256 as we're not interested in more than 256 colours just for messages.
pub enum ColorSupport {
    Xterm256,
    Ansi16,
    None,
}

#[derive(EnumString, Display, PartialEq, Eq)]
pub enum TermBgLuma {
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, EnumString, Display, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum MessageLevel {
    Error,
    Warning,
    Emphasis,
    OuterPrompt,
    InnerPrompt,
    Normal,
    Debug,
    Ghost,
}

pub trait NuThemeStyle: Display {
    fn get_style(&self) -> nu_ansi_term::Style;
}

#[derive(Clone, Debug, Display, EnumIter, EnumString, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
#[strum(use_phf)]
pub enum MessageStyle {
    Ansi16LightError,
    Ansi16LightWarning,
    Ansi16LightEmphasis,
    Ansi16LightOuterPrompt,
    Ansi16LightInnerPrompt,
    Ansi16LightNormal,
    Ansi16LightDebug,
    Ansi16LightGhost,

    Ansi16DarkError,
    Ansi16DarkWarning,
    Ansi16DarkEmphasis,
    Ansi16DarkOuterPrompt,
    Ansi16DarkInnerPrompt,
    Ansi16DarkNormal,
    Ansi16DarkDebug,
    Ansi16DarkGhost,

    Xterm256LightError,
    Xterm256LightWarning,
    Xterm256LightEmphasis,
    Xterm256LightOuterPrompt,
    Xterm256LightInnerPrompt,
    Xterm256LightNormal,
    Xterm256LightDebug,
    Xterm256LightGhost,

    Xterm256DarkError,
    Xterm256DarkWarning,
    Xterm256DarkEmphasis,
    Xterm256DarkOuterPrompt,
    Xterm256DarkInnerPrompt,
    Xterm256DarkNormal,
    Xterm256DarkDebug,
    Xterm256DarkGhost,
}

#[allow(clippy::match_same_arms)]
impl NuThemeStyle for MessageStyle {
    fn get_style(&self) -> nu_ansi_term::Style {
        match self {
            Self::Ansi16LightError => nu_ansi_term::Color::Red.bold(),
            Self::Ansi16LightWarning => nu_ansi_term::Color::Magenta.bold(),
            Self::Ansi16LightEmphasis => nu_ansi_term::Color::Yellow.bold(),
            Self::Ansi16LightOuterPrompt => nu_ansi_term::Color::Blue.bold(),
            Self::Ansi16LightInnerPrompt => nu_ansi_term::Color::Cyan.bold(),
            Self::Ansi16LightNormal => nu_ansi_term::Color::White.normal(),
            Self::Ansi16LightDebug => nu_ansi_term::Color::Cyan.normal(),
            Self::Ansi16LightGhost => nu_ansi_term::Color::Cyan.dimmed().italic(),
            Self::Ansi16DarkError => nu_ansi_term::Color::Red.bold(),
            Self::Ansi16DarkWarning => nu_ansi_term::Color::Magenta.bold(),
            Self::Ansi16DarkEmphasis => nu_ansi_term::Color::Yellow.bold(),
            Self::Ansi16DarkOuterPrompt => nu_ansi_term::Color::Cyan.bold(),
            Self::Ansi16DarkInnerPrompt => nu_ansi_term::Color::Green.bold(),
            Self::Ansi16DarkNormal => nu_ansi_term::Color::White.normal(),
            Self::Ansi16DarkDebug => nu_ansi_term::Color::Cyan.normal(),
            Self::Ansi16DarkGhost => nu_ansi_term::Color::LightGray.dimmed().italic(),
            Self::Xterm256LightError => XtermColor::GuardsmanRed.get_color().bold(),
            Self::Xterm256LightWarning => XtermColor::DarkPurplePizzazz.get_color().bold(),
            Self::Xterm256LightEmphasis => XtermColor::Copperfield.get_color().bold(),
            Self::Xterm256LightOuterPrompt => XtermColor::MidnightBlue.get_color().bold(),
            Self::Xterm256LightInnerPrompt => XtermColor::ScienceBlue.get_color().normal(),
            Self::Xterm256LightNormal => XtermColor::Black.get_color().normal(),
            Self::Xterm256LightDebug => XtermColor::LochmaraBlue.get_color().normal(),
            Self::Xterm256LightGhost => XtermColor::BittersweetOrange.get_color().normal().italic(),
            Self::Xterm256DarkError => XtermColor::GuardsmanRed.get_color().bold(),
            Self::Xterm256DarkWarning => XtermColor::DarkViolet.get_color().bold(),
            Self::Xterm256DarkEmphasis => XtermColor::Copperfield.get_color().bold(),
            Self::Xterm256DarkOuterPrompt => XtermColor::DarkMalibuBlue.get_color().bold(),
            Self::Xterm256DarkInnerPrompt => XtermColor::CaribbeanGreen.get_color().normal(),
            Self::Xterm256DarkNormal => XtermColor::Silver.get_color().normal(),
            Self::Xterm256DarkDebug => XtermColor::BondiBlue.get_color().normal(),
            Self::Xterm256DarkGhost => XtermColor::DarkSilverChalice.get_color().dimmed().italic(),
        }
    }
}

pub fn nu_resolve_style(message_level: MessageLevel) -> nu_ansi_term::Style {
    let maybe_color_support = COLOR_SUPPORT.as_ref();
    if let Some(color_support) = maybe_color_support {
        let color_qual = color_support.to_string().to_lowercase();
        let theme_qual = TERM_THEME.to_string().to_lowercase();
        let msg_level_qual = message_level.to_string().to_lowercase();
        let message_style =
            MessageStyle::from_str(&format!("{color_qual}_{theme_qual}_{msg_level_qual}"));
        debug!(
            "Called from_str on {}_{}_{}, found {message_style:#?}",
            color_qual, theme_qual, msg_level_qual,
        );
        match message_style {
            Ok(message_style) => NuThemeStyle::get_style(&message_style),
            Err(_) => nu_ansi_term::Style::default(),
        }
    } else {
        nu_ansi_term::Style::default()
    }
}

#[allow(dead_code)]
fn main() {
    let term = termbg::terminal();
    // clear_screen();
    debug!("  Term : {:?}", term);

    let color_support = match supports_color::on(Stream::Stdout) {
        Some(color_support) => {
            if color_support.has_16m || color_support.has_256 {
                Some(ColorSupport::Xterm256)
            } else {
                Some(ColorSupport::Ansi16)
            }
        }
        None => None,
    };

    match color_support {
        None => {
            vprtln!(Verbosity::Normal, "No colour support found for terminal");
        }
        Some(support) => {
            vprtln!(
                Verbosity::Normal,
                "{}",
                nu_resolve_style(MessageLevel::Warning).paint("Colored Warning message\n")
            );

            for variant in MessageStyle::iter() {
                let variant_string: &str = &variant.to_string();
                vprtln!(
                    Verbosity::Normal,
                    "My {} message",
                    variant.get_style().paint(variant_string)
                );
            }

            if matches!(support, ColorSupport::Xterm256) {
                vprtln!(Verbosity::Normal, "");
                XtermColor::iter().for_each(|variant| {
                    let color = variant.get_color();
                    vprtln!(Verbosity::Normal, "{}", color.paint(variant.to_string()));
                });
            }
        }
    }
}

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
