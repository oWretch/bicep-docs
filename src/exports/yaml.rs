/// YAML export functionality for Bicep documents
///
/// This module provides functions to export parsed Bicep documents
/// to YAML format with improved multiline string representation.
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::parsing::BicepDocument;

/// Export a parsed Bicep document as YAML to a file
///
/// # Arguments
///
/// * `document` - The BicepDocument to export
/// * `output_path` - The path where the YAML file should be written
///
/// # Returns
///
/// A Result indicating success or an error
pub fn export_to_file<P: AsRef<Path>>(
    document: &BicepDocument,
    output_path: P,
) -> Result<(), Box<dyn Error>> {
    let yaml = export_to_string(document)?;
    let mut file = File::create(output_path)?;
    file.write_all(yaml.as_bytes())?;
    Ok(())
}

/// Export a parsed Bicep document as YAML string
///
/// # Arguments
///
/// * `document` - The BicepDocument to export
///
/// # Returns
///
/// A Result containing the YAML string or an error
pub fn export_to_string(document: &BicepDocument) -> Result<String, Box<dyn Error>> {
    // First serialize using standard serde_yaml
    let yaml = serde_yaml::to_string(document)?;

    // Post-process to improve multiline string representation
    let improved_yaml = improve_multiline_string_representation(&yaml);
    Ok(improved_yaml)
}

/// Parse a Bicep file and export it as YAML in one step
///
/// # Arguments
///
/// * `source_code` - The source code of the Bicep file
/// * `output_path` - The path where the YAML file should be written
///
/// # Returns
///
/// A Result indicating success or an error
pub fn parse_and_export<P: AsRef<Path>>(
    source_code: &str,
    output_path: P,
) -> Result<(), Box<dyn Error>> {
    let document = crate::parse_bicep_document(source_code)?;
    export_to_file(&document, output_path)?;
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
    let unescaped = content
        .replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace("\\\\", "\\")
        .replace("\\\"", "\"")
        .replace("\\'", "'");

    // Determine indentation based on key part
    let base_indent = key_part.len() - key_part.trim_start().len();
    let content_indent = " ".repeat(base_indent + 2);

    // Split into lines and format as block scalar
    let lines: Vec<&str> = unescaped.lines().collect();
    if lines.len() > 1 {
        let mut result = format!("{} |-", key_part);
        for line in lines {
            result.push_str(&format!("\n{}{}", content_indent, line));
        }
        result
    } else {
        // Single line, keep as quoted string
        format!("{} \"{}\"", key_part, content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
