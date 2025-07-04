use std::{error::Error, path::Path};

use tree_sitter::{Parser, Tree};

// Initialize rust-i18n
rust_i18n::i18n!("locales");

pub mod exports;
pub mod localization;
pub mod parsing;

pub use parsing::{BicepDocument, BicepParserError, BicepType, BicepValue};

// Re-export the t! macro to make it available throughout the crate
pub use rust_i18n::t;

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
/// * `exclude_empty` - Whether to exclude empty sections from the output
///
/// # Returns
///
/// A Result indicating success or an error
pub fn export_bicep_document_to_yaml<P: AsRef<Path>>(
    document: &BicepDocument,
    output_path: P,
    exclude_empty: bool,
) -> Result<(), Box<dyn Error>> {
    exports::yaml::export_to_file(document, output_path, exclude_empty)
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
pub fn export_bicep_document_to_yaml_string(
    document: &BicepDocument,
    exclude_empty: bool,
) -> Result<String, Box<dyn Error>> {
    exports::yaml::export_to_string(document, exclude_empty)
}

/// Export a parsed Bicep document as JSON to a file
///
/// # Arguments
///
/// * `document` - The BicepDocument to export
/// * `output_path` - The path where the JSON file should be written
/// * `pretty` - Whether to format the JSON with indentation for readability
/// * `exclude_empty` - Whether to exclude empty sections from the output
///
/// # Returns
///
/// A Result indicating success or an error
pub fn export_bicep_document_to_json<P: AsRef<Path>>(
    document: &BicepDocument,
    output_path: P,
    pretty: bool,
    exclude_empty: bool,
) -> Result<(), Box<dyn Error>> {
    exports::json::export_to_file(document, output_path, pretty, exclude_empty)
}

/// Export a parsed Bicep document as JSON string
///
/// # Arguments
///
/// * `document` - The BicepDocument to export
/// * `pretty` - Whether to format the JSON with indentation for readability
/// * `exclude_empty` - Whether to exclude empty sections from the output
///
/// # Returns
///
/// A Result containing the JSON string or an error
pub fn export_bicep_document_to_json_string(
    document: &BicepDocument,
    pretty: bool,
    exclude_empty: bool,
) -> Result<String, Box<dyn Error>> {
    exports::json::export_to_string(document, pretty, exclude_empty)
}

/// Export a parsed Bicep document as Markdown to a file
///
/// # Arguments
///
/// * `document` - The BicepDocument to export
/// * `output_path` - The path where the Markdown file should be written
/// * `use_emoji` - Whether to use emoji symbols (✅/❌) for Yes/No values
/// * `exclude_empty` - Whether to exclude empty sections from the output
///
/// # Returns
///
/// A Result indicating success or an error
pub fn export_bicep_document_to_markdown<P: AsRef<Path>>(
    document: &BicepDocument,
    output_path: P,
    use_emoji: bool,
    exclude_empty: bool,
) -> Result<(), Box<dyn Error>> {
    exports::markdown::export_to_file(document, output_path, use_emoji, exclude_empty)
}

/// Export a parsed Bicep document as Markdown string
///
/// # Arguments
///
/// * `document` - The BicepDocument to export
/// * `use_emoji` - Whether to use emoji symbols (✅/❌) for Yes/No values
/// * `exclude_empty` - Whether to exclude empty sections from the output
///
/// # Returns
///
/// A Result containing the Markdown string or an error
pub fn export_bicep_document_to_markdown_string(
    document: &BicepDocument,
    use_emoji: bool,
    exclude_empty: bool,
) -> Result<String, Box<dyn Error>> {
    exports::markdown::export_to_string(document, use_emoji, exclude_empty)
}

/// Export a parsed Bicep document as Markdown string with localization support
///
/// # Arguments
///
/// * `document` - The BicepDocument to export
/// * `use_emoji` - Whether to use emoji symbols (✅/❌) for Yes/No values
/// * `exclude_empty` - Whether to exclude empty sections from the output
///
/// # Returns
///
/// A Result containing the Markdown string or an error
pub fn export_bicep_document_to_markdown_string_localized(
    document: &BicepDocument,
    use_emoji: bool,
    exclude_empty: bool,
) -> Result<String, Box<dyn Error>> {
    exports::markdown::export_to_string_localized(document, use_emoji, exclude_empty)
}

/// Export a parsed Bicep document as AsciiDoc to a file
///
/// # Arguments
///
/// * `document` - The BicepDocument to export
/// * `output_path` - The path where the AsciiDoc file should be written
/// * `use_emoji` - Whether to use emoji symbols (✅/❌) for Yes/No values
/// * `exclude_empty` - Whether to exclude empty sections from the output
///
/// # Returns
///
/// A Result indicating success or an error
pub fn export_bicep_document_to_asciidoc<P: AsRef<Path>>(
    document: &BicepDocument,
    output_path: P,
    use_emoji: bool,
    exclude_empty: bool,
) -> Result<(), Box<dyn Error>> {
    exports::asciidoc::export_to_file(document, output_path, use_emoji, exclude_empty)
}

/// Export a parsed Bicep document as AsciiDoc string
///
/// # Arguments
///
/// * `document` - The BicepDocument to export
/// * `use_emoji` - Whether to use emoji symbols (✅/❌) for Yes/No values
/// * `exclude_empty` - Whether to exclude empty sections from the output
///
/// # Returns
///
/// A Result containing the AsciiDoc string or an error
pub fn export_bicep_document_to_asciidoc_string(
    document: &BicepDocument,
    use_emoji: bool,
    exclude_empty: bool,
) -> Result<String, Box<dyn Error>> {
    exports::asciidoc::export_to_string(document, use_emoji, exclude_empty)
}

/// Parse a Bicep file and export it as AsciiDoc in one step
///
/// # Arguments
///
/// * `file_path` - The path to the Bicep file to parse
/// * `output_path` - The path where the AsciiDoc file should be written
/// * `exclude_empty` - Whether to exclude empty sections from the output
///
/// # Returns
///
/// A Result indicating success or an error
pub fn parse_and_export_to_asciidoc<P: AsRef<Path>, Q: AsRef<Path>>(
    file_path: P,
    output_path: Q,
    exclude_empty: bool,
) -> Result<(), Box<dyn Error>> {
    exports::asciidoc::parse_and_export(file_path, output_path, exclude_empty)
}

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
pub fn parse_and_export_to_yaml<P: AsRef<Path>>(
    source_code: &str,
    output_path: P,
    exclude_empty: bool,
) -> Result<(), Box<dyn Error>> {
    exports::yaml::parse_and_export(source_code, output_path, exclude_empty)
}

/// Parse a Bicep file and export it as JSON in one step
///
/// # Arguments
///
/// * `source_code` - The source code of the Bicep file
/// * `output_path` - The path where the JSON file should be written
/// * `pretty` - Whether to format the JSON with indentation for readability
/// * `exclude_empty` - Whether to exclude empty sections from the output
///
/// # Returns
///
/// A Result indicating success or an error
pub fn parse_and_export_to_json<P: AsRef<Path>>(
    source_code: &str,
    output_path: P,
    pretty: bool,
    exclude_empty: bool,
) -> Result<(), Box<dyn Error>> {
    exports::json::parse_and_export(source_code, output_path, pretty, exclude_empty)
}

/// Parse a Bicep file and export it as Markdown in one step
///
/// # Arguments
///
/// * `file_path` - The path to the Bicep file to parse
/// * `output_path` - The path where the Markdown file should be written
/// * `exclude_empty` - Whether to exclude empty sections from the output
///
/// # Returns
///
/// A Result indicating success or an error
pub fn parse_and_export_to_markdown<P: AsRef<Path>, Q: AsRef<Path>>(
    file_path: P,
    output_path: Q,
    exclude_empty: bool,
) -> Result<(), Box<dyn Error>> {
    exports::markdown::parse_and_export(file_path, output_path, exclude_empty)
}

/// Test example to demonstrate the localization system
#[cfg(test)]
mod localization_demo {
    use crate::{
        localization::{init_localization, Language},
        t,
    };

    #[test]
    fn demonstrate_translations() {
        // Test key translations across languages
        init_localization(Language::English);
        assert_eq!(t!("common.yes"), "Yes");
        assert_eq!(t!("export.types"), "Types");
        assert_eq!(t!("export.target_scope"), "Target Scope");

        init_localization(Language::Spanish);
        assert_eq!(t!("common.yes"), "Sí");
        assert_eq!(t!("export.types"), "Tipos");
        assert_eq!(t!("export.target_scope"), "Ámbito de Destino");

        init_localization(Language::French);
        assert_eq!(t!("common.yes"), "Oui");
        assert_eq!(t!("export.types"), "Types");
        assert_eq!(t!("export.target_scope"), "Portée Cible");

        println!("✅ All translations working correctly!");
    }
}
