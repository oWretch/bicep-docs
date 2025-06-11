//! Value parsing utilities for Bicep files
//!
//! This module contains utilities for parsing various value expressions in Bicep,
//! including arrays, objects, and literal values.

use std::error::Error;

use indexmap::IndexMap;
use tree_sitter::Node;

use crate::BicepValue;

/// Parse an array value from array items
///
/// Extracts individual elements from array expressions,
/// handling nested arrays and various value types.
///
/// # Arguments
///
/// * `array_item_node` - The tree-sitter Node representing array items
/// * `source_code` - The source code text
///
/// # Returns
///
/// A Result containing a Vec of BicepValues
pub fn parse_array_items(
    array_item_node: Node,
    source_code: &str,
) -> Result<Vec<BicepValue>, Box<dyn Error>> {
    let mut items = Vec::new();
    let mut cursor = array_item_node.walk();
    let children = array_item_node.children(&mut cursor).collect::<Vec<_>>();

    for child in children {
        if child.kind() == "," {
            continue;
        }
        if let Some(value) = parse_value_node(child, source_code)? {
            items.push(value);
        }
    }

    Ok(items)
}

/// Parse a value node and return the corresponding BicepValue
///
/// This function handles various value types including strings, numbers,
/// booleans, arrays, objects, and expressions.
///
/// # Arguments
///
/// * `node` - The tree-sitter Node representing a value
/// * `source_code` - The source code text
///
/// # Returns
///
/// A Result containing the parsed BicepValue wrapped in Option, or None if parsing fails
pub fn parse_value_node(
    node: Node,
    source_code: &str,
) -> Result<Option<BicepValue>, Box<dyn Error>> {
    match node.kind() {
        "string" => {
            // For string nodes, look for string_content child nodes instead of using the entire text
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "string_content" {
                    let content = child.utf8_text(source_code.as_bytes())?.to_string();
                    return Ok(Some(BicepValue::String(content)));
                }
            }
            Err("No string_content child found".into())
        },
        "integer" => Ok(Some(BicepValue::Int(
            node.utf8_text(source_code.as_bytes())?
                .to_string()
                .parse::<i64>()?,
        ))),
        "boolean" => {
            let text = node.utf8_text(source_code.as_bytes())?.to_string();
            match text.as_str() {
                "true" => Ok(Some(BicepValue::Bool(true))),
                "false" => Ok(Some(BicepValue::Bool(false))),
                _ => Err("Invalid boolean value".into()),
            }
        },
        "array" => {
            let mut cursor = node.walk();
            let children = node.children(&mut cursor).collect::<Vec<_>>();

            for child in children {
                if child.kind() == "array_item" {
                    let items = parse_array_items(child, source_code)?;
                    return Ok(Some(BicepValue::Array(items)));
                }
            }
            // Empty array
            Ok(Some(BicepValue::Array(Vec::new())))
        },
        "object" => {
            let properties = parse_object_properties_for_value(node, source_code)?;
            Ok(Some(BicepValue::Object(properties)))
        },
        "identifier" => {
            let name = node.utf8_text(source_code.as_bytes())?.to_string();
            Ok(Some(BicepValue::String(name)))
        },
        "member_expression" => {
            let text = node.utf8_text(source_code.as_bytes())?.to_string();
            Ok(Some(BicepValue::String(text)))
        },
        "call_expression" => {
            let text = node.utf8_text(source_code.as_bytes())?.to_string();
            Ok(Some(BicepValue::String(text)))
        },
        "binary_expression" => {
            let text = node.utf8_text(source_code.as_bytes())?.to_string();
            Ok(Some(BicepValue::String(text)))
        },
        "unary_expression" => {
            let text = node.utf8_text(source_code.as_bytes())?.to_string();
            Ok(Some(BicepValue::String(text)))
        },
        "parenthesized_expression" => {
            let mut cursor = node.walk();
            let children = node.children(&mut cursor).collect::<Vec<_>>();

            for child in children {
                if child.kind() != "(" && child.kind() != ")" {
                    return parse_value_node(child, source_code);
                }
            }
            Ok(Some(BicepValue::String(
                node.utf8_text(source_code.as_bytes())?.to_string(),
            )))
        },
        "subscript_expression" => {
            let text = node.utf8_text(source_code.as_bytes())?.to_string();
            Ok(Some(BicepValue::String(text)))
        },
        "null" => Ok(Some(BicepValue::String("null".to_string()))),
        _ => {
            // For unknown node types, just get the text
            let text = node.utf8_text(source_code.as_bytes())?.to_string();
            Ok(Some(BicepValue::String(text)))
        },
    }
}

/// Parse object properties for value contexts
///
/// Extracts key-value pairs from object expressions,
/// handling various property types and nested structures.
///
/// # Arguments
///
/// * `object_node` - The tree-sitter Node representing an object
/// * `source_code` - The source code text
///
/// # Returns
///
/// A Result containing an IndexMap of property names to BicepValues
pub fn parse_object_properties_for_value(
    object_node: Node,
    source_code: &str,
) -> Result<IndexMap<String, BicepValue>, Box<dyn Error>> {
    let mut properties = IndexMap::new();
    let mut cursor = object_node.walk();
    let children = object_node.children(&mut cursor).collect::<Vec<_>>();

    for child in children {
        if child.kind() == "object_property" {
            let mut property_cursor = child.walk();
            let property_children = child.children(&mut property_cursor).collect::<Vec<_>>();

            let mut key: Option<String> = None;
            let mut value: Option<BicepValue> = None;

            for prop_child in property_children {
                match prop_child.kind() {
                    "identifier" | "string" => {
                        if key.is_none() {
                            key = Some(prop_child.utf8_text(source_code.as_bytes())?.to_string());
                        } else if value.is_none() {
                            value = parse_value_node(prop_child, source_code)?;
                        }
                    },
                    ":" => {
                        // Skip the colon separator
                        continue;
                    },
                    _ => {
                        if value.is_none() {
                            value = parse_value_node(prop_child, source_code)?;
                        }
                    },
                }
            }

            if let (Some(k), Some(v)) = (key, value) {
                properties.insert(k, v);
            }
        }
    }

    Ok(properties)
}
