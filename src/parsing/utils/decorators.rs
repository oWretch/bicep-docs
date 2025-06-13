//! Decorator parsing and extraction utilities for Bicep files.
//!
//! This module provides centralized utilities for parsing and processing decorators
//! across all Bicep declaration types. It handles common decorator patterns like
//! description extraction, metadata processing, and constraint validation.

use std::error::Error;

use indexmap::IndexMap;
use tracing::{debug, warn};
use tree_sitter::Node;

use super::{
    super::{BicepDecorator, BicepValue},
    get_node_text,
    values::parse_value_node,
};

/// Type alias for the return type of `process_common_decorators` function.
///
/// Contains: (description, metadata, min_length, max_length, min_value, max_value, is_secure, is_sealed)
type CommonDecoratorsResult = (
    Option<String>,
    Option<IndexMap<String, BicepValue>>,
    Option<i64>,
    Option<i64>,
    Option<i64>,
    Option<i64>,
    bool,
    bool,
);

// ---------------------------------------------------------------
// Description Extraction
// ---------------------------------------------------------------

/// Extract description from decorators using the standard priority order.
///
/// Searches decorators for description values in the following priority:
/// 1. Explicit @description or @sys.description decorators
/// 2. Description field within @metadata or @sys.metadata decorators
///
/// # Arguments
///
/// * `decorators` - Slice of decorators to search
///
/// # Returns
///
/// Optional description string if found
pub fn extract_description_from_decorators(decorators: &[BicepDecorator]) -> Option<String> {
    // First, prioritize explicit description decorators
    for decorator in decorators {
        match decorator.name.as_str() {
            "description" | "sys.description" => {
                if let BicepValue::String(desc_text) = &decorator.argument {
                    if !desc_text.is_empty() {
                        return Some(desc_text.clone());
                    }
                }
            },
            _ => {},
        }
    }

    // If no explicit description decorator, check metadata
    for decorator in decorators {
        match decorator.name.as_str() {
            "metadata" | "sys.metadata" => {
                if let BicepValue::Object(map) = &decorator.argument {
                    if let Some(BicepValue::String(desc_text)) = map.get("description") {
                        if !desc_text.is_empty() {
                            return Some(desc_text.clone());
                        }
                    }
                }
            },
            _ => {},
        }
    }

    None
}

// ---------------------------------------------------------------
// Decorator Parsing
// ---------------------------------------------------------------

/// Parse decorators from a decorators node.
///
/// Extracts all decorator declarations from a tree-sitter node and converts
/// them into BicepDecorator structures.
///
/// # Arguments
///
/// * `node` - The tree-sitter Node containing decorator declarations
/// * `source_code` - The source code text for text extraction
///
/// # Returns
///
/// Result containing vector of parsed decorators or an error
pub fn parse_decorators(
    node: Node,
    source_code: &str,
) -> Result<Vec<BicepDecorator>, Box<dyn Error>> {
    let mut decorators = Vec::new();
    let mut cursor = node.walk();

    let children = node.children(&mut cursor).collect::<Vec<_>>();
    for child in children {
        if child.kind() == "decorator" {
            match parse_decorator(child, source_code) {
                Ok(decorator) => {
                    // Only add decorators with non-empty names
                    if !decorator.name.is_empty() {
                        decorators.push(decorator);
                    }
                },
                Err(e) => warn!("Failed to parse decorator: {}", e),
            }
        }
    }

    Ok(decorators)
}

/// Parse a single decorator from a tree-sitter node.
///
/// Handles various decorator formats including simple decorators (@export),
/// decorators with string arguments (@description('text')), and decorators
/// with object arguments (@metadata({key: 'value'})).
///
/// # Arguments
///
/// * `node` - The tree-sitter Node representing the decorator
/// * `source_code` - The source code text for text extraction
///
/// # Returns
///
/// Result containing the parsed BicepDecorator or an error
pub fn parse_decorator(node: Node, source_code: &str) -> Result<BicepDecorator, Box<dyn Error>> {
    let mut name = String::new();
    let mut argument = BicepValue::String(String::new());

    //  Get the call_expression
    let mut cursor = node.walk();
    let call_expression = node.children(&mut cursor).collect::<Vec<_>>()[1];

    // Parse the call_expression
    for child in call_expression.children(&mut cursor) {
        match child.kind() {
            "identifier" => name = get_node_text(&child, source_code)?,
            "member_expression" => {
                // Handle sys.description, sys.metadata, etc.
                name = get_node_text(&child, source_code)?;
            },
            "arguments" => {
                let mut gc_cursor = child.walk();
                for grandchild in child.children(&mut gc_cursor) {
                    match grandchild.kind() {
                        "(" | ")" | "," => {},
                        _ => {
                            if let Ok(Some(value)) = parse_value_node(grandchild, source_code) {
                                argument = value;
                            } else {
                                return Err(
                                    format!("Invalid decorator argument for {}", name).into()
                                );
                            }
                        },
                    }
                }
            },
            _ => {},
        }
    }

    Ok(BicepDecorator { name, argument })
}

// ---------------------------------------------------------------
// Constraint and Property Extraction
// ---------------------------------------------------------------

/// Extract numeric constraint from decorator argument.
///
/// Handles @minValue, @maxValue, @minLength, and @maxLength decorators.
///
/// # Arguments
///
/// * `decorator` - The decorator to extract the constraint from
///
/// # Returns
///
/// Optional numeric constraint value
pub fn extract_numeric_constraint(decorator: &BicepDecorator) -> Option<i64> {
    match &decorator.argument {
        BicepValue::Int(value) => Some(*value),
        BicepValue::String(s) => s.parse::<i64>().ok(),
        _ => None,
    }
}

/// Check if a decorator indicates a boolean flag (like @secure, @export).
///
/// # Arguments
///
/// * `decorator_name` - The name of the decorator to check
///
/// # Returns
///
/// True if this is a boolean flag decorator
pub fn is_boolean_flag_decorator(decorator_name: &str) -> bool {
    matches!(
        decorator_name,
        "secure" | "sys.secure" | "export" | "sys.export" | "sealed" | "sys.sealed"
    )
}

/// Extract metadata object from decorator, excluding description field.
///
/// # Arguments
///
/// * `decorator` - The decorator to extract metadata from
///
/// # Returns
///
/// Optional metadata map without the description field
pub fn extract_metadata_without_description(
    decorator: &BicepDecorator,
) -> Option<IndexMap<String, BicepValue>> {
    if let BicepValue::Object(map) = &decorator.argument {
        let mut metadata = map.clone();
        metadata.shift_remove("description");
        if !metadata.is_empty() {
            Some(metadata)
        } else {
            None
        }
    } else {
        None
    }
}

/// Process common decorators for parameters, outputs, etc.
///
/// Extracts description, metadata, and common constraints from a set of decorators.
///
/// # Arguments
///
/// * `decorators` - Slice of decorators to process
///
/// # Returns
///
/// Tuple containing (description, metadata, min_length, max_length, min_value, max_value, is_secure, is_sealed)
pub fn process_common_decorators(decorators: &[BicepDecorator]) -> CommonDecoratorsResult {
    let description = extract_description_from_decorators(decorators);
    let mut metadata = None;
    let mut min_length = None;
    let mut max_length = None;
    let mut min_value = None;
    let mut max_value = None;
    let mut is_secure = false;
    let mut is_sealed = false;

    for decorator in decorators {
        match decorator.name.as_str() {
            "metadata" | "sys.metadata" => {
                metadata = extract_metadata_without_description(decorator);
            },
            "minLength" | "sys.minLength" => {
                min_length = extract_numeric_constraint(decorator);
            },
            "maxLength" | "sys.maxLength" => {
                max_length = extract_numeric_constraint(decorator);
            },
            "minValue" | "sys.minValue" => {
                min_value = extract_numeric_constraint(decorator);
            },
            "maxValue" | "sys.maxValue" => {
                max_value = extract_numeric_constraint(decorator);
            },
            "secure" | "sys.secure" => {
                is_secure = true;
            },
            "sealed" | "sys.sealed" => {
                is_sealed = true;
            },
            _ => {
                debug!("Processing decorator: {}", decorator.name);
            },
        }
    }

    (
        description,
        metadata,
        min_length,
        max_length,
        min_value,
        max_value,
        is_secure,
        is_sealed,
    )
}
