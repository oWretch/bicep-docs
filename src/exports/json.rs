/// JSON export functionality for Bicep documents
///
/// This module provides functions to export parsed Bicep documents
/// to JSON format with support for both compact and pretty-printed output.
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::parsing::BicepDocument;

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
pub fn export_to_file<P: AsRef<Path>>(
    document: &BicepDocument,
    output_path: P,
    pretty: bool,
) -> Result<(), Box<dyn Error>> {
    let json = export_to_string(document, pretty)?;
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
pub fn export_to_string(document: &BicepDocument, pretty: bool) -> Result<String, Box<dyn Error>> {
    if pretty {
        Ok(serde_json::to_string_pretty(document)?)
    } else {
        Ok(serde_json::to_string(document)?)
    }
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
pub fn parse_and_export<P: AsRef<Path>>(
    source_code: &str,
    output_path: P,
    pretty: bool,
) -> Result<(), Box<dyn Error>> {
    let document = crate::parse_bicep_document(source_code)?;
    export_to_file(&document, output_path, pretty)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::{BicepDocument, BicepValue};
    use indexmap::IndexMap;

    #[test]
    fn test_export_to_string_pretty() {
        let mut document = BicepDocument {
            name: None,
            description: None,
            metadata: IndexMap::new(),
            target_scope: None,
            imports: Vec::new(),
            types: IndexMap::new(),
            functions: IndexMap::new(),
            parameters: IndexMap::new(),
            variables: IndexMap::new(),
            resources: IndexMap::new(),
            modules: IndexMap::new(),
            outputs: IndexMap::new(),
        };

        document
            .metadata
            .insert("test".to_string(), BicepValue::String("value".to_string()));

        let result = export_to_string(&document, true);
        assert!(result.is_ok());

        let json = result.unwrap();
        assert!(json.contains("{\n")); // Pretty-printed should have newlines
        assert!(json.contains("  ")); // Pretty-printed should have indentation
    }

    #[test]
    fn test_export_to_string_compact() {
        let document = BicepDocument {
            name: None,
            description: None,
            metadata: IndexMap::new(),
            target_scope: None,
            imports: Vec::new(),
            types: IndexMap::new(),
            functions: IndexMap::new(),
            parameters: IndexMap::new(),
            variables: IndexMap::new(),
            resources: IndexMap::new(),
            modules: IndexMap::new(),
            outputs: IndexMap::new(),
        };

        let result = export_to_string(&document, false);
        assert!(result.is_ok());

        let json = result.unwrap();
        assert!(!json.contains("\n")); // Compact should not have newlines
    }
}
