use bicep_docs::{
    export_bicep_document_to_json_string, export_bicep_document_to_yaml_string,
    parse_bicep_document,
};
use std::fs;

#[cfg(test)]
mod exports {
    use super::*;

    #[test]
    fn yaml() {
        // Read example Bicep file
        let test_file = "./examples/single-file/example.bicep";
        let source_code = fs::read_to_string(test_file).unwrap();

        // Parse the Bicep document
        let document = parse_bicep_document(&source_code).unwrap();

        // Export to YAML
        let yaml = export_bicep_document_to_yaml_string(&document).unwrap();

        // Basic validation - check that the YAML contains key elements
        assert!(yaml.contains("name:"));
        assert!(yaml.contains("description:"));
        assert!(yaml.contains("targetScope:"));
        assert!(yaml.contains("parameters:"));
        assert!(yaml.contains("resources:"));
        assert!(yaml.contains("outputs:"));

        // Check for specific content
        assert!(yaml.contains("requiredParam"));
        assert!(yaml.contains("nameVar"));
    }

    #[test]
    fn json() {
        // Read example Bicep file
        let test_file = "./examples/single-file/example.bicep";
        let source_code = fs::read_to_string(test_file).unwrap();

        // Parse the Bicep document
        let document = parse_bicep_document(&source_code).unwrap();

        // Export to JSON (both pretty and compact)
        let json_pretty = export_bicep_document_to_json_string(&document, true).unwrap();
        let json_compact = export_bicep_document_to_json_string(&document, false).unwrap();

        // Basic validation - check that the JSON contains key elements
        assert!(json_pretty.contains("\"name\":"));
        assert!(json_pretty.contains("\"description\":"));
        assert!(json_pretty.contains("\"targetScope\":"));
        assert!(json_pretty.contains("\"parameters\":"));
        assert!(json_pretty.contains("\"resources\":"));
        assert!(json_pretty.contains("\"outputs\":"));

        // Check for specific content
        assert!(json_pretty.contains("\"requiredParam\""));
        assert!(json_pretty.contains("\"nameVar\""));

        // Verify that compact JSON is smaller than pretty JSON
        assert!(json_compact.len() < json_pretty.len());
    }
}
