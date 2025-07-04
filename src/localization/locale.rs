/// Locale detection and parsing functionality
///
/// This module provides functionality to detect the system locale
/// and parse locale strings into Language enums.
use super::{Language, LocalizationError};

/// Represents a locale with language and optional region
#[derive(Debug, Clone, PartialEq)]
pub struct Locale {
    pub language: Language,
    pub region: Option<String>,
}

impl Locale {
    /// Create a new locale with just a language
    pub fn new(language: Language) -> Self {
        Self {
            language,
            region: None,
        }
    }

    /// Create a new locale with language and region
    pub fn with_region(language: Language, region: String) -> Self {
        Self {
            language,
            region: Some(region),
        }
    }
}

impl Default for Locale {
    fn default() -> Self {
        Self::new(Language::English)
    }
}

/// Detect the system locale using the sys-locale crate
///
/// # Returns
///
/// Returns the detected locale, falling back to English if detection fails
pub fn detect_system_locale() -> Locale {
    match sys_locale::get_locale() {
        Some(locale_str) => parse_locale_string(&locale_str).unwrap_or_default(),
        None => Locale::default(),
    }
}

/// Parse a locale string (e.g., "en_US", "fr-FR") into a Locale struct
///
/// # Arguments
///
/// * `locale_str` - The locale string to parse
///
/// # Returns
///
/// Returns a Result with the parsed Locale or an error if parsing fails
pub fn parse_locale_string(locale_str: &str) -> Result<Locale, LocalizationError> {
    // Handle different separator formats (_, -, .)
    let normalized = locale_str.replace(['_', '.'], "-").to_lowercase();

    let parts: Vec<&str> = normalized.split('-').collect();

    if parts.is_empty() {
        return Err(LocalizationError::InvalidLanguage(locale_str.to_string()));
    }

    let language_code = parts[0];
    let language = Language::from_code(language_code)
        .ok_or_else(|| LocalizationError::InvalidLanguage(locale_str.to_string()))?;

    let region = if parts.len() > 1 {
        Some(parts[1].to_uppercase())
    } else {
        None
    };

    Ok(Locale { language, region })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_locale_string_simple() {
        let locale = parse_locale_string("en").unwrap();
        assert_eq!(locale.language, Language::English);
        assert_eq!(locale.region, None);
    }

    #[test]
    fn test_parse_locale_string_with_region() {
        let locale = parse_locale_string("en_US").unwrap();
        assert_eq!(locale.language, Language::English);
        assert_eq!(locale.region, Some("US".to_string()));
    }

    #[test]
    fn test_parse_locale_string_with_dash() {
        let locale = parse_locale_string("fr-FR").unwrap();
        assert_eq!(locale.language, Language::French);
        assert_eq!(locale.region, Some("FR".to_string()));
    }

    #[test]
    fn test_parse_locale_string_with_dot() {
        let locale = parse_locale_string("de.DE").unwrap();
        assert_eq!(locale.language, Language::German);
        assert_eq!(locale.region, Some("DE".to_string()));
    }

    #[test]
    fn test_parse_locale_string_case_insensitive() {
        let locale = parse_locale_string("ES-mx").unwrap();
        assert_eq!(locale.language, Language::Spanish);
        assert_eq!(locale.region, Some("MX".to_string()));
    }

    #[test]
    fn test_parse_locale_string_invalid() {
        let result = parse_locale_string("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_system_locale() {
        // This test just ensures the function doesn't panic
        let locale = detect_system_locale();
        // Should always return at least English as fallback
        assert!(matches!(
            locale.language,
            Language::English
                | Language::Spanish
                | Language::French
                | Language::German
                | Language::Japanese
                | Language::Chinese
        ));
    }

    #[test]
    fn test_locale_new() {
        let locale = Locale::new(Language::Japanese);
        assert_eq!(locale.language, Language::Japanese);
        assert_eq!(locale.region, None);
    }

    #[test]
    fn test_locale_with_region() {
        let locale = Locale::with_region(Language::Chinese, "CN".to_string());
        assert_eq!(locale.language, Language::Chinese);
        assert_eq!(locale.region, Some("CN".to_string()));
    }
}
