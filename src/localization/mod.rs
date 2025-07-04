/// Localization module for bicep-docs
///
/// This module provides internationalization support for the CLI and generated documentation.
/// It uses a simple JSON-based translation system with fallback to English.
use std::error::Error;
use std::fmt;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

pub mod locale;
pub mod translations;

pub use locale::{detect_system_locale, parse_locale_string, Locale};
pub use translations::{load_translations, Translator};

/// Supported languages in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ValueEnum)]
pub enum Language {
    #[serde(rename = "en")]
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

    /// Get the language name in English
    pub fn name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Spanish => "Spanish",
            Language::French => "French",
            Language::German => "German",
            Language::Japanese => "Japanese",
            Language::Chinese => "Chinese",
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

impl Default for Language {
    fn default() -> Self {
        Language::English
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

/// Translation keys used throughout the application
#[derive(Debug, Clone)]
pub enum TranslationKey {
    // CLI help text
    AppDescription,
    AppAbout,
    VerboseHelp,
    QuietHelp,
    LogFormatHelp,
    LogFileHelp,
    LanguageHelp,

    // Command descriptions
    MarkdownCommandDesc,
    AsciidocCommandDesc,
    YamlCommandDesc,
    JsonCommandDesc,

    // Common export options
    InputHelp,
    OutputHelp,
    EmojiHelp,
    ExcludeEmptyHelp,
    CheckHelp,
    PrettyHelp,

    // Export section headers
    BicepTemplate,
    TargetScope,
    AdditionalMetadata,
    Imports,
    Types,
    Functions,
    Parameters,
    Variables,
    Resources,
    Modules,
    Outputs,

    // Table headers
    NamespaceHeader,
    VersionHeader,
    SourceHeader,
    SymbolsHeader,
    NameHeader,
    TypeHeader,
    RequiredHeader,
    DefaultHeader,
    DescriptionHeader,

    // Empty state messages
    NoImportsDefined,
    NoTypesDefined,
    NoFunctionsDefined,
    NoParametersDefined,
    NoVariablesDefined,
    NoResourcesDefined,
    NoModulesDefined,
    NoOutputsDefined,

    // Field labels
    MinimumValue,
    MaximumValue,
    MinimumLength,
    MaximumLength,
    AllowedValues,
    Discriminator,
    Sealed,

    // Boolean values
    Yes,
    No,

    // Error messages
    FileNotFound,
    ParseError,
    ExportError,
    InvalidLanguage,

    // Custom key for any string not covered above
    Custom(String),
}

impl TranslationKey {
    /// Get the key as a string for lookup in translation files
    pub fn key(&self) -> String {
        match self {
            // CLI help text
            TranslationKey::AppDescription => "cli.app_description".to_string(),
            TranslationKey::AppAbout => "cli.app_about".to_string(),
            TranslationKey::VerboseHelp => "cli.verbose_help".to_string(),
            TranslationKey::QuietHelp => "cli.quiet_help".to_string(),
            TranslationKey::LogFormatHelp => "cli.log_format_help".to_string(),
            TranslationKey::LogFileHelp => "cli.log_file_help".to_string(),
            TranslationKey::LanguageHelp => "cli.language_help".to_string(),

            // Command descriptions
            TranslationKey::MarkdownCommandDesc => "cli.markdown_command_desc".to_string(),
            TranslationKey::AsciidocCommandDesc => "cli.asciidoc_command_desc".to_string(),
            TranslationKey::YamlCommandDesc => "cli.yaml_command_desc".to_string(),
            TranslationKey::JsonCommandDesc => "cli.json_command_desc".to_string(),

            // Common export options
            TranslationKey::InputHelp => "cli.input_help".to_string(),
            TranslationKey::OutputHelp => "cli.output_help".to_string(),
            TranslationKey::EmojiHelp => "cli.emoji_help".to_string(),
            TranslationKey::ExcludeEmptyHelp => "cli.exclude_empty_help".to_string(),
            TranslationKey::CheckHelp => "cli.check_help".to_string(),
            TranslationKey::PrettyHelp => "cli.pretty_help".to_string(),

            // Export section headers
            TranslationKey::BicepTemplate => "export.bicep_template".to_string(),
            TranslationKey::TargetScope => "export.target_scope".to_string(),
            TranslationKey::AdditionalMetadata => "export.additional_metadata".to_string(),
            TranslationKey::Imports => "export.imports".to_string(),
            TranslationKey::Types => "export.types".to_string(),
            TranslationKey::Functions => "export.functions".to_string(),
            TranslationKey::Parameters => "export.parameters".to_string(),
            TranslationKey::Variables => "export.variables".to_string(),
            TranslationKey::Resources => "export.resources".to_string(),
            TranslationKey::Modules => "export.modules".to_string(),
            TranslationKey::Outputs => "export.outputs".to_string(),

            // Table headers
            TranslationKey::NamespaceHeader => "export.namespace_header".to_string(),
            TranslationKey::VersionHeader => "export.version_header".to_string(),
            TranslationKey::SourceHeader => "export.source_header".to_string(),
            TranslationKey::SymbolsHeader => "export.symbols_header".to_string(),
            TranslationKey::NameHeader => "export.name_header".to_string(),
            TranslationKey::TypeHeader => "export.type_header".to_string(),
            TranslationKey::RequiredHeader => "export.required_header".to_string(),
            TranslationKey::DefaultHeader => "export.default_header".to_string(),
            TranslationKey::DescriptionHeader => "export.description_header".to_string(),

            // Empty state messages
            TranslationKey::NoImportsDefined => "export.no_imports_defined".to_string(),
            TranslationKey::NoTypesDefined => "export.no_types_defined".to_string(),
            TranslationKey::NoFunctionsDefined => "export.no_functions_defined".to_string(),
            TranslationKey::NoParametersDefined => "export.no_parameters_defined".to_string(),
            TranslationKey::NoVariablesDefined => "export.no_variables_defined".to_string(),
            TranslationKey::NoResourcesDefined => "export.no_resources_defined".to_string(),
            TranslationKey::NoModulesDefined => "export.no_modules_defined".to_string(),
            TranslationKey::NoOutputsDefined => "export.no_outputs_defined".to_string(),

            // Field labels
            TranslationKey::MinimumValue => "export.minimum_value".to_string(),
            TranslationKey::MaximumValue => "export.maximum_value".to_string(),
            TranslationKey::MinimumLength => "export.minimum_length".to_string(),
            TranslationKey::MaximumLength => "export.maximum_length".to_string(),
            TranslationKey::AllowedValues => "export.allowed_values".to_string(),
            TranslationKey::Discriminator => "export.discriminator".to_string(),
            TranslationKey::Sealed => "export.sealed".to_string(),

            // Boolean values
            TranslationKey::Yes => "common.yes".to_string(),
            TranslationKey::No => "common.no".to_string(),

            // Error messages
            TranslationKey::FileNotFound => "error.file_not_found".to_string(),
            TranslationKey::ParseError => "error.parse_error".to_string(),
            TranslationKey::ExportError => "error.export_error".to_string(),
            TranslationKey::InvalidLanguage => "error.invalid_language".to_string(),

            // Custom key
            TranslationKey::Custom(key) => key.clone(),
        }
    }
}

/// Localization error types
#[derive(Debug)]
pub enum LocalizationError {
    TranslationNotFound(String),
    InvalidLanguage(String),
    LoadError(String),
}

impl fmt::Display for LocalizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocalizationError::TranslationNotFound(key) => {
                write!(f, "Translation not found for key: {key}")
            },
            LocalizationError::InvalidLanguage(lang) => {
                write!(f, "Invalid language: {lang}")
            },
            LocalizationError::LoadError(msg) => {
                write!(f, "Failed to load translations: {msg}")
            },
        }
    }
}

impl Error for LocalizationError {}
