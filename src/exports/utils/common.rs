/// Common utility functions shared across export formats
///
/// This module contains utility functions that are used by multiple
/// export formats to avoid code duplication and ensure consistency.
use indexmap::IndexMap;

use crate::localization::{TranslationKey, Translator};
use crate::parsing::BicepValue;

/// Helper function to format Yes/No values with or without emoji
///
/// # Arguments
///
/// * `value` - Boolean value to format
/// * `use_emoji` - Whether to use emoji symbols (✅/❌) or plain text (Yes/No)
/// * `translator` - The translator for localized text
///
/// # Returns
///
/// Formatted string with either emoji or plain text
pub fn format_yes_no(value: bool, use_emoji: bool, translator: &Translator) -> String {
    let text = if value {
        translator.translate(&TranslationKey::Yes)
    } else {
        translator.translate(&TranslationKey::No)
    };

    if use_emoji {
        let emoji = if value { "✅" } else { "❌" };
        format!("{emoji} {text}")
    } else {
        text
    }
}

/// Legacy wrapper for format_yes_no for backwards compatibility
/// This will be removed once all callers are updated
pub fn format_yes_no_legacy(value: bool, use_emoji: bool) -> String {
    match (value, use_emoji) {
        (true, true) => "✅ Yes".to_string(),
        (true, false) => "Yes".to_string(),
        (false, true) => "❌ No".to_string(),
        (false, false) => "No".to_string(),
    }
}

/// Generate metadata display for Markdown format
///
/// # Arguments
///
/// * `output` - Mutable reference to the output string
/// * `metadata` - Metadata map to display
pub fn generate_metadata_display_markdown(
    output: &mut String,
    metadata: &IndexMap<String, BicepValue>,
) {
    use super::formatting::escape_markdown;

    if !metadata.is_empty() {
        output.push_str("| Key | Value |\n");
        output.push_str("|-----|-------|\n");

        for (key, value) in metadata {
            let value_str = value.to_string();
            output.push_str(&format!(
                "| {} | {} |\n",
                escape_markdown(key),
                escape_markdown(&value_str)
            ));
        }
        output.push('\n');
    }
}

/// Generate metadata display for AsciiDoc format
///
/// # Arguments
///
/// * `output` - Mutable reference to the output string
/// * `metadata` - Metadata map to display
pub fn generate_metadata_display_asciidoc(
    output: &mut String,
    metadata: &IndexMap<String, BicepValue>,
) {
    use super::formatting::escape_asciidoc;

    if !metadata.is_empty() {
        output.push_str("[%autowidth,cols=\"h,1\",frame=none]\n");
        output.push_str("|===\n");

        for (key, value) in metadata {
            let value_str = value.to_string();
            output.push_str(&format!(
                "| {}\n| {}\n\n",
                escape_asciidoc(key),
                escape_asciidoc(&value_str)
            ));
        }
        output.push_str("|===\n\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_yes_no() {
        use crate::localization::{load_translations, Language};

        let translator = load_translations(Language::English).unwrap();
        assert_eq!(format_yes_no(true, true, &translator), "✅ Yes");
        assert_eq!(format_yes_no(true, false, &translator), "Yes");
        assert_eq!(format_yes_no(false, true, &translator), "❌ No");
        assert_eq!(format_yes_no(false, false, &translator), "No");

        let spanish_translator = load_translations(Language::Spanish).unwrap();
        assert_eq!(format_yes_no(true, false, &spanish_translator), "Sí");
        assert_eq!(format_yes_no(false, false, &spanish_translator), "No");
    }

    #[test]
    fn test_format_yes_no_legacy() {
        assert_eq!(format_yes_no_legacy(true, true), "✅ Yes");
        assert_eq!(format_yes_no_legacy(true, false), "Yes");
        assert_eq!(format_yes_no_legacy(false, true), "❌ No");
        assert_eq!(format_yes_no_legacy(false, false), "No");
    }

    #[test]
    fn test_generate_metadata_display_markdown_empty() {
        let mut output = String::new();
        let metadata = IndexMap::new();
        generate_metadata_display_markdown(&mut output, &metadata);
        assert!(output.is_empty());
    }

    #[test]
    fn test_generate_metadata_display_asciidoc_empty() {
        let mut output = String::new();
        let metadata = IndexMap::new();
        generate_metadata_display_asciidoc(&mut output, &metadata);
        assert!(output.is_empty());
    }
}
