use bicep_docs::{
    export_bicep_document_to_markdown_string, parse_and_export_to_markdown, parse_bicep_document,
};
use std::fs;

#[cfg(test)]
mod markdown {
    use super::*;

    #[test]
    fn test_markdown_export() {
        // Read the parameters test file
        let content =
            fs::read_to_string("tests/parsing/parameters.bicep").expect("Failed to read test file");

        // Parse the document
        let document = parse_bicep_document(&content).expect("Failed to parse document");

        // Export to markdown string
        let markdown = export_bicep_document_to_markdown_string(&document)
            .expect("Failed to export to markdown");

        // Basic checks
        assert!(markdown.contains("# Bicep Template"));
        assert!(markdown.contains("## Parameters"));
        assert!(markdown.len() > 100); // Should have substantial content

        println!("Generated markdown preview:");
        let lines: Vec<&str> = markdown.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if i < 30 {
                // Print first 30 lines
                println!("{}", line);
            }
        }
        if lines.len() > 30 {
            println!("... ({} more lines)", lines.len() - 30);
        }
    }

    #[test]
    fn test_parse_and_export_to_markdown() {
        let output_path = "sample_parameters.md";

        // Test the convenience function
        parse_and_export_to_markdown("tests/parsing/parameters.bicep", output_path)
            .expect("Failed to parse and export");

        // Verify the file was created
        assert!(std::path::Path::new(output_path).exists());

        // Read and verify content
        let content = fs::read_to_string(output_path).expect("Failed to read output file");

        assert!(content.contains("# Bicep Template"));
        assert!(content.contains("## Parameters"));

        println!("Markdown successfully exported to {}", output_path);
        // Note: Not cleaning up the file so we can inspect it
    }
}
