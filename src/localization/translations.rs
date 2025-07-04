/// Translation loading and management functionality
///
/// This module provides the Translator struct and functions to load
/// translations from embedded JSON files.
use std::collections::HashMap;

use serde_json::Value;

use super::{Language, LocalizationError, TranslationKey};

/// Translator struct that holds translations for a specific language
/// with fallback to English for missing translations
#[derive(Debug, Clone)]
pub struct Translator {
    language: Language,
    translations: HashMap<String, String>,
    fallback_translations: HashMap<String, String>,
}

impl Translator {
    /// Create a new translator for the specified language
    ///
    /// # Arguments
    ///
    /// * `language` - The target language for translations
    ///
    /// # Returns
    ///
    /// Returns a Result with the Translator or an error if loading fails
    pub fn new(language: Language) -> Result<Self, LocalizationError> {
        let translations = load_language_translations(language)?;
        let fallback_translations = if language != Language::English {
            load_language_translations(Language::English)?
        } else {
            translations.clone()
        };

        Ok(Self {
            language,
            translations,
            fallback_translations,
        })
    }

    /// Get the current language
    pub fn language(&self) -> Language {
        self.language
    }

    /// Translate a key to the target language
    ///
    /// # Arguments
    ///
    /// * `key` - The translation key to look up
    ///
    /// # Returns
    ///
    /// Returns the translated string, falling back to English if not found,
    /// or the key itself if no translation exists
    pub fn translate(&self, key: &TranslationKey) -> String {
        let key_str = key.key();

        // Try target language first
        if let Some(translation) = self.translations.get(&key_str) {
            return translation.clone();
        }

        // Fall back to English
        if let Some(fallback) = self.fallback_translations.get(&key_str) {
            return fallback.clone();
        }

        // If no translation found, return the key for debugging
        format!("[{key_str}]")
    }

    /// Translate with format arguments
    ///
    /// # Arguments
    ///
    /// * `key` - The translation key to look up
    /// * `args` - Arguments to substitute in the translation string
    ///
    /// # Returns
    ///
    /// Returns the translated and formatted string
    pub fn translate_with_args(&self, key: &TranslationKey, args: &[&str]) -> String {
        let translation = self.translate(key);

        // Simple substitution for {0}, {1}, etc.
        let mut result = translation;
        for (i, arg) in args.iter().enumerate() {
            result = result.replace(&format!("{{{i}}}"), arg);
        }

        result
    }

    /// Check if a translation exists for the given key
    pub fn has_translation(&self, key: &TranslationKey) -> bool {
        let key_str = key.key();
        self.translations.contains_key(&key_str)
            || self.fallback_translations.contains_key(&key_str)
    }

    /// Get all available translation keys
    pub fn available_keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.translations.keys().cloned().collect();
        for key in self.fallback_translations.keys() {
            if !keys.contains(key) {
                keys.push(key.clone());
            }
        }
        keys.sort();
        keys
    }
}

/// Load translations for a specific language from embedded JSON
fn load_language_translations(
    language: Language,
) -> Result<HashMap<String, String>, LocalizationError> {
    let json_content = match language {
        Language::English => include_str!("../locales/en.json"),
        Language::Spanish => include_str!("../locales/es.json"),
        Language::French => include_str!("../locales/fr.json"),
        Language::German => include_str!("../locales/de.json"),
        Language::Japanese => include_str!("../locales/ja.json"),
        Language::Chinese => include_str!("../locales/zh.json"),
    };

    parse_json_translations(json_content).map_err(|e| {
        LocalizationError::LoadError(format!("Failed to parse {}: {e}", language.code()))
    })
}

/// Parse JSON content into a flat HashMap of translation keys and values
fn parse_json_translations(
    json_content: &str,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let json: Value = serde_json::from_str(json_content)?;
    let mut translations = HashMap::new();
    flatten_json_object(&json, String::new(), &mut translations);
    Ok(translations)
}

/// Recursively flatten a JSON object into dot-notation keys
fn flatten_json_object(value: &Value, prefix: String, result: &mut HashMap<String, String>) {
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                let new_prefix = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                flatten_json_object(val, new_prefix, result);
            }
        },
        Value::String(s) => {
            result.insert(prefix, s.clone());
        },
        _ => {
            // For non-string values, convert to string
            result.insert(prefix, value.to_string().trim_matches('"').to_string());
        },
    }
}

/// Load translations for the given language, with fallback loading
///
/// # Arguments
///
/// * `language` - The language to load translations for
///
/// # Returns
///
/// Returns a Result with the loaded Translator
pub fn load_translations(language: Language) -> Result<Translator, LocalizationError> {
    Translator::new(language)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatten_json_object() {
        let json_str = r#"
        {
            "cli": {
                "app_description": "Test app",
                "commands": {
                    "help": "Show help"
                }
            },
            "export": {
                "types": "Types"
            }
        }
        "#;

        let json: Value = serde_json::from_str(json_str).unwrap();
        let mut result = HashMap::new();
        flatten_json_object(&json, String::new(), &mut result);

        assert_eq!(
            result.get("cli.app_description"),
            Some(&"Test app".to_string())
        );
        assert_eq!(
            result.get("cli.commands.help"),
            Some(&"Show help".to_string())
        );
        assert_eq!(result.get("export.types"), Some(&"Types".to_string()));
    }

    #[test]
    fn test_translator_translate() {
        // We can't easily test with the actual embedded files in unit tests,
        // but we can test the structure
        let translator = Translator {
            language: Language::English,
            translations: [
                ("export.types".to_string(), "Types".to_string()),
                ("common.yes".to_string(), "Yes".to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
            fallback_translations: HashMap::new(),
        };

        assert_eq!(translator.translate(&TranslationKey::Types), "Types");
        assert_eq!(translator.translate(&TranslationKey::Yes), "Yes");
    }

    #[test]
    fn test_translator_fallback() {
        let translator = Translator {
            language: Language::Spanish,
            translations: [("export.types".to_string(), "Tipos".to_string())]
                .iter()
                .cloned()
                .collect(),
            fallback_translations: [
                ("export.types".to_string(), "Types".to_string()),
                ("common.yes".to_string(), "Yes".to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
        };

        assert_eq!(translator.translate(&TranslationKey::Types), "Tipos");
        assert_eq!(translator.translate(&TranslationKey::Yes), "Yes"); // Fallback
    }

    #[test]
    fn test_translator_missing_key() {
        let translator = Translator {
            language: Language::English,
            translations: HashMap::new(),
            fallback_translations: HashMap::new(),
        };

        let result = translator.translate(&TranslationKey::Types);
        assert!(result.starts_with('[') && result.ends_with(']'));
    }

    #[test]
    fn test_translate_with_args() {
        let translator = Translator {
            language: Language::English,
            translations: [(
                "test.message".to_string(),
                "Hello {0}, you have {1} messages".to_string(),
            )]
            .iter()
            .cloned()
            .collect(),
            fallback_translations: HashMap::new(),
        };

        let result = translator.translate_with_args(
            &TranslationKey::Custom("test.message".to_string()),
            &["John", "5"],
        );
        assert_eq!(result, "Hello John, you have 5 messages");
    }
}
