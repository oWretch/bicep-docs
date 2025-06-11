//! Type parsing utilities for Bicep files
//!
//! This module contains utilities for parsing various type expressions in Bicep,
//! including union types, array types, and property types.

use std::error::Error;

use indexmap::IndexMap;
use tree_sitter::Node;

use super::super::BicepParameter;
use super::decorators::{
    extract_description_from_decorators, parse_decorators, process_common_decorators,
};
use crate::BicepType;

/// Parse a property type from a type node
///
/// This function extracts type information from property declarations,
/// handling various type expressions including primitives, arrays, and unions.
///
/// # Arguments
///
/// * `node` - The tree-sitter Node representing a type expression
/// * `source_code` - The source code text
///
/// # Returns
///
/// A Result containing the parsed BicepType or an error
pub fn parse_property_type(node: Node, source_code: &str) -> Result<BicepType, Box<dyn Error>> {
    let mut type_value: Option<BicepType> = None;
    let mut cursor = node.walk();
    let children = node.children(&mut cursor).collect::<Vec<_>>();

    for child in children {
        match child.kind() {
            "primitive_type" => {
                let type_text = super::get_node_text(&child, source_code)?;
                type_value = Some(match type_text.as_str() {
                    "string" => BicepType::String,
                    "int" => BicepType::Int,
                    "bool" => BicepType::Bool,
                    "object" => BicepType::Object(None),
                    _ => BicepType::String, // Default fallback
                });
            },
            "array_type" => {
                type_value = Some(parse_array_type(child, source_code)?);
            },
            "union_type" => {
                type_value = Some(parse_union_type(child, source_code)?);
            },
            "ambient_type_reference" | "type_reference" | "identifier" => {
                let type_name = super::get_node_text(&child, source_code)?;
                type_value = Some(BicepType::CustomType(type_name));
            },
            "member_expression" => {
                // Handle qualified type references like types.environmentCodes
                let type_name = super::get_node_text(&child, source_code)?;
                type_value = Some(BicepType::CustomType(type_name));
            },
            _ => {
                // Handle other type nodes or continue
            },
        }
    }

    if let Some(tv) = type_value {
        Ok(tv)
    } else {
        // If no specific type found, try to parse the node text directly
        let node_text = super::get_node_text(&node, source_code)?;
        match node_text.as_str() {
            "string" => Ok(BicepType::String),
            "int" => Ok(BicepType::Int),
            "bool" => Ok(BicepType::Bool),
            "object" => Ok(BicepType::Object(None)),
            _ => {
                if node_text.contains('|') {
                    // Try to parse as union type
                    parse_union_type(node, source_code)
                } else if node_text.ends_with("[]") {
                    // Try to parse as array type
                    parse_array_type(node, source_code)
                } else {
                    // Assume it's a custom type reference
                    Ok(BicepType::CustomType(node_text))
                }
            },
        }
    }
}

/// Parse a union type (like 'A' | 'B' | 'C')
///
/// Extracts individual type options from union type expressions,
/// handling quoted string literals and type references.
///
/// # Arguments
///
/// * `node` - The tree-sitter Node representing a union type
/// * `source_code` - The source code text
///
/// # Returns
///
/// A Result containing a BicepType::Union with the parsed options
pub fn parse_union_type(node: Node, source_code: &str) -> Result<BicepType, Box<dyn Error>> {
    let mut values = Vec::new();
    let mut cursor = node.walk();
    let children = node.children(&mut cursor).collect::<Vec<_>>();

    for child in children {
        match child.kind() {
            "string" => {
                let text = super::get_node_text(&child, source_code)?;
                // Remove surrounding quotes
                let clean_text = if (text.starts_with('"') && text.ends_with('"'))
                    || (text.starts_with('\'') && text.ends_with('\''))
                {
                    text[1..text.len() - 1].to_string()
                } else {
                    text
                };
                values.push(clean_text);
            },
            "primitive_type" | "identifier" | "type_reference" => {
                values.push(super::get_node_text(&child, source_code)?);
            },
            "|" => {
                // Skip the union operator
                continue;
            },
            _ => {
                // Handle other potential union members
                let text = super::get_node_text(&child, source_code)?;
                if !text.trim().is_empty() && text != "|" {
                    values.push(text);
                }
            },
        }
    }

    // If we didn't find values through tree structure, try parsing the text directly
    if values.is_empty() {
        let full_text = super::get_node_text(&node, source_code)?;
        if full_text.contains('|') {
            values = full_text
                .split('|')
                .map(|s| {
                    let trimmed = s.trim();
                    // Remove quotes if present
                    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
                        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
                    {
                        trimmed[1..trimmed.len() - 1].to_string()
                    } else {
                        trimmed.to_string()
                    }
                })
                .filter(|s| !s.is_empty())
                .collect();
        }
    }

    Ok(BicepType::Union(values))
}

/// Parse an array type (like string[])
///
/// Extracts the element type from array type expressions,
/// handling nested arrays and complex element types.
///
/// # Arguments
///
/// * `node` - The tree-sitter Node representing an array type
/// * `source_code` - The source code text
///
/// # Returns
///
/// A Result containing a BicepType::Array with the parsed element type
pub fn parse_array_type(node: Node, source_code: &str) -> Result<BicepType, Box<dyn Error>> {
    let mut inner_type = BicepType::String; // Default
    let mut cursor = node.walk();
    let children = node.children(&mut cursor).collect::<Vec<_>>();

    // Find the element type (the type before the [])
    for child in &children {
        match child.kind() {
            "type" => {
                // The array type contains a nested type node
                if let Ok((parsed_type, _)) = parse_type_node(*child, source_code) {
                    inner_type = parsed_type;
                    break;
                }
            },
            "primitive_type" => {
                let type_text = super::get_node_text(child, source_code)?;
                inner_type = match type_text.as_str() {
                    "string" => BicepType::String,
                    "int" => BicepType::Int,
                    "bool" => BicepType::Bool,
                    "object" => BicepType::Object(None),
                    _ => BicepType::String,
                };
                break;
            },
            "array_type" => {
                // Nested array
                inner_type = parse_array_type(*child, source_code)?;
                break;
            },
            "union_type" => {
                inner_type = parse_union_type(*child, source_code)?;
                break;
            },
            "identifier" | "type_reference" => {
                let type_name = super::get_node_text(child, source_code)?;
                inner_type = BicepType::CustomType(type_name);
                break;
            },
            _ => {},
        }
    }

    // If no child provided the type, try parsing from node text
    if matches!(inner_type, BicepType::String) && children.is_empty() {
        let node_text = super::get_node_text(&node, source_code)?;
        if node_text.ends_with("[]") {
            let element_text = &node_text[..node_text.len() - 2];
            inner_type = match element_text {
                "string" => BicepType::String,
                "int" => BicepType::Int,
                "bool" => BicepType::Bool,
                "object" => BicepType::Object(None),
                _ => BicepType::CustomType(element_text.to_string()),
            };
        }
    }

    Ok(BicepType::Array(Box::new(inner_type)))
}

/// Parse an inline object type definition with properties
///
/// Handles object type definitions that include property specifications,
/// such as those found in parameter declarations with inline types.
///
/// # Arguments
///
/// * `object_node` - The tree-sitter Node representing the object
/// * `source_code` - The source code text
///
/// # Returns
///
/// A Result containing an IndexMap of property names to BicepParameter definitions
pub fn parse_inline_object_type(
    object_node: Node,
    source_code: &str,
) -> Result<IndexMap<String, BicepParameter>, Box<dyn Error>> {
    let mut properties = IndexMap::new();
    let mut cursor = object_node.walk();
    let all_children = object_node.children(&mut cursor).collect::<Vec<_>>();

    // Group decorators with their following properties
    let mut i = 0;
    let mut pending_decorators = Vec::new();

    while i < all_children.len() {
        let child = all_children[i];
        match child.kind() {
            "decorators" => {
                // Parse decorators and add them to pending list
                if let Ok(decorators) = parse_decorators(child, source_code) {
                    pending_decorators.extend(decorators);
                }
            },
            "object_property" => {
                // Parse the property and apply any pending decorators
                if let Ok((prop_name, mut prop_param)) = parse_object_property(child, source_code) {
                    // Apply pending decorators to this property
                    if !pending_decorators.is_empty() {
                        // Extract description from decorators
                        if prop_param.description.is_none() {
                            prop_param.description =
                                extract_description_from_decorators(&pending_decorators);
                        }

                        // Process common decorators
                        let (
                            _,
                            metadata,
                            min_length,
                            max_length,
                            min_value,
                            max_value,
                            is_secure,
                            is_sealed,
                        ) = process_common_decorators(&pending_decorators);

                        if let Some(meta) = metadata {
                            prop_param.metadata = meta;
                        }
                        if let Some(min_len) = min_length {
                            prop_param.min_length = Some(min_len);
                        }
                        if let Some(max_len) = max_length {
                            prop_param.max_length = Some(max_len);
                        }
                        if let Some(min_val) = min_value {
                            prop_param.min_value = Some(min_val);
                        }
                        if let Some(max_val) = max_value {
                            prop_param.max_value = Some(max_val);
                        }
                        prop_param.is_secure = is_secure;
                        prop_param.is_sealed = is_sealed;

                        // Clear pending decorators
                        pending_decorators.clear();
                    }

                    properties.insert(prop_name, prop_param);
                }
            },
            "{" | "}" => {
                // Skip braces
            },
            _ => {
                // Skip other nodes
            },
        }
        i += 1;
    }

    Ok(properties)
}

/// Parse a single object property definition
///
/// Extracts the property name and type from object property nodes.
/// Handles both primitive types and nested object types recursively.
///
/// # Arguments
///
/// * `property_node` - The tree-sitter Node representing an object property
/// * `source_code` - The source code text
///
/// # Returns
///
/// A Result containing a tuple of (property_name, BicepParameter)
fn parse_object_property(
    property_node: Node,
    source_code: &str,
) -> Result<(String, BicepParameter), Box<dyn Error>> {
    let mut cursor = property_node.walk();
    let children = property_node.children(&mut cursor).collect::<Vec<_>>();

    if children.len() < 3 {
        return Err("Invalid object property structure".into());
    }

    // First child should be the property name (identifier)
    let prop_name = super::get_node_text(&children[0], source_code)?;

    // Third child should be the type (after the colon)
    let type_node = children[2];

    let mut param = BicepParameter::default();

    // Check if this is a nested object
    if type_node.kind() == "object" {
        // This is a nested object, parse it recursively
        if let Ok(nested_properties) = parse_inline_object_type(type_node, source_code) {
            param.parameter_type = BicepType::Object(Some(nested_properties));
            param.is_nullable = false;
        } else {
            // Fallback to generic object type
            param.parameter_type = BicepType::Object(None);
            param.is_nullable = false;
        }
    } else {
        // This is a primitive or other type
        let (prop_type, is_nullable) = parse_type_node(type_node, source_code)?;
        param.parameter_type = prop_type;
        param.is_nullable = is_nullable;
    }

    Ok((prop_name, param))
}

/// Parse a type node and return both the type and whether it's nullable
///
/// # Arguments
///
/// * `node` - The tree-sitter Node representing a type
/// * `source_code` - The source code text
///
/// # Returns
///
/// A Result containing a tuple of (BicepType, is_nullable)
pub fn parse_type_node(node: Node, source_code: &str) -> Result<(BicepType, bool), Box<dyn Error>> {
    let mut nullable = false;
    let mut bicep_type = BicepType::String; // Default

    // Handle the case where the node itself is a primitive_type
    if node.kind() == "primitive_type" {
        let type_text = super::get_node_text(&node, source_code)?;
        bicep_type = match type_text.as_str() {
            "string" => BicepType::String,
            "int" => BicepType::Int,
            "bool" => BicepType::Bool,
            "object" => BicepType::Object(None),
            _ => BicepType::String,
        };
        return Ok((bicep_type, nullable));
    }

    // Handle the case where the node itself is a nullable_type
    if node.kind() == "nullable_type" {
        nullable = true;
        let mut cursor = node.walk();
        let children = node.children(&mut cursor).collect::<Vec<_>>();
        for child in &children {
            if child.kind() != "?" {
                if let Ok((inner_type, _)) = parse_type_node(*child, source_code) {
                    bicep_type = inner_type;
                    break;
                }
            }
        }
        return Ok((bicep_type, nullable));
    }

    let mut cursor = node.walk();
    let children = node.children(&mut cursor).collect::<Vec<_>>();

    for child in &children {
        match child.kind() {
            "primitive_type" => {
                let type_text = super::get_node_text(child, source_code)?;
                bicep_type = match type_text.as_str() {
                    "string" => BicepType::String,
                    "int" => BicepType::Int,
                    "bool" => BicepType::Bool,
                    "object" => BicepType::Object(None),
                    _ => BicepType::String,
                };
            },
            "object" => {
                // This is an inline object type definition with properties
                if let Ok(properties) = parse_inline_object_type(*child, source_code) {
                    bicep_type = BicepType::Object(Some(properties));
                } else {
                    bicep_type = BicepType::Object(None);
                }
            },
            "array_type" => {
                bicep_type = parse_array_type(*child, source_code)?;
            },
            "union_type" => {
                bicep_type = parse_union_type(*child, source_code)?;
            },
            "nullable_type" => {
                nullable = true;
                // Parse the inner type
                if let Ok((inner_type, _)) = parse_type_node(*child, source_code) {
                    bicep_type = inner_type;
                }
            },
            "identifier" | "type_reference" => {
                let type_name = super::get_node_text(child, source_code)?;
                bicep_type = BicepType::CustomType(type_name);
            },
            "?" => {
                nullable = true;
            },
            _ => {},
        }
    }

    // If no specific type was found, try parsing the node text directly
    if matches!(bicep_type, BicepType::String) && children.is_empty() {
        let node_text = super::get_node_text(&node, source_code)?;
        if node_text.contains('|') {
            bicep_type = parse_union_type(node, source_code)?;
        } else if node_text.ends_with("[]") {
            bicep_type = parse_array_type(node, source_code)?;
        } else if node_text.ends_with('?') {
            nullable = true;
            let inner_text = &node_text[..node_text.len() - 1];
            bicep_type = match inner_text {
                "string" => BicepType::String,
                "int" => BicepType::Int,
                "bool" => BicepType::Bool,
                "object" => BicepType::Object(None),
                _ => BicepType::CustomType(inner_text.to_string()),
            };
        } else {
            bicep_type = match node_text.as_str() {
                "string" => BicepType::String,
                "int" => BicepType::Int,
                "bool" => BicepType::Bool,
                "object" => BicepType::Object(None),
                _ => BicepType::CustomType(node_text),
            };
        }
    }

    Ok((bicep_type, nullable))
}
