use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tree_sitter::{Parser, Tree};

pub mod parsing;
pub use parsing::{BicepDocument, BicepParserError, BicepType, BicepValue};

/// Parse a bicep file content and return the tree-sitter Tree
///
/// # Arguments
///
/// * `content` - The content of the Bicep file to parse
///
/// # Returns
///
/// An Option containing the parsed Tree if successful, None otherwise
pub fn parse_bicep_file(content: &str) -> Option<Tree> {
    let mut parser = Parser::new();

    if parser
        .set_language(&tree_sitter_bicep::LANGUAGE.into())
        .is_err()
    {
        return None;
    }

    parser.parse(content, None)
}

/// Wrapper function to parse a Bicep document from source code
///
/// # Arguments
///
/// * `source_code` - The source code of the Bicep file to parse
///
/// # Returns
///
/// A Result containing the parsed BicepDocument if successful, or an error
pub fn parse_bicep_document(source_code: &str) -> Result<parsing::BicepDocument, Box<dyn Error>> {
    let tree = parse_bicep_file(source_code)
        .ok_or_else(|| Box::<dyn Error>::from("Failed to parse Bicep file"))?;
    parsing::parse_bicep_document(&tree, source_code)
}

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
pub fn export_bicep_document_to_yaml<P: AsRef<Path>>(
    document: &BicepDocument,
    output_path: P,
) -> Result<(), Box<dyn Error>> {
    let yaml = export_bicep_document_to_yaml_string(document)?;
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
pub fn export_bicep_document_to_yaml_string(
    document: &BicepDocument,
) -> Result<String, Box<dyn Error>> {
    // First serialize using standard serde_yaml
    let yaml = serde_yaml::to_string(document)?;

    // Post-process to improve multiline string representation
    let improved_yaml = improve_multiline_string_representation(&yaml);
    Ok(improved_yaml)
}

/// Improve the YAML representation of multiline strings by ensuring consistency
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

/// Export a parsed Bicep document as JSON to a file
///
/// # Arguments
///
/// * `document` - The BicepDocument to export
/// * `output_path` - The path where the JSON file should be written
/// * `pretty` - Whether to format the JSON with indentation for readability
///
/// # Returns
///
/// A Result indicating success or an error
pub fn export_bicep_document_to_json<P: AsRef<Path>>(
    document: &BicepDocument,
    output_path: P,
    pretty: bool,
) -> Result<(), Box<dyn Error>> {
    let json = if pretty {
        serde_json::to_string_pretty(document)?
    } else {
        serde_json::to_string(document)?
    };

    let mut file = File::create(output_path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

/// Export a parsed Bicep document as JSON string
///
/// # Arguments
///
/// * `document` - The BicepDocument to export
/// * `pretty` - Whether to format the JSON with indentation for readability
///
/// # Returns
///
/// A Result containing the JSON string or an error
pub fn export_bicep_document_to_json_string(
    document: &BicepDocument,
    pretty: bool,
) -> Result<String, Box<dyn Error>> {
    if pretty {
        Ok(serde_json::to_string_pretty(document)?)
    } else {
        Ok(serde_json::to_string(document)?)
    }
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
pub fn parse_and_export_to_yaml<P: AsRef<Path>>(
    source_code: &str,
    output_path: P,
) -> Result<(), Box<dyn Error>> {
    let document = parse_bicep_document(source_code)?;
    export_bicep_document_to_yaml(&document, output_path)?;
    Ok(())
}

/// Parse a Bicep file and export it as JSON in one step
///
/// # Arguments
///
/// * `source_code` - The source code of the Bicep file
/// * `output_path` - The path where the JSON file should be written
/// * `pretty` - Whether to format the JSON with indentation for readability
///
/// # Returns
///
/// A Result indicating success or an error
pub fn parse_and_export_to_json<P: AsRef<Path>>(
    source_code: &str,
    output_path: P,
    pretty: bool,
) -> Result<(), Box<dyn Error>> {
    let document = parse_bicep_document(source_code)?;
    export_bicep_document_to_json(&document, output_path, pretty)?;
    Ok(())
}
