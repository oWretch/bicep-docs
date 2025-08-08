/// Localization module for bicep-docs using rust-i18n
///
/// This module provides internationalization support for the CLI and generated documentation
/// using the rust-i18n crate.
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

/// Supported languages in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ValueEnum, Default)]
pub enum Language {
    #[serde(rename = "en")]
    #[default]
    English,
    #[serde(rename = "es")]
    Spanish,
    #[serde(rename = "fr")]
    French,
    #[serde(rename = "de")]
    German,
    #[serde(rename = "ja")]
    Japanese,
    #[serde(rename = "zh")]
    Chinese,
}

impl Language {
    /// Get the language code as a string
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Spanish => "es",
            Language::French => "fr",
            Language::German => "de",
            Language::Japanese => "ja",
            Language::Chinese => "zh",
        }
    }

    /// Get the language name in its native form
    pub fn name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Spanish => "Español",
            Language::French => "Français",
            Language::German => "Deutsch",
            Language::Japanese => "日本語",
            Language::Chinese => "中文",
        }
    }

    /// Parse a language code string to a Language enum
    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_lowercase().as_str() {
            "en" | "en-us" | "en-gb" => Some(Language::English),
            "es" | "es-es" | "es-mx" => Some(Language::Spanish),
            "fr" | "fr-fr" | "fr-ca" => Some(Language::French),
            "de" | "de-de" | "de-at" => Some(Language::German),
            "ja" | "ja-jp" => Some(Language::Japanese),
            "zh" | "zh-cn" | "zh-tw" => Some(Language::Chinese),
            _ => None,
        }
    }

    /// Get all supported languages
    pub fn all() -> Vec<Self> {
        vec![
            Language::English,
            Language::Spanish,
            Language::French,
            Language::German,
            Language::Japanese,
            Language::Chinese,
        ]
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

/// Initialize localization with the specified language
pub fn init_localization(language: Language) {
    rust_i18n::set_locale(language.code());
}

/// Detect the system locale and return the corresponding Language
pub fn detect_system_locale() -> Language {
    if let Some(locale) = sys_locale::get_locale() {
        Language::from_code(&locale).unwrap_or(Language::English)
    } else {
        Language::English
    }
}

/// Translate a key using rust-i18n's t! macro
/// This is a convenience function that wraps the t! macro
pub fn translate(key: &str) -> String {
    crate::t!(key).to_string()
}
