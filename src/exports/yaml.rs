/// YAML export functionality for Bicep documents
///
/// This module provides functions to export parsed Bicep documents
/// to YAML format with improved multiline string representation.
use std::error::Error;
use std::{fs::File, io::Write, path::Path};

use crate::parsing::BicepDocument;

/// Export a parsed Bicep document as YAML to a file
///
/// # Arguments
///
/// * `document` - The BicepDocument to export
/// * `output_path` - The path where the YAML file should be written
/// * `exclude_empty` - Whether to exclude empty sections from the output
///
/// # Returns
///
/// A Result indicating success or an error
pub fn export_to_file<P: AsRef<Path>>(
    document: &BicepDocument,
    output_path: P,
    exclude_empty: bool,
) -> Result<(), Box<dyn Error>> {
    let yaml = export_to_string(document, exclude_empty)?;
    let mut file = File::create(output_path)?;
    file.write_all(yaml.as_bytes())?;
    Ok(())
}

/// Export a parsed Bicep document as YAML string
///
/// # Arguments
///
/// * `document` - The BicepDocument to export
/// * `exclude_empty` - Whether to exclude empty sections from the output
///
/// # Returns
///
/// A Result containing the YAML string or an error
pub fn export_to_string(
    document: &BicepDocument,
    _exclude_empty: bool,
) -> Result<String, Box<dyn Error>> {
    // Note: exclude_empty parameter is kept for API consistency with other exporters
    // The BicepDocument already has serde attributes that handle skipping empty collections
    let yaml = serde_yaml::to_string(document)?;

    // Post-process to improve multiline string representation
    let improved_yaml = improve_multiline_string_representation(&yaml);
    Ok(improved_yaml)
}

// We use the #[serde(skip_serializing_if = "...")] attributes on the BicepDocument struct
// to handle skipping empty collections during serialization, so no explicit
// filter_empty_sections function is needed.

/// Parse a Bicep file and export it as YAML in one step
///
/// # Arguments
///
/// * `source_code` - The source code of the Bicep file
/// * `output_path` - The path where the YAML file should be written
/// * `exclude_empty` - Whether to exclude empty sections from the output
///
/// # Returns
///
/// A Result indicating success or an error
pub fn parse_and_export<P: AsRef<Path>>(
    source_code: &str,
    output_path: P,
    exclude_empty: bool,
) -> Result<(), Box<dyn Error>> {
    let document = crate::parse_bicep_document(source_code)?;
    export_to_file(&document, output_path, exclude_empty)?;
    Ok(())
}

/// Improve the YAML representation of multiline strings by ensuring consistency
///
/// This function processes YAML output to convert escaped multiline strings
/// to block scalar format for better readability.
///
/// # Arguments
///
/// * `yaml` - The YAML string to process
///
/// # Returns
///
/// An improved YAML string with better multiline string representation
fn improve_multiline_string_representation(yaml: &str) -> String {
    let lines: Vec<&str> = yaml.lines().collect();
    let mut result = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        // Check if this line contains a quoted multiline string that should be block style
        if line.contains("\"") && (line.contains("\\n") || line.contains("\\t")) {
            // Extract the key and value
            if let Some(colon_pos) = line.find(':') {
                let key_part = &line[..colon_pos + 1];
                let value_part = line[colon_pos + 1..].trim();

                // Check if it's a quoted string with escape sequences
                if value_part.starts_with('"') && value_part.ends_with('"') && value_part.len() > 2
                {
                    let inner_content = &value_part[1..value_part.len() - 1];

                    // If it contains newlines, convert to block scalar
                    if inner_content.contains("\\n") {
                        // Convert to block scalar format
                        let block_content = convert_to_block_scalar(inner_content, key_part);
                        result.push(block_content);
                        i += 1;
                        continue;
                    }
                }
            }
        }

        result.push(line.to_string());
        i += 1;
    }

    result.join("\n")
}

/// Unescapes a string containing common YAML escape sequences.
///
/// Handles \\n, \\t, \\\\, \\", \\'.
fn unescape_yaml_string(s: &str) -> String {
    let mut unescaped = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => unescaped.push('\n'),
                Some('t') => unescaped.push('\t'),
                Some('\\') => unescaped.push('\\'),
                Some('"') => unescaped.push('"'),
                Some('\'') => unescaped.push('\''),
                Some(other) => {
                    // Pass through unrecognized escape sequences
                    unescaped.push('\\');
                    unescaped.push(other);
                },
                None => unescaped.push('\\'), // Trailing backslash
            }
        } else {
            unescaped.push(c);
        }
    }
    unescaped
}

/// Convert escaped string content to block scalar format
///
/// This function converts escaped string content to YAML block scalar format
/// for improved readability of multiline strings.
///
/// # Arguments
///
/// * `content` - The escaped string content to convert
/// * `key_part` - The key part of the YAML line (used for indentation)
///
/// # Returns
///
/// A formatted YAML block scalar string
fn convert_to_block_scalar(content: &str, key_part: &str) -> String {
    // Unescape the content
    let unescaped = unescape_yaml_string(content);

    // Determine indentation based on key part
    let base_indent = key_part.len() - key_part.trim_start().len();
    let content_indent = " ".repeat(base_indent + 2);

    // Split into lines and format as block scalar
    let lines: Vec<&str> = unescaped.lines().collect();
    if lines.len() > 1 {
        let mut result = format!("{key_part} |-");
        for line in lines {
            result.push_str(&format!("\n{content_indent}{line}"));
        }
        result
    } else {
        // Single line, keep as quoted string
        format!("{key_part} \"{content}\"")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::{BicepDocument, BicepType};

    #[test]
    fn test_convert_to_block_scalar_multiline() {
        let content = "Line 1\\nLine 2\\nLine 3";
        let key_part = "  description:";
        let result = convert_to_block_scalar(content, key_part);

        let expected = "  description: |-\n    Line 1\n    Line 2\n    Line 3";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_convert_to_block_scalar_single_line() {
        let content = "Single line description";
        let key_part = "  description:";
        let result = convert_to_block_scalar(content, key_part);

        let expected = "  description: \"Single line description\"";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_improve_multiline_string_representation() {
        let yaml = r#"field: "Line 1\nLine 2\nLine 3""#;
        let result = improve_multiline_string_representation(yaml);

        let expected = "field: |-\n  Line 1\n  Line 2\n  Line 3";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_export_to_string_with_exclude_empty() {
        // Create a document with some empty collections
        let mut document = BicepDocument {
            name: Some("Test Template".to_string()),
            description: Some("A test template".to_string()),
            ..Default::default()
        };

        // Add one parameter to make that collection non-empty
        document.parameters.insert(
            "testParam".to_string(),
            crate::parsing::BicepParameter {
                parameter_type: BicepType::String,
                description: Some("Test parameter".to_string()),
                ..Default::default()
            },
        );

        // Test with exclude_empty = false (default behavior)
        let result_with_all = export_to_string(&document, false).unwrap();

        // Test with exclude_empty = true
        let result_without_empty = export_to_string(&document, true).unwrap();

        // Both should contain the document name and the parameter
        assert!(result_with_all.contains("name: Test Template"));
        assert!(result_without_empty.contains("name: Test Template"));
        assert!(result_with_all.contains("testParam:"));
        assert!(result_without_empty.contains("testParam:"));

        // The YAML export relies on the serde attributes to skip empty collections,
        // so both outputs should be identical in this case
        assert_eq!(result_with_all, result_without_empty);
    }
}
