// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::path::PathBuf;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

// One income or expense saved in the ledger.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub(crate) struct Data {
    // Name shown in the transaction history.
    pub(crate) d_name: String,
    // Usually `INCOME` or `EXPENSE`.
    pub(crate) d_type: String,
    // Whole-number amount stored in the ledger.
    pub(crate) d_amount: i128,
    // Currency used for this amount.
    pub(crate) d_currency: Currency,
    // Date shown beside the transaction.
    pub(crate) d_date: String,
}

// All transaction rows stored in one data file.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub(crate) struct VData {
    // The full list of saved transactions.
    pub(crate) v_data: Vec<Data>,
}

// App state for the ledger, form, and display settings.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct RScashM {
    // Transactions loaded from the active data file.
    pub(crate) data: VData,
    // File used to read and save the ledger.
    pub(crate) path: PathBuf,
    // Short message shown below the app header.
    pub(crate) status: String,
    // Amount currently typed into the form.
    pub(crate) d_ui_amount: String,
    // Name currently typed into the form.
    pub(crate) d_ui_name: String,
    // Type currently selected in the form.
    pub(crate) d_ui_type: String,
    // Currency currently selected in the form.
    pub(crate) d_ui_currency: String,
    // Date currently typed into the form.
    pub(crate) d_ui_date: String,
    // Startup read state; kept in memory only.
    #[serde(skip)]
    pub(crate) read_state: FileReadLoop,
    // Currency used for totals; different currencies stay separate.
    #[serde(skip)]
    pub(crate) balance_currency: String,
    // Whether the Settings panel is open.
    #[serde(skip)]
    pub(crate) show_settings_panel: bool,
    // Active built-in color palette.
    #[serde(skip)]
    pub(crate) app_theme: AppTheme,
    // Custom colors and wallpaper path.
    #[serde(skip)]
    pub(crate) appearance: AppearanceSettings,
    // Values currently shown in the background RGB inputs.
    #[serde(skip)]
    pub(crate) background_rgb_inputs: [String; 3],
    // Values currently shown in the text RGB inputs.
    #[serde(skip)]
    pub(crate) text_rgb_inputs: [String; 3],
}

// Built-in palettes available in Settings.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum AppTheme {
    #[default]
    // Dark background with light text.
    Dark,
    // Light background with dark text.
    Light,
}

// RGB color with one byte per channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub(crate) struct RgbColor {
    // Red channel, from 0 to 255.
    pub(crate) red: u8,
    // Green channel, from 0 to 255.
    pub(crate) green: u8,
    // Blue channel, from 0 to 255.
    pub(crate) blue: u8,
}

impl RgbColor {
    // Return the builtin page background for this theme.
    pub(crate) fn default_background(theme: AppTheme) -> Self {
        match theme {
            AppTheme::Dark => Self {
                red: 0,
                green: 0,
                blue: 0,
            },
            AppTheme::Light => Self {
                red: 245,
                green: 247,
                blue: 242,
            },
        }
    }

    // Return the built-in text color for this theme.
    pub(crate) fn default_text(theme: AppTheme) -> Self {
        match theme {
            AppTheme::Dark => Self {
                red: 255,
                green: 255,
                blue: 255,
            },
            AppTheme::Light => Self {
                red: 26,
                green: 33,
                blue: 31,
            },
        }
    }

    // Return channels in red, green, blue order.
    pub(crate) fn channels(self) -> [u8; 3] {
        [self.red, self.green, self.blue]
    }
}

// Optional custom colors and wallpaper chosen by the user.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct AppearanceSettings {
    // Custom page background, if set.
    pub(crate) background: Option<RgbColor>,
    // Custom text color, if set.
    pub(crate) text: Option<RgbColor>,
    // Wallpaper file shown behind the main page.
    pub(crate) wallpaper_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub(crate) struct AppConfig {
    pub(crate) ledger_path: Option<PathBuf>,
    pub(crate) theme: AppTheme,
    pub(crate) appearance: AppearanceSettings,
}

// Tells an RGB input which appearance color to edit.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ColorTarget {
    // Page background color.
    Background,
    // Main text color.
    Text,
}

// One channel in an RGB color.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ColorChannel {
    // Red channel.
    Red,
    // Green channel.
    Green,
    // Blue channel.
    Blue,
}

impl ColorChannel {
    // Return this channel's position in an RGB array.
    pub(crate) fn index(self) -> usize {
        match self {
            Self::Red => 0,
            Self::Green => 1,
            Self::Blue => 2,
        }
    }
}

// Events the view sends to update app state.
#[derive(Debug, Clone)]
pub(crate) enum Message {
    // Save a new transaction.
    TransmitData,
    // Remove one transaction by its list position.
    RemoveData(usize),
    // Remove every transaction.
    RemoveAllData,
    // Update the transaction name input.
    NameChanged(String),
    // Update the transaction amount input.
    AmountChanged(String),
    // Change the transaction currency.
    CurrencyChanged(String),
    // Update the transaction date input.
    DateChanged(String),
    // Change the transaction type.
    TypeChanged(String),
    // Retry reading the active data file.
    FileReadTick,
    // Change the currency used by the balance totals.
    BalanceCurrencyChanged(String),
    // Show or hide Settings.
    ToggleSettingsPanel,
    // Choose a folder for the ledger file.
    SelectDataFolder,
    // Choose a different ledger file.
    ChooseFile,
    // Switch to a built-in color palette.
    SetTheme(AppTheme),
    // Update one RGB channel from the Settings input.
    UpdateRgbChannel {
        // Color being edited.
        target: ColorTarget,
        // RGB channel being changed.
        channel: ColorChannel,
        // New value typed by the user.
        value: String,
    },
    // Open the wallpaper picker.
    ChooseWallpaper,
    // Return the selected wallpaper path from the picker.
    WallpaperChosen(Option<PathBuf>),
    // Remove the current wallpaper.
    ClearWallpaper,
}

// State of the initial data-file read.
#[derive(Debug, Clone, Default)]
pub(crate) enum FileReadLoop {
    #[default]
    // No read has started yet.
    Idle,
    // Waiting for the file to become readable.
    Waiting,
    // The file was read successfully.
    Loaded(VData),
    // The last read attempt failed.
    Error(String),
}

// Currency codes supported by the form and balance view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub(crate) enum Currency {
    // A
    AED, // UAE Dirham
    AFN, // Afghan Afghani
    ALL, // Albanian Lek
    AMD, // Armenian Dram
    AOA, // Angolan Kwanza
    ARS, // Argentine Peso
    AUD, // Australian Dollar
    AWG, // Aruban Florin
    AZN, // Azerbaijani Manat

    // B
    BAM, // Bosnia Convertible Mark
    BBD, // Barbados Dollar
    BDT, // Bangladeshi Taka
    BHD, // Bahraini Dinar
    BIF, // Burundian Franc
    BMD, // Bermudian Dollar
    BND, // Brunei Dollar
    BOB, // Bolivian Boliviano
    BRL, // Brazilian Real
    BSD, // Bahamian Dollar
    BTN, // Bhutanese Ngultrum
    BWP, // Botswana Pula
    BYN, // Belarusian Ruble
    BZD, // Belize Dollar

    // C
    CAD, // Canadian Dollar
    CDF, // Congolese Franc
    CHF, // Swiss Franc
    CLP, // Chilean Peso
    CNY, // Chinese Yuan
    COP, // Colombian Peso
    CRC, // Costa Rican Colon
    CUP, // Cuban Peso
    CVE, // Cape Verdean Escudo
    CZK, // Czech Koruna

    // D
    DJF, // Djiboutian Franc
    DKK, // Danish Krone
    DOP, // Dominican Peso
    DZD, // Algerian Dinar

    // E
    EGP, // Egyptian Pound
    ERN, // Eritrean Nakfa
    ETB, // Ethiopian Birr
    EUR, // Euro

    // F
    FJD, // Fijian Dollar
    FKP, // Falkland Islands Pound

    // G
    GBP, // Pound Sterling
    GEL, // Georgian Lari
    GHS, // Ghanaian Cedi
    GIP, // Gibraltar Pound
    GMD, // Gambian Dalasi
    GNF, // Guinean Franc
    GTQ, // Guatemalan Quetzal
    GYD, // Guyanese Dollar

    // H
    HKD, // Hong Kong Dollar
    HNL, // Honduran Lempira
    HTG, // Haitian Gourde
    HUF, // Hungarian Forint

    // I
    IDR, // Indonesian Rupiah
    ILS, // Israeli New Shekel
    INR, // Indian Rupee
    IQD, // Iraqi Dinar
    IRR, // Iranian Rial
    ISK, // Icelandic Krona

    // J
    JMD, // Jamaican Dollar
    JOD, // Jordanian Dinar
    JPY, // Japanese Yen

    // K
    KES, // Kenyan Shilling
    KGS, // Kyrgyzstani Som
    KHR, // Cambodian Riel
    KMF, // Comorian Franc
    KPW, // North Korean Won
    KRW, // South Korean Won
    KWD, // Kuwaiti Dinar
    KYD, // Cayman Islands Dollar
    KZT, // Kazakhstani Tenge

    // L
    LAK, // Lao Kip
    LBP, // Lebanese Pound
    LKR, // Sri Lankan Rupee
    LRD, // Liberian Dollar
    LSL, // Lesotho Loti
    LYD, // Libyan Dinar

    // M
    MAD, // Moroccan Dirham
    MDL, // Moldovan Leu
    MGA, // Malagasy Ariary
    MKD, // Macedonian Denar
    MMK, // Myanmar Kyat
    MNT, // Mongolian Tugrik
    MOP, // Macanese Pataca
    MRU, // Mauritanian Ouguiya
    MUR, // Mauritian Rupee
    MVR, // Maldivian Rufiyaa
    MWK, // Malawian Kwacha
    MXN, // Mexican Peso
    MYR, // Malaysian Ringgit
    MZN, // Mozambican Metical

    // N
    NAD, // Namibian Dollar
    NGN, // Nigerian Naira
    NIO, // Nicaraguan Cordoba
    NOK, // Norwegian Krone
    NPR, // Nepalese Rupee
    NZD, // New Zealand Dollar

    // O
    OMR, // Omani Rial

    // P
    PAB, // Panamanian Balboa
    PEN, // Peruvian Sol
    PGK, // Papua New Guinean Kina
    PHP, // Philippine Peso
    PKR, // Pakistani Rupee
    PLN, // Polish Zloty
    PYG, // Paraguayan Guarani

    // Q
    QAR, // Qatari Riyal

    // R
    RON, // Romanian Leu
    RSD, // Serbian Dinar
    RUB, // Russian Ruble
    RWF, // Rwandan Franc

    // S
    SAR, // Saudi Riyal
    SBD, // Solomon Islands Dollar
    SCR, // Seychellois Rupee
    SDG, // Sudanese Pound
    SEK, // Swedish Krona
    SGD, // Singapore Dollar
    SHP, // Saint Helena Pound
    SLE, // Sierra Leonean Leone
    SOS, // Somali Shilling
    SRD, // Surinamese Dollar
    SSP, // South Sudanese Pound
    STN, // Sao Tome and Principe Dobra
    SVC, // Salvadoran Colon
    SYP, // Syrian Pound
    SZL, // Eswatini Lilangeni

    // T
    THB, // Thai Baht
    TJS, // Tajikistani Somoni
    TMT, // Turkmenistani Manat
    TND, // Tunisian Dinar
    TOP, // Tongan Pa'anga
    TRY, // Turkish Lira
    TTD, // Trinidad and Tobago Dollar
    TWD, // New Taiwan Dollar
    TZS, // Tanzanian Shilling

    // U
    UAH, // Ukrainian Hryvnia
    UGX, // Ugandan Shilling
    #[default]
    USD, // US Dollar
    UYU, // Uruguayan Peso
    UZS, // Uzbekistani Sum

    // V
    VED, // Venezuelan Digital Bolivar
    VES, // Venezuelan Sovereign Bolivar
    VND, // Vietnamese Dong
    VUV, // Vanuatu Vatu

    // W
    WST, // Samoan Tala

    // X
    XAF, // Central African CFA Franc
    XCD, // East Caribbean Dollar
    XCG, // Caribbean Guilder
    XOF, // West African CFA Franc
    XPF, // CFP Franc

    // Y
    YER, // Yemeni Rial

    // Z
    ZAR, // South African Rand
    ZMW, // Zambian Kwacha
    ZWG, // Zimbabwe Gold
}

impl FromStr for Currency {
    type Err = String;

    // Match a currency code such as `USD` to its enum value.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim();
        match normalized {
            "AED" => Ok(Self::AED),
            "AFN" => Ok(Self::AFN),
            "ALL" => Ok(Self::ALL),
            "AMD" => Ok(Self::AMD),
            "AOA" => Ok(Self::AOA),
            "ARS" => Ok(Self::ARS),
            "AUD" => Ok(Self::AUD),
            "AWG" => Ok(Self::AWG),
            "AZN" => Ok(Self::AZN),
            "BAM" => Ok(Self::BAM),
            "BBD" => Ok(Self::BBD),
            "BDT" => Ok(Self::BDT),
            "BHD" => Ok(Self::BHD),
            "BIF" => Ok(Self::BIF),
            "BMD" => Ok(Self::BMD),
            "BND" => Ok(Self::BND),
            "BOB" => Ok(Self::BOB),
            "BRL" => Ok(Self::BRL),
            "BSD" => Ok(Self::BSD),
            "BTN" => Ok(Self::BTN),
            "BWP" => Ok(Self::BWP),
            "BYN" => Ok(Self::BYN),
            "BZD" => Ok(Self::BZD),
            "CAD" => Ok(Self::CAD),
            "CDF" => Ok(Self::CDF),
            "CHF" => Ok(Self::CHF),
            "CLP" => Ok(Self::CLP),
            "CNY" => Ok(Self::CNY),
            "COP" => Ok(Self::COP),
            "CRC" => Ok(Self::CRC),
            "CUP" => Ok(Self::CUP),
            "CVE" => Ok(Self::CVE),
            "CZK" => Ok(Self::CZK),
            "DJF" => Ok(Self::DJF),
            "DKK" => Ok(Self::DKK),
            "DOP" => Ok(Self::DOP),
            "DZD" => Ok(Self::DZD),
            "EGP" => Ok(Self::EGP),
            "ERN" => Ok(Self::ERN),
            "ETB" => Ok(Self::ETB),
            "EUR" => Ok(Self::EUR),
            "FJD" => Ok(Self::FJD),
            "FKP" => Ok(Self::FKP),
            "GBP" => Ok(Self::GBP),
            "GEL" => Ok(Self::GEL),
            "GHS" => Ok(Self::GHS),
            "GIP" => Ok(Self::GIP),
            "GMD" => Ok(Self::GMD),
            "GNF" => Ok(Self::GNF),
            "GTQ" => Ok(Self::GTQ),
            "GYD" => Ok(Self::GYD),
            "HKD" => Ok(Self::HKD),
            "HNL" => Ok(Self::HNL),
            "HTG" => Ok(Self::HTG),
            "HUF" => Ok(Self::HUF),
            "IDR" => Ok(Self::IDR),
            "ILS" => Ok(Self::ILS),
            "INR" => Ok(Self::INR),
            "IQD" => Ok(Self::IQD),
            "IRR" => Ok(Self::IRR),
            "ISK" => Ok(Self::ISK),
            "JMD" => Ok(Self::JMD),
            "JOD" => Ok(Self::JOD),
            "JPY" => Ok(Self::JPY),
            "KES" => Ok(Self::KES),
            "KGS" => Ok(Self::KGS),
            "KHR" => Ok(Self::KHR),
            "KMF" => Ok(Self::KMF),
            "KPW" => Ok(Self::KPW),
            "KRW" => Ok(Self::KRW),
            "KWD" => Ok(Self::KWD),
            "KYD" => Ok(Self::KYD),
            "KZT" => Ok(Self::KZT),
            "LAK" => Ok(Self::LAK),
            "LBP" => Ok(Self::LBP),
            "LKR" => Ok(Self::LKR),
            "LRD" => Ok(Self::LRD),
            "LSL" => Ok(Self::LSL),
            "LYD" => Ok(Self::LYD),
            "MAD" => Ok(Self::MAD),
            "MDL" => Ok(Self::MDL),
            "MGA" => Ok(Self::MGA),
            "MKD" => Ok(Self::MKD),
            "MMK" => Ok(Self::MMK),
            "MNT" => Ok(Self::MNT),
            "MOP" => Ok(Self::MOP),
            "MRU" => Ok(Self::MRU),
            "MUR" => Ok(Self::MUR),
            "MVR" => Ok(Self::MVR),
            "MWK" => Ok(Self::MWK),
            "MXN" => Ok(Self::MXN),
            "MYR" => Ok(Self::MYR),
            "MZN" => Ok(Self::MZN),
            "NAD" => Ok(Self::NAD),
            "NGN" => Ok(Self::NGN),
            "NIO" => Ok(Self::NIO),
            "NOK" => Ok(Self::NOK),
            "NPR" => Ok(Self::NPR),
            "NZD" => Ok(Self::NZD),
            "OMR" => Ok(Self::OMR),
            "PAB" => Ok(Self::PAB),
            "PEN" => Ok(Self::PEN),
            "PGK" => Ok(Self::PGK),
            "PHP" => Ok(Self::PHP),
            "PKR" => Ok(Self::PKR),
            "PLN" => Ok(Self::PLN),
            "PYG" => Ok(Self::PYG),
            "QAR" => Ok(Self::QAR),
            "RON" => Ok(Self::RON),
            "RSD" => Ok(Self::RSD),
            "RUB" => Ok(Self::RUB),
            "RWF" => Ok(Self::RWF),
            "SAR" => Ok(Self::SAR),
            "SBD" => Ok(Self::SBD),
            "SCR" => Ok(Self::SCR),
            "SDG" => Ok(Self::SDG),
            "SEK" => Ok(Self::SEK),
            "SGD" => Ok(Self::SGD),
            "SHP" => Ok(Self::SHP),
            "SLE" => Ok(Self::SLE),
            "SOS" => Ok(Self::SOS),
            "SRD" => Ok(Self::SRD),
            "SSP" => Ok(Self::SSP),
            "STN" => Ok(Self::STN),
            "SVC" => Ok(Self::SVC),
            "SYP" => Ok(Self::SYP),
            "SZL" => Ok(Self::SZL),
            "THB" => Ok(Self::THB),
            "TJS" => Ok(Self::TJS),
            "TMT" => Ok(Self::TMT),
            "TND" => Ok(Self::TND),
            "TOP" => Ok(Self::TOP),
            "TRY" => Ok(Self::TRY),
            "TTD" => Ok(Self::TTD),
            "TWD" => Ok(Self::TWD),
            "TZS" => Ok(Self::TZS),
            "UAH" => Ok(Self::UAH),
            "UGX" => Ok(Self::UGX),
            "USD" => Ok(Self::USD),
            "UYU" => Ok(Self::UYU),
            "UZS" => Ok(Self::UZS),
            "VED" => Ok(Self::VED),
            "VES" => Ok(Self::VES),
            "VND" => Ok(Self::VND),
            "VUV" => Ok(Self::VUV),
            "WST" => Ok(Self::WST),
            "XAF" => Ok(Self::XAF),
            "XCD" => Ok(Self::XCD),
            "XCG" => Ok(Self::XCG),
            "XOF" => Ok(Self::XOF),
            "XPF" => Ok(Self::XPF),
            "YER" => Ok(Self::YER),
            "ZAR" => Ok(Self::ZAR),
            "ZMW" => Ok(Self::ZMW),
            "ZWG" => Ok(Self::ZWG),
            _ => Err(format!("Unsupported currency code: {normalized}")),
        }
    }
}
