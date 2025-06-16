use std::fs;

use bicep_docs::{
    export_bicep_document_to_markdown_string, parse_and_export_to_markdown, parse_bicep_document,
};

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

        // Export to markdown string (with exclude_empty = false)
        let markdown = export_bicep_document_to_markdown_string(&document, true, false)
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
        let output_path = std::env::temp_dir().join("sample_parameters.md");

        // Use a closure to ensure cleanup happens even if the test fails
        let result = std::panic::catch_unwind(|| {
            // Test the convenience function (with exclude_empty = false)
            parse_and_export_to_markdown(
                "tests/parsing/parameters.bicep",
                output_path.clone(),
                false,
            )
            .expect("Failed to parse and export");

            // Verify the file was created
            assert!(output_path.exists());

            // Read and verify content
            let content = fs::read_to_string(&output_path).expect("Failed to read output file");

            assert!(content.contains("# Bicep Template"));
            assert!(content.contains("## Parameters"));

            println!(
                "Markdown successfully exported to {}",
                output_path.display()
            );
        });

        // Always attempt to remove the file, regardless of test outcome
        let _ = fs::remove_file(&output_path);

        // Resume panic if the test closure failed
        if let Err(err) = result {
            std::panic::resume_unwind(err);
        }
    }

    #[test]
    fn test_markdown_with_exclude_empty() {
        // Read the parameters test file
        let content =
            fs::read_to_string("tests/parsing/parameters.bicep").expect("Failed to read test file");

        // Parse the document
        let document = parse_bicep_document(&content).expect("Failed to parse document");

        // Test with exclude_empty = false (default behavior)
        let markdown_with_empty = export_bicep_document_to_markdown_string(&document, true, false)
            .expect("Failed to export to markdown");

        // Test with exclude_empty = true
        let markdown_without_empty =
            export_bicep_document_to_markdown_string(&document, true, true)
                .expect("Failed to export to markdown");

        // Both should contain essential sections
        assert!(markdown_with_empty.contains("# Bicep Template"));
        assert!(markdown_without_empty.contains("# Bicep Template"));
        assert!(markdown_with_empty.contains("## Parameters"));
        assert!(markdown_without_empty.contains("## Parameters"));

        // The version with empty sections should be longer
        assert!(markdown_with_empty.len() > markdown_without_empty.len());

        // Check if empty sections are present/missing as expected
        if !document.resources.is_empty() {
            assert!(markdown_with_empty.contains("## Resources"));
            assert!(markdown_without_empty.contains("## Resources"));
        } else {
            assert!(markdown_with_empty.contains("## Resources"));
            assert!(markdown_with_empty.contains("*No resources defined*"));
            assert!(!markdown_without_empty.contains("## Resources"));
        }
    }
}
