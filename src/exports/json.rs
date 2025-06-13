/// JSON export functionality for Bicep documents
///
/// This module provides functions to export parsed Bicep documents
/// to JSON format with support for both compact and pretty-printed output.
use std::error::Error;
use std::{fs::File, io::Write, path::Path};

use crate::parsing::BicepDocument;

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
pub fn export_to_file<P: AsRef<Path>>(
    document: &BicepDocument,
    output_path: P,
    pretty: bool,
    exclude_empty: bool,
) -> Result<(), Box<dyn Error>> {
    let json = export_to_string(document, pretty, exclude_empty)?;
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
/// * `exclude_empty` - Whether to exclude empty sections from the output
///
/// # Returns
///
/// A Result containing the JSON string or an error
pub fn export_to_string(
    document: &BicepDocument,
    pretty: bool,
    _exclude_empty: bool,
) -> Result<String, Box<dyn Error>> {
    // Note: exclude_empty parameter is kept for API consistency with other exporters
    // The BicepDocument already has serde attributes that handle skipping empty collections
    let json = if pretty {
        serde_json::to_string_pretty(document)?
    } else {
        serde_json::to_string(document)?
    };

    // The #[serde(skip_serializing_if = "...")] attributes on the BicepDocument struct
    // handle skipping empty collections during serialization, so we don't need
    // to do any additional filtering

    Ok(json)
}

// We use the #[serde(skip_serializing_if = "...")] attributes on the BicepDocument struct
// to handle skipping empty collections during serialization, so no explicit
// filter_empty_sections function is needed.

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
pub fn parse_and_export<P: AsRef<Path>>(
    source_code: &str,
    output_path: P,
    pretty: bool,
    exclude_empty: bool,
) -> Result<(), Box<dyn Error>> {
    let document = crate::parse_bicep_document(source_code)?;
    export_to_file(&document, output_path, pretty, exclude_empty)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use super::*;
    use crate::parsing::{BicepDocument, BicepParameter, BicepType, BicepValue};

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

        let result = export_to_string(&document, true, false);
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

        let result = export_to_string(&document, false, false);
        assert!(result.is_ok());

        let json = result.unwrap();
        assert!(!json.contains("\n")); // Compact should not have newlines
    }

    #[test]
    fn test_export_to_string_with_exclude_empty() {
        // Create a document with some empty collections and one non-empty collection
        let mut document = BicepDocument {
            name: Some("Test Template".to_string()),
            description: Some("A test template".to_string()),
            ..Default::default()
        };

        // Add one parameter to make that collection non-empty
        let parameter = BicepParameter {
            parameter_type: BicepType::String,
            description: Some("Test parameter".to_string()),
            ..Default::default()
        };
        document
            .parameters
            .insert("testParam".to_string(), parameter);

        // Test with exclude_empty = false (default behavior)
        let result_with_all = export_to_string(&document, true, false).unwrap();

        // Test with exclude_empty = true
        let result_without_empty = export_to_string(&document, true, true).unwrap();

        // Both should contain the document name and the parameter
        assert!(result_with_all.contains("\"name\": \"Test Template\""));
        assert!(result_without_empty.contains("\"name\": \"Test Template\""));
        assert!(result_with_all.contains("\"testParam\""));
        assert!(result_without_empty.contains("\"testParam\""));

        // The JSON export relies on the serde attributes to skip empty collections,
        // so both outputs should be identical in this case
        assert_eq!(result_with_all, result_without_empty);
    }
}
