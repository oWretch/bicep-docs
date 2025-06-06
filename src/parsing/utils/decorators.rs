//! Decorator parsing and extraction utilities for Bicep files.
//!
//! This module provides centralized utilities for parsing and processing decorators
//! across all Bicep declaration types. It handles common decorator patterns like
//! description extraction, metadata processing, and constraint validation.

use indexmap::IndexMap;
use std::error::Error;
use tracing::{debug, warn};
use tree_sitter::Node;

use super::super::{get_node_text, BicepDecorator, BicepValue};

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

    // Get the full text of the decorator for better parsing
    let full_text = get_node_text(node, source_code);

    // If it starts with @ and contains a parenthesis, try direct parsing
    if (full_text.starts_with('@') && full_text.contains('('))
        || (full_text.starts_with('@') && full_text.contains('{'))
    {
        let text_without_at = &full_text[1..];

        if let Some(open_paren) = text_without_at.find('(') {
            // Extract the name part (everything before the first parenthesis)
            name = text_without_at[0..open_paren].trim().to_string();

            // Try to extract the content inside parentheses
            if let Some(close_paren) = text_without_at.rfind(')') {
                if open_paren < close_paren {
                    let arg_content = text_without_at[open_paren + 1..close_paren].trim();

                    // Handle string arguments - check for triple quotes first
                    if arg_content.starts_with("'''") && arg_content.ends_with("'''") {
                        if arg_content.len() >= 6 {
                            argument = BicepValue::String(
                                arg_content[3..arg_content.len() - 3].to_string(),
                            );
                        }
                    } else if (arg_content.starts_with('\'') && arg_content.ends_with('\''))
                        || (arg_content.starts_with('"') && arg_content.ends_with('"'))
                    {
                        if arg_content.len() >= 2 {
                            argument = BicepValue::String(
                                arg_content[1..arg_content.len() - 1].to_string(),
                            );
                        }
                    } else if arg_content == "true" || arg_content == "false" {
                        // Handle boolean arguments
                        if let Ok(b) = arg_content.parse::<bool>() {
                            argument = BicepValue::Bool(b);
                        }
                    } else if let Ok(n) = arg_content.parse::<i64>() {
                        // Handle number arguments
                        argument = BicepValue::Int(n);
                    } else if arg_content.starts_with('{') && arg_content.ends_with('}') {
                        // Handle object arguments (especially metadata objects)
                        argument = parse_object_argument(arg_content)?;
                    } else {
                        // Default to string if we can't parse it
                        argument = BicepValue::String(arg_content.to_string());
                    }
                }
            }
        }
    } else if let Some(stripped) = full_text.strip_prefix('@') {
        // Simple decorator without arguments
        name = stripped.trim().to_string();
        argument = BicepValue::String(String::new());
    } else {
        // Try tree-sitter based parsing as fallback
        return parse_decorator_with_tree_sitter(node, source_code);
    }

    Ok(BicepDecorator { name, argument })
}

// ---------------------------------------------------------------
// Helper Functions
// ---------------------------------------------------------------

/// Parse object argument from string content.
fn parse_object_argument(arg_content: &str) -> Result<BicepValue, Box<dyn Error>> {
    let mut obj_map = IndexMap::new();
    let inner_content = arg_content[1..arg_content.len() - 1].trim();

    // Split by new lines or by commas for single-line objects
    let lines = if inner_content.contains('\n') {
        inner_content.split('\n').collect::<Vec<_>>()
    } else {
        inner_content.split(',').collect::<Vec<_>>()
    };

    // Process each line/property
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Split by colon to get key and value
        if let Some(colon_pos) = trimmed.find(':') {
            let key = trimmed[0..colon_pos].trim().to_string();
            let value_part = trimmed[colon_pos + 1..].trim();

            // Process the value based on its format
            if (value_part.starts_with('\'') && value_part.ends_with('\''))
                || (value_part.starts_with('"') && value_part.ends_with('"'))
            {
                // String value
                let str_value = value_part[1..value_part.len() - 1].to_string();
                obj_map.insert(key, BicepValue::String(str_value));
            } else if value_part == "true" || value_part == "false" {
                // Boolean value
                if let Ok(b) = value_part.parse::<bool>() {
                    obj_map.insert(key, BicepValue::Bool(b));
                }
            } else if let Ok(n) = value_part.parse::<i64>() {
                // Number value
                obj_map.insert(key, BicepValue::Int(n));
            } else {
                // Default to string
                obj_map.insert(key, BicepValue::String(value_part.to_string()));
            }
        }
    }

    Ok(BicepValue::Object(obj_map))
}

/// Fallback decorator parsing using tree-sitter traversal.
fn parse_decorator_with_tree_sitter(
    node: Node,
    source_code: &str,
) -> Result<BicepDecorator, Box<dyn Error>> {
    let mut name = String::new();
    let mut argument = BicepValue::String(String::new());
    let mut cursor = node.walk();

    let children = node.children(&mut cursor).collect::<Vec<_>>();
    for child in children {
        match child.kind() {
            "identifier" => {
                let text = get_node_text(child, source_code);
                if name.is_empty() {
                    name = text;
                }
            },
            "call_expression" => {
                if let Ok(call_result) = parse_call_expression(child, source_code) {
                    name = call_result.0;
                    argument = call_result.1;
                }
            },
            _ => {},
        }
    }

    if name.is_empty() {
        name = get_node_text(node, source_code);
        if name.starts_with('@') {
            name = name[1..].to_string();
        }
    }

    Ok(BicepDecorator { name, argument })
}

/// Parse a call expression for decorator arguments.
fn parse_call_expression(
    node: Node,
    source_code: &str,
) -> Result<(String, BicepValue), Box<dyn Error>> {
    let mut function_name = String::new();
    let mut argument = BicepValue::String(String::new());
    let mut cursor = node.walk();

    let children = node.children(&mut cursor).collect::<Vec<_>>();
    for child in children {
        match child.kind() {
            "identifier" => {
                if function_name.is_empty() {
                    function_name = get_node_text(child, source_code);
                }
            },
            "arguments" => {
                if let Ok(args) = parse_call_arguments(child, source_code) {
                    argument = args;
                }
            },
            _ => {},
        }
    }

    Ok((function_name, argument))
}

/// Parse call arguments from an arguments node.
fn parse_call_arguments(node: Node, source_code: &str) -> Result<BicepValue, Box<dyn Error>> {
    let mut cursor = node.walk();
    let children = node.children(&mut cursor).collect::<Vec<_>>();

    // Find the first non-punctuation child and parse it as the argument
    for child in children {
        if child.kind() != "(" && child.kind() != ")" && child.kind() != "," {
            return super::values::parse_value_node(child, source_code)?
                .ok_or_else(|| Box::<dyn Error>::from("Failed to parse argument value"));
        }
    }

    Ok(BicepValue::String(String::new()))
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
        "secure"
            | "sys.secure"
            | "export"
            | "sys.export"
            | "sealed"
            | "sys.sealed"
            | "batchSize"
            | "sys.batchSize"
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
