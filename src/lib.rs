use std::error::Error;
use std::path::Path;
use tree_sitter::{Parser, Tree};

pub mod exports;
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

// Backward compatibility functions that delegate to the new export modules

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
    exports::yaml::export_to_file(document, output_path)
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
    exports::yaml::export_to_string(document)
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
    exports::json::export_to_file(document, output_path, pretty)
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
    exports::json::export_to_string(document, pretty)
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
    exports::yaml::parse_and_export(source_code, output_path)
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
    exports::json::parse_and_export(source_code, output_path, pretty)
}
