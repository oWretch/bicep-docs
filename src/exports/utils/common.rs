/// Common utility functions shared across export formats
///
/// This module contains utility functions that are used by multiple
/// export formats to avoid code duplication and ensure consistency.
use indexmap::IndexMap;

use crate::parsing::BicepValue;

/// Helper function to format Yes/No values with or without emoji
///
/// # Arguments
///
/// * `value` - Boolean value to format
/// * `use_emoji` - Whether to use emoji symbols (✅/❌) or plain text (Yes/No)
///
/// # Returns
///
/// Formatted string with either emoji or plain text
pub fn format_yes_no(value: bool, use_emoji: bool) -> String {
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
    use super::formatting::{escape_markdown, format_bicep_value};

    if !metadata.is_empty() {
        output.push_str("| Key | Value |\n");
        output.push_str("|-----|-------|\n");

        for (key, value) in metadata {
            let value_str = format_bicep_value(value);
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
    use super::formatting::{escape_asciidoc, format_bicep_value};

    if !metadata.is_empty() {
        output.push_str("[%autowidth,cols=\"h,1\",frame=none]\n");
        output.push_str("|===\n");

        for (key, value) in metadata {
            let value_str = format_bicep_value(value);
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
        assert_eq!(format_yes_no(true, true), "✅ Yes");
        assert_eq!(format_yes_no(true, false), "Yes");
        assert_eq!(format_yes_no(false, true), "❌ No");
        assert_eq!(format_yes_no(false, false), "No");
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
