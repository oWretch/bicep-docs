/// Utilities for export functionality
///
/// This module provides common utility functions used across
/// different export formats to eliminate code duplication.
pub mod common;
pub mod formatting;

// Re-export commonly used functions for easy access
pub use common::{
    format_yes_no, generate_metadata_display_asciidoc, generate_metadata_display_markdown,
};
pub use formatting::{
    escape_asciidoc, escape_markdown, format_bicep_type_with_backticks,
    format_bicep_type_with_monospace, format_bicep_value_as_code,
};
