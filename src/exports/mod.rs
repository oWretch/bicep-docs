/// Export functionality for Bicep documents
///
/// This module provides various export formats for parsed Bicep documents.
/// Each export format is implemented in its own submodule to maintain
/// separation of concerns and make it easy to add new formats.
pub mod asciidoc;
pub mod json;
pub mod markdown;
pub mod utils;
pub mod yaml;

// Re-export the main export functions for convenience
pub use asciidoc::{
    export_to_file as export_asciidoc_to_file, export_to_string as export_asciidoc_to_string,
    parse_and_export as parse_and_export_asciidoc,
};
pub use json::{
    export_to_file as export_json_to_file, export_to_string as export_json_to_string,
    parse_and_export as parse_and_export_json,
};
pub use markdown::{
    export_to_file as export_markdown_to_file, export_to_string as export_markdown_to_string,
    parse_and_export as parse_and_export_markdown,
};
pub use yaml::{
    export_to_file as export_yaml_to_file, export_to_string as export_yaml_to_string,
    parse_and_export as parse_and_export_yaml,
};
