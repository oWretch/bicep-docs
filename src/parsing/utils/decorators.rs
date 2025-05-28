//! Decorator parsing utilities
//!
//! This module provides functionality to parse Bicep decorators and extract
//! metadata from them. Decorators are used in Bicep to provide additional
//! information about parameters, resources, and other declarations.

use indexmap::IndexMap;
use std::error::Error;
use tracing::warn;
use tree_sitter::Node;

use crate::{BicepDecorator, BicepValue};
use super::text::get_node_text;

/// Extract description from decorators
///
/// This function searches through a list of decorators for description metadata
/// and returns the first description found.
///
/// # Arguments
///
/// * `decorators` - A vector of BicepDecorator objects to search
///
/// # Returns
///
/// An Option containing the description string if found, None otherwise
pub fn extract_description_from_decorators(
    decorators: &[BicepDecorator],
) -> Option<String> {
    // First, prioritize explicit description decorators
    for decorator in decorators {
        match decorator.name.as_str() {
            "description" | "sys.description" => {
                if let Some(args) = &decorator.arguments {
                    if let Some(BicepValue::String(desc_text)) = args.get(0) {
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
                if let Some(arguments) = &decorator.arguments {
                    if let Some(BicepValue::Object(map)) = arguments.get(0) {
                        if let Some(BicepValue::String(desc_text)) = map.get("description") {
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

/// Parse decorators from a decorators node
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

/// Parse a single decorator
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

                    // Handle string arguments
                    if (arg_content.starts_with('\'') && arg_content.ends_with('\''))
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
                        let mut obj_map = IndexMap::new();
                        let inner_content = &arg_content[1..arg_content.len() - 1].trim();

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
                                    let bool_value = value_part == "true";
                                    obj_map.insert(key, BicepValue::Bool(bool_value));
                                } else if let Ok(num) = value_part.parse::<i64>() {
                                    // Number value
                                    obj_map.insert(key, BicepValue::Int(num));
                                } else {
                                    // Default to string for other values
                                    obj_map.insert(key, BicepValue::String(value_part.to_string()));
                                }
                            }
                        }

                        argument = BicepValue::Object(obj_map);
                    } else {
                        // Default to string for other types
                        argument = BicepValue::String(arg_content.to_string());
                    }
                }
            }
        }
    }

    // If direct parsing failed, use the tree-sitter approach
    if name.is_empty() {
        let mut cursor = node.walk();
        let children = node.children(&mut cursor).collect::<Vec<_>>();

        // A decorator node should have a direct call_expression child
        for child in &children {
            if child.kind() == "call_expression" {
                // Process the call expression to extract name and arguments
                let (decorator_name, decorator_arg) = parse_call_expression(*child, source_code)?;
                name = decorator_name;
                argument = decorator_arg;
                break;
            }
        }

        // If we didn't find a call_expression, try to get the name from an identifier
        if name.is_empty() {
            for child in &children {
                if child.kind() == "identifier" {
                    name = get_node_text(*child, source_code);
                    break;
                } else if child.kind() == "decorator_name" || child.kind() == "member_expression" {
                    // Handle namespace qualified names like sys.description
                    name = get_node_text(*child, source_code);
                    break;
                }
            }
        }
    }

    Ok(BicepDecorator { 
        name, 
        arguments: Some(vec![argument]) 
    })
}

/// Parse a call expression (used for decorators like @description('text'))
fn parse_call_expression(
    node: Node,
    source_code: &str,
) -> Result<(String, BicepValue), Box<dyn Error>> {
    let mut name = String::new();
    let mut argument = BicepValue::String(String::new());
    let mut cursor = node.walk();

    // Get all children nodes
    let children = node.children(&mut cursor).collect::<Vec<_>>();

    // First child should be the function name (could be an identifier or member_expression)
    if !children.is_empty() {
        let function_node = children[0];
        if function_node.kind() == "identifier" {
            name = get_node_text(function_node, source_code);
        } else if function_node.kind() == "member_expression" {
            // Handle namespace qualified names like sys.description
            name = get_node_text(function_node, source_code);

            // For qualified names like sys.description, extract just the function name (description)
            if let Some(dot_pos) = name.rfind('.') {
                if dot_pos < name.len() - 1 {
                    // Keep the namespace (like "sys") as part of the name
                    // name = name[dot_pos+1..].to_string();
                }
            }
        }

        // Look for arguments
        for child in &children {
            if child.kind() == "arguments" {
                // Parse the arguments
                argument = parse_call_arguments(*child, source_code)?;
                break;
            }
        }
    }

    // Use the full node text as a fallback if we didn't get a name
    if name.is_empty() {
        let node_text = get_node_text(node, source_code);
        if node_text.starts_with('@') && node_text.contains('(') {
            // Extract name from format like @description('text')
            if let Some(open_paren) = node_text.find('(') {
                if open_paren > 1 {
                    name = node_text[1..open_paren].trim().to_string();
                }
            }
        }
    }

    Ok((name, argument))
}

/// Parse call arguments (the values inside parentheses in function/decorator calls)
fn parse_call_arguments(node: Node, source_code: &str) -> Result<BicepValue, Box<dyn Error>> {
    // First, try to get the full text of the arguments node
    let node_text = get_node_text(node, source_code);

    // If the node text starts with '(' and ends with ')', extract what's inside
    if node_text.starts_with('(') && node_text.ends_with(')') && node_text.len() > 2 {
        let content = node_text[1..node_text.len() - 1].trim();

        // Handle string arguments in quotes
        if (content.starts_with('\'') && content.ends_with('\''))
            || (content.starts_with('"') && content.ends_with('"'))
        {
            if content.len() > 2 {
                return Ok(BicepValue::String(
                    content[1..content.len() - 1].to_string(),
                ));
            }
            return Ok(BicepValue::String(String::new()));
        }

        // Handle numbers
        if let Ok(n) = content.parse::<i64>() {
            return Ok(BicepValue::Int(n));
        }

        // Handle booleans
        if content == "true" || content == "false" {
            if let Ok(b) = content.parse::<bool>() {
                return Ok(BicepValue::Bool(b));
            }
        }

        // Handle arrays and objects
        if content.starts_with('[') && content.ends_with(']') {
            // This is an array
            if content.len() > 2 {
                // Extract the content inside the array brackets
                let inner_content = &content[1..content.len() - 1].trim();

                // Split by commas - this is a simplistic approach that might need refinement
                let items = inner_content.split(',').collect::<Vec<_>>();

                let mut array_items = Vec::new();
                for item in items {
                    let trimmed_item = item.trim();

                    // Handle string values (strip quotes)
                    if (trimmed_item.starts_with('\'') && trimmed_item.ends_with('\''))
                        || (trimmed_item.starts_with('"') && trimmed_item.ends_with('"'))
                    {
                        if trimmed_item.len() >= 2 {
                            array_items.push(BicepValue::String(
                                trimmed_item[1..trimmed_item.len() - 1].to_string(),
                            ));
                        }
                    } else if trimmed_item == "true" || trimmed_item == "false" {
                        // Handle boolean values
                        if let Ok(b) = trimmed_item.parse::<bool>() {
                            array_items.push(BicepValue::Bool(b));
                        }
                    } else if let Ok(n) = trimmed_item.parse::<i64>() {
                        // Handle number values
                        array_items.push(BicepValue::Int(n));
                    } else {
                        // Default to string for other values
                        array_items.push(BicepValue::String(trimmed_item.to_string()));
                    }
                }

                return Ok(BicepValue::Array(array_items));
            } else {
                // Empty array
                return Ok(BicepValue::Array(Vec::new()));
            }
        } else if content.starts_with('{') && content.ends_with('}') {
            // This is an object
            let mut obj_map = IndexMap::new();
            // Try to extract description if this is a metadata object
            if content.contains("description:") {
                // Try to find the description value
                if let Some(desc_start) = content.find("description:") {
                    let desc_content = &content[desc_start + "description:".len()..];
                    // Look for a string after "description:"
                    if let Some(quote_start) = desc_content.find('\'') {
                        if let Some(quote_end) = desc_content[quote_start + 1..].find('\'') {
                            let desc = desc_content[quote_start + 1..quote_start + 1 + quote_end]
                                .to_string();
                            obj_map.insert("description".to_string(), BicepValue::String(desc));
                        }
                    }
                }
            }
            return Ok(BicepValue::Object(obj_map));
        }

        // Default to returning the content as a string
        return Ok(BicepValue::String(content.to_string()));
    }

    // Fall back to parsing child nodes if the direct text extraction fails
    let mut cursor = node.walk();
    let children = node.children(&mut cursor).collect::<Vec<_>>();

    // For simplicity, we'll just handle the first argument
    // In the future, this could be extended to handle multiple arguments
    if children.is_empty() {
        return Ok(BicepValue::String(String::new()));
    }

    let arg_node = children[0];

    match arg_node.kind() {
        "string" => {
            let text = get_node_text(arg_node, source_code);
            // Clean the string by removing quotes
            let clean_text = if (text.starts_with('\'') && text.ends_with('\''))
                || (text.starts_with('"') && text.ends_with('"'))
            {
                text[1..text.len() - 1].to_string()
            } else {
                text
            };
            Ok(BicepValue::String(clean_text))
        },
        "number" => {
            let text = get_node_text(arg_node, source_code);
            match text.parse::<i64>() {
                Ok(n) => Ok(BicepValue::Int(n)),
                Err(_) => Ok(BicepValue::String(text)),
            }
        },
        "boolean" | "true" | "false" => {
            let text = get_node_text(arg_node, source_code);
            match text.parse::<bool>() {
                Ok(b) => Ok(BicepValue::Bool(b)),
                Err(_) => Ok(BicepValue::String(text)),
            }
        },
        "array" => {
            use super::values::parse_array_items;
            let values = parse_array_items(arg_node, source_code)?;
            Ok(BicepValue::Array(values))
        },
        "object" => {
            use super::values::parse_object_properties_for_value;
            let props = parse_object_properties_for_value(arg_node, source_code)?;
            Ok(BicepValue::Object(props))
        },
        _ => {
            // For any other type, just use the node text
            let text = get_node_text(arg_node, source_code);
            Ok(BicepValue::String(text))
        },
    }
}
