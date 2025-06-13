/// Formatting utilities for Bicep types and values
///
/// This module provides functions for formatting Bicep types and values
/// consistently across different export formats, along with text escaping
/// functions for Markdown and AsciiDoc.
use crate::parsing::{BicepType, BicepValue};

/// Format a Bicep type with backticks for Markdown
///
/// # Arguments
///
/// * `bicep_type` - The BicepType to format
///
/// # Returns
///
/// String representation of the type wrapped in backticks
pub fn format_bicep_type_with_backticks(bicep_type: &BicepType) -> String {
    format!("`{}`", bicep_type)
}

/// Format a Bicep value as code with backticks
///
/// # Arguments
///
/// * `value` - The BicepValue to format
///
/// # Returns
///
/// String representation of the value wrapped in backticks
pub fn format_bicep_value_with_backticks(value: &BicepValue) -> String {
    format!("`{}`", value)
}

/// Formats a Bicep array value as a newline-separated list
///
/// Converts an array of `BicepValue`s into a single `String` where each element
/// is separated by a newline character. This is useful for rendering array values
/// in documentation or export formats that expect plain lists.
///
/// # Arguments
///
/// * `array` - Reference to an array of `BicepValue`
///
/// # Returns
///
/// * `String` containing the newline-separated list
pub fn format_bicep_array_as_list(array: &[BicepValue]) -> String {
    let mut formatted = '\n'.to_string();
    formatted.push_str(
        &array
            .iter()
            .map(|item| format!("- `{}`", item))
            .collect::<Vec<_>>()
            .join("\n"),
    );
    formatted
}

/// Escape special characters for Markdown
///
/// # Arguments
///
/// * `text` - Text to escape
///
/// # Returns
///
/// Escaped text safe for Markdown
pub fn escape_markdown(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace('\n', "  \n")
}

/// Escape special characters for AsciiDoc
///
/// # Arguments
///
/// * `text` - Text to escape
///
/// # Returns
///
/// Escaped text safe for AsciiDoc
pub fn escape_asciidoc(text: &str) -> String {
    text.replace('\\', "\\\\") // Must be first to avoid double-escaping
        .replace('|', "\\|")
        .replace('\n', " +\n")
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use super::*;

    #[test]
    fn test_format_bicep_type() {
        assert_eq!(BicepType::String.to_string(), "string");
        assert_eq!(BicepType::Int.to_string(), "int");
        assert_eq!(BicepType::Bool.to_string(), "bool");
        assert_eq!(
            BicepType::Array(Box::new(BicepType::String)).to_string(),
            "string[]"
        );
        assert_eq!(
            BicepType::CustomType("MyType".to_string()).to_string(),
            "MyType"
        );
        assert_eq!(
            BicepType::Union(vec!["A".to_string(), "B".to_string()]).to_string(),
            "A | B"
        );

        assert_eq!(BicepType::Object(None).to_string(), "object");

        let empty_props = IndexMap::new();
        assert_eq!(BicepType::Object(Some(empty_props)).to_string(), "object");
    }

    #[test]
    fn test_format_bicep_type_with_backticks() {
        assert_eq!(
            format_bicep_type_with_backticks(&BicepType::String),
            "`string`"
        );
        assert_eq!(format_bicep_type_with_backticks(&BicepType::Int), "`int`");
    }

    #[test]
    fn test_format_bicep_value() {
        assert_eq!(BicepValue::String("test".to_string()).to_string(), "test");
        assert_eq!(BicepValue::Int(42).to_string(), "42");
        assert_eq!(BicepValue::Bool(true).to_string(), "true");
        assert_eq!(
            BicepValue::Identifier("myVar".to_string()).to_string(),
            "${myVar}"
        );

        let arr = vec![BicepValue::Int(1), BicepValue::Int(2)];
        assert_eq!(BicepValue::Array(arr).to_string(), "[1, 2]");

        let mut obj = IndexMap::new();
        obj.insert("key1".to_string(), BicepValue::String("value1".to_string()));
        obj.insert("key2".to_string(), BicepValue::Int(42));
        let result = BicepValue::Object(obj).to_string();
        // Note: IndexMap preserves insertion order
        assert!(result.contains("key1: value1"));
        assert!(result.contains("key2: 42"));
    }

    #[test]
    fn test_format_bicep_value_with_multiline_string() {
        let multiline = "Line 1\nLine 2\nLine 3".to_string();
        let result = BicepValue::String(multiline).to_string();
        assert_eq!(result, "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_escape_markdown() {
        assert_eq!(escape_markdown("simple text"), "simple text");
        assert_eq!(escape_markdown("text with *bold*"), "text with *bold*");
        assert_eq!(escape_markdown("text with [link]"), "text with [link]");
        assert_eq!(
            escape_markdown("text | with | pipes"),
            "text \\| with \\| pipes"
        );
        assert_eq!(escape_markdown("line1\nline2"), "line1  \nline2");
        assert_eq!(escape_markdown("back\\slash"), "back\\\\slash");
    }

    #[test]
    fn test_escape_asciidoc() {
        assert_eq!(escape_asciidoc("simple text"), "simple text");
        assert_eq!(
            escape_asciidoc("text with |pipes|"),
            "text with \\|pipes\\|"
        );
    }
}
