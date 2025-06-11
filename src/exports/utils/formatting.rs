/// Formatting utilities for Bicep types and values
///
/// This module provides functions for formatting Bicep types and values
/// consistently across different export formats, along with text escaping
/// functions for Markdown and AsciiDoc.
use crate::parsing::{BicepType, BicepValue};

/// Format a Bicep type as a string representation
///
/// # Arguments
///
/// * `bicep_type` - The BicepType to format
///
/// # Returns
///
/// String representation of the type
pub fn format_bicep_type(bicep_type: &BicepType) -> String {
    match bicep_type {
        BicepType::Array(inner) => format!("{}[]", format_bicep_type(inner)),
        BicepType::Bool => "bool".to_string(),
        BicepType::Int => "int".to_string(),
        BicepType::String => "string".to_string(),
        BicepType::Object(properties) => {
            if let Some(_props) = properties {
                // Always return "object" for objects with properties
                // Individual properties will be documented separately
                "object".to_string()
            } else {
                "object".to_string()
            }
        },
        BicepType::Union(types) => types.join(" | "),
        BicepType::CustomType(name) => name.clone(),
    }
}

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
    format!("`{}`", format_bicep_type(bicep_type))
}

/// Format a Bicep type with monospace formatting for AsciiDoc
///
/// # Arguments
///
/// * `bicep_type` - The BicepType to format
///
/// # Returns
///
/// String representation of the type with AsciiDoc monospace formatting
pub fn format_bicep_type_with_monospace(bicep_type: &BicepType) -> String {
    format!("`{}`", format_bicep_type(bicep_type))
}

/// Format a Bicep value as a string representation
///
/// # Arguments
///
/// * `value` - The BicepValue to format
///
/// # Returns
///
/// String representation of the value
pub fn format_bicep_value(value: &BicepValue) -> String {
    match value {
        BicepValue::String(s) => {
            s.clone() // Unquoted strings (both single-line and multiline)
        },
        BicepValue::Int(i) => i.to_string(),
        BicepValue::Bool(b) => b.to_string(),
        BicepValue::Array(arr) => {
            let items: Vec<String> = arr.iter().map(format_bicep_value).collect();
            format!("[{}]", items.join(", "))
        },
        BicepValue::Object(obj) => {
            if obj.is_empty() {
                "{}".to_string()
            } else {
                let items: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, format_bicep_value(v)))
                    .collect();
                format!("{{ {} }}", items.join(", "))
            }
        },
        BicepValue::Identifier(name) => format!("${{{}}}", name),
    }
}

/// Format a Bicep value as code with backticks for Markdown
///
/// # Arguments
///
/// * `value` - The BicepValue to format
///
/// # Returns
///
/// String representation of the value wrapped in backticks
pub fn format_bicep_value_as_code(value: &BicepValue) -> String {
    format!("`{}`", format_bicep_value(value))
}

/// Format a Bicep value for AsciiDoc (unquoted strings, plain multiline)
///
/// # Arguments
///
/// * `value` - The BicepValue to format
///
/// # Returns
///
/// String representation of the value for AsciiDoc format
pub fn format_bicep_value_asciidoc(value: &BicepValue) -> String {
    match value {
        BicepValue::String(s) => {
            s.clone() // Unquoted strings for AsciiDoc (both single-line and multiline)
        },
        BicepValue::Int(i) => i.to_string(),
        BicepValue::Bool(b) => b.to_string(),
        BicepValue::Array(arr) => {
            let items: Vec<String> = arr.iter().map(format_bicep_value_asciidoc).collect();
            format!("[{}]", items.join(", "))
        },
        BicepValue::Object(obj) => {
            if obj.is_empty() {
                "{}".to_string()
            } else {
                let items: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, format_bicep_value_asciidoc(v)))
                    .collect();
                format!("{{ {} }}", items.join(", "))
            }
        },
        BicepValue::Identifier(name) => format!("${{{}}}", name),
    }
}

/// Format a Bicep type for AsciiDoc (escaped pipes in unions)
///
/// # Arguments
///
/// * `bicep_type` - The BicepType to format
///
/// # Returns
///
/// String representation of the type for AsciiDoc format
pub fn format_bicep_type_asciidoc(bicep_type: &BicepType) -> String {
    match bicep_type {
        BicepType::Array(inner) => format!("{}[]", format_bicep_type_asciidoc(inner)),
        BicepType::Bool => "bool".to_string(),
        BicepType::Int => "int".to_string(),
        BicepType::String => "string".to_string(),
        BicepType::Object(properties) => {
            if let Some(props) = properties {
                if props.is_empty() {
                    "object".to_string()
                } else {
                    // Always return "object" for objects with properties
                    // Individual properties will be documented separately
                    "object".to_string()
                }
            } else {
                "object".to_string()
            }
        },
        BicepType::Union(types) => {
            // Use escaped pipes for AsciiDoc tables
            types.join(" \\| ")
        },
        BicepType::CustomType(name) => name.clone(),
    }
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
        .replace('*', "\\*")
        .replace('_', "\\_")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('(', "\\(")
        .replace(')', "\\)")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('&', "&amp;")
        .replace('|', "\\|")
        .replace('`', "\\`")
        .replace('#', "\\#")
        .replace('+', "\\+")
        .replace('-', "\\-")
        .replace('.', "\\.")
        .replace('!', "\\!")
        .replace('{', "\\{")
        .replace('}', "\\}")
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
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('*', "\\*")
        .replace('#', "\\#")
        .replace('_', "\\_")
        .replace('^', "\\^")
        .replace('~', "\\~")
        .replace('`', "\\`")
        .replace('+', "\\+")
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use super::*;

    #[test]
    fn test_format_bicep_type() {
        assert_eq!(format_bicep_type(&BicepType::String), "string");
        assert_eq!(format_bicep_type(&BicepType::Int), "int");
        assert_eq!(format_bicep_type(&BicepType::Bool), "bool");
        assert_eq!(
            format_bicep_type(&BicepType::Array(Box::new(BicepType::String))),
            "string[]"
        );
        assert_eq!(
            format_bicep_type(&BicepType::CustomType("MyType".to_string())),
            "MyType"
        );
        assert_eq!(
            format_bicep_type(&BicepType::Union(vec!["A".to_string(), "B".to_string()])),
            "A | B"
        );

        assert_eq!(format_bicep_type(&BicepType::Object(None)), "object");

        let empty_props = IndexMap::new();
        assert_eq!(
            format_bicep_type(&BicepType::Object(Some(empty_props))),
            "object"
        );
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
        assert_eq!(
            format_bicep_value(&BicepValue::String("test".to_string())),
            "test"
        );
        assert_eq!(format_bicep_value(&BicepValue::Int(42)), "42");
        assert_eq!(format_bicep_value(&BicepValue::Bool(true)), "true");
        assert_eq!(
            format_bicep_value(&BicepValue::Identifier("myVar".to_string())),
            "${myVar}"
        );

        let arr = vec![BicepValue::Int(1), BicepValue::Int(2)];
        assert_eq!(format_bicep_value(&BicepValue::Array(arr)), "[1, 2]");

        let mut obj = IndexMap::new();
        obj.insert("key1".to_string(), BicepValue::String("value1".to_string()));
        obj.insert("key2".to_string(), BicepValue::Int(42));
        let result = format_bicep_value(&BicepValue::Object(obj));
        // Note: IndexMap preserves insertion order
        assert!(result.contains("key1: value1"));
        assert!(result.contains("key2: 42"));
    }

    #[test]
    fn test_format_bicep_value_with_multiline_string() {
        let multiline = "Line 1\nLine 2\nLine 3".to_string();
        let result = format_bicep_value(&BicepValue::String(multiline));
        assert_eq!(result, "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_escape_markdown() {
        assert_eq!(escape_markdown("simple text"), "simple text");
        assert_eq!(escape_markdown("text with *bold*"), "text with \\*bold\\*");
        assert_eq!(escape_markdown("text with [link]"), "text with \\[link\\]");
        assert_eq!(
            escape_markdown("text | with | pipes"),
            "text \\| with \\| pipes"
        );
    }

    #[test]
    fn test_escape_asciidoc() {
        assert_eq!(escape_asciidoc("simple text"), "simple text");
        assert_eq!(
            escape_asciidoc("text with |pipes|"),
            "text with \\|pipes\\|"
        );
        assert_eq!(
            escape_asciidoc("text with [brackets]"),
            "text with \\[brackets\\]"
        );
        assert_eq!(escape_asciidoc("text with *bold*"), "text with \\*bold\\*");
    }
}
