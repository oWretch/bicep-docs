//! Type parsing utilities for Bicep files
//!
//! This module contains utilities for parsing various type expressions in Bicep,
//! including union types, array types, and property types.

use std::error::Error;

use tree_sitter::Node;

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
            "primitive_type" => {
                let type_text = super::get_node_text(&child, source_code)?;
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
                let type_name = super::get_node_text(&child, source_code)?;
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

    let mut cursor = node.walk();
    let children = node.children(&mut cursor).collect::<Vec<_>>();

    for child in &children {
        match child.kind() {
            "primitive_type" => {
                let type_text = super::get_node_text(&child, source_code)?;
                bicep_type = match type_text.as_str() {
                    "string" => BicepType::String,
                    "int" => BicepType::Int,
                    "bool" => BicepType::Bool,
                    "object" => BicepType::Object(None),
                    _ => BicepType::String,
                };
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
                let type_name = super::get_node_text(&child, source_code)?;
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
