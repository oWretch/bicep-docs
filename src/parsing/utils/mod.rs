//! Utility modules for Bicep parsing
//!
//! This module contains specialized utility functions organized by domain:
//! - decorators: Decorator parsing and extraction utilities
//! - types: Type parsing utilities for various Bicep type expressions
//! - values: Value parsing utilities for literals and expressions

use std::error::Error;
use std::str;
use tree_sitter::Node;

pub mod decorators;
pub mod types;
pub mod values;

// Re-export commonly used utilities
pub use decorators::{extract_description_from_decorators, parse_decorator, parse_decorators};

pub use types::{parse_array_type, parse_property_type, parse_type_node, parse_union_type};
pub use values::{parse_array_items, parse_value_node};

/// Extracts and trims the UTF-8 text for a given tree-sitter node from the source code.
///
/// This utility function safely retrieves the text represented by a `tree_sitter::Node`
/// from the provided source code, trims leading newline, trailing whitespace, and any
/// common indentation whitespace, then returns it as a `String`.
///
/// # Arguments
///
/// * `node` - The tree-sitter node whose text should be extracted.
/// * `source_code` - The full source code as a string slice.
///
/// # Returns
///
/// Returns a `Result<String, Box<dyn Error>>` containing the trimmed node text on success,
/// or an error if the node's text cannot be extracted (e.g., invalid UTF-8).
///
/// # Errors
///
/// Returns an error if the node's byte range is invalid for the given source code,
/// or if the text is not valid UTF-8.
///
/// # Examples
///
/// ```ignore
/// let text = get_node_text(&node, source_code)?;
/// ```
///
pub fn get_node_text(node: &Node, source_code: &str) -> Result<String, Box<dyn Error>> {
    let mut text = node.utf8_text(source_code.as_bytes())?;
    text = text.strip_prefix('\n').unwrap_or(text);

    // Identify indentation
    let mut leading_whitespace = String::new();
    for c in text.chars() {
        if c.is_whitespace() {
            leading_whitespace.push(c);
        } else {
            break;
        }
    }
    Ok(text
        .trim_end()
        .split('\n')
        .map(|line| {
            line.strip_prefix(&leading_whitespace)
                .unwrap_or(line)
                .trim_end()
        })
        .collect::<Vec<&str>>()
        .join("\n")
        .to_string())
}
