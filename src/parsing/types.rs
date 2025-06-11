//! Type declaration parsing for Bicep files.
//!
//! This module handles the parsing of custom type declarations in Bicep files,
//! including object types, union types, and array types with their decorators
//! and validation constraints.

use std::error::Error;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use tracing::{debug, warn};
use tree_sitter::Node;

use super::{
    get_node_text,
    utils::{
        decorators::{extract_description_from_decorators, parse_decorator, parse_decorators},
        types::{parse_array_type, parse_property_type, parse_union_type},
    },
    BicepParameter, BicepParserError, BicepType, BicepValue,
};

// ---------------------------------------------------------------
// Structs, Enums & Types
// ---------------------------------------------------------------

/// Represents a custom type declaration in Bicep
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[skip_serializing_none]
pub struct BicepCustomType {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub definition: BicepType,
    #[serde(rename = "exported")]
    pub is_exported: bool,
    #[serde(rename = "secure")]
    pub is_secure: bool,
}

// ---------------------------------------------------------------
// Functions
// ---------------------------------------------------------------

/// Parses a tree-sitter Node representing a type declaration and converts it into a BicepCustomType.
///
/// This function handles the parsing of custom type declarations in Bicep files,
/// extracting the type name, definition, and metadata from decorators.
///
/// # Arguments
///
/// * `node` - The tree-sitter Node representing the type declaration
/// * `source_code` - The source code text containing the type declaration
///
/// # Returns
///
/// A Result containing a tuple of (type_name, BicepCustomType) if successful, or an error
/// if the parsing fails or required elements are missing.
///
/// # Errors
///
/// Returns an error if:
/// - The type declaration is missing an identifier
/// - The type definition cannot be parsed
/// - Invalid syntax is encountered
pub fn parse_type_declaration(
    node: Node,
    source_code: &str,
) -> Result<(String, BicepCustomType), Box<dyn Error>> {
    let description: Option<String> = None;
    let mut name = String::new();
    let mut definition = BicepType::Object(None); // Empty object type
    let is_secure = false;
    let is_exported = false;

    // Find the identifier (type name)
    let mut cursor = node.walk();
    let children = node.children(&mut cursor).collect::<Vec<_>>();
    for child in &children {
        if child.kind() == "identifier" {
            name = get_node_text(*child, source_code);
            debug!("Parsing type declaration for: {}", name);
            break;
        }
    }

    // Return early if we couldn't find a name
    if name.is_empty() {
        return Err(Box::new(BicepParserError::ParseError(
            "Type declaration missing identifier".to_string(),
        )));
    }

    // Find the type definition
    for child in &children {
        match child.kind() {
            "object_type" | "object" => {
                // This is an object type definition
                match parse_object_properties(*child, source_code) {
                    Ok(props) => {
                        // props is already IndexMap<String, BicepParameter>
                        definition = BicepType::Object(Some(props));
                    },
                    Err(e) => {
                        warn!("Failed to parse object properties for type {}: {}", name, e);
                        definition = BicepType::Object(None);
                    },
                };
            },
            "union_type" => {
                // This is a union type (like 'A' | 'B' | 'C')
                match parse_union_type(*child, source_code) {
                    Ok(union_type) => {
                        definition = union_type;
                    },
                    Err(e) => {
                        warn!("Failed to parse union type for {}: {}", name, e);
                        definition = BicepType::String; // Default to string on error
                    },
                }
            },
            "type" => {
                // For complex type nodes, iterate through children to find the actual type
                let mut type_cursor = child.walk();
                let type_children = child.children(&mut type_cursor).collect::<Vec<_>>();
                for type_child in &type_children {
                    match type_child.kind() {
                        "string" | "identifier" => {
                            let type_name = get_node_text(*type_child, source_code);
                            definition = match type_name.as_str() {
                                "string" => BicepType::String,
                                "int" => BicepType::Int,
                                "boolean" => BicepType::Bool,
                                "object" => BicepType::Object(None),
                                _ => BicepType::CustomType(type_name),
                            };
                        },
                        "array_type" => {
                            // Handle array types
                            match parse_array_type(*type_child, source_code) {
                                Ok(inner_type) => {
                                    definition = BicepType::Array(Box::new(inner_type))
                                },
                                Err(e) => {
                                    warn!("Failed to parse array type for {}: {}", name, e);
                                    definition = BicepType::Array(Box::new(BicepType::String));
                                    // Default to string[] on error
                                },
                            }
                        },
                        "object_type" => {
                            // Handle inline object types
                            match parse_object_properties(*type_child, source_code) {
                                Ok(props) => {
                                    // props is already IndexMap<String, BicepParameter>
                                    definition = BicepType::Object(Some(props));
                                },
                                Err(e) => {
                                    warn!(
                                        "Failed to parse inline object properties for type {}: {}",
                                        name, e
                                    );
                                    definition = BicepType::Object(None);
                                },
                            };
                        },
                        _ => {},
                    }
                }
            },
            "string" | "identifier" => {
                // This is a simple type reference
                let type_name = get_node_text(*child, source_code);
                definition = match type_name.as_str() {
                    "string" => BicepType::String,
                    "int" => BicepType::Int,
                    "boolean" => BicepType::Bool,
                    "object" => BicepType::Object(None), // Empty object type
                    _ => BicepType::CustomType(type_name),
                };
            },
            _ => {},
        }
    }

    // Note: We no longer need to check for properties here since they are now stored directly in BicepType::Object

    Ok((
        name,
        BicepCustomType {
            definition,
            description,
            is_secure,
            is_exported,
        },
    ))
}

/// Parse object properties from an object_type node
pub fn parse_object_properties(
    node: Node,
    source_code: &str,
) -> Result<IndexMap<String, BicepParameter>, Box<dyn Error>> {
    let mut properties = IndexMap::new();
    let mut property_decorators = IndexMap::new();
    let mut cursor = node.walk();
    let children = node.children(&mut cursor).collect::<Vec<_>>();

    // First pass: collect all decorators for properties
    let mut i = 0;
    while i < children.len() {
        if children[i].kind() == "decorators" {
            // Look ahead for the next object_property (could be multiple decorators in a row)
            let mut j = i + 1;
            while j < children.len() && children[j].kind() == "decorators" {
                j += 1;
            }

            // If we found an object_property, associate all decorators with it
            if j < children.len() && children[j].kind() == "object_property" {
                property_decorators.insert(j, children[i]);
            }
        }
        i += 1;
    }

    // Second pass: process all properties
    for (i, child) in children.iter().enumerate() {
        if child.kind() == "object_property" {
            let (name, mut property) = parse_object_property(*child, source_code)?;

            // Add any decorators found in the first pass
            if let Some(dec_node) = property_decorators.get(&i) {
                if let Ok(decorators) = parse_decorators(*dec_node, source_code) {
                    // Check if we got a description from any decorator
                    let desc = extract_description_from_decorators(&decorators);
                    if desc.is_some() {
                        property.description = desc;
                    }

                    // Process decorators to extract constraint values
                    for decorator in &decorators {
                        match decorator.name.as_str() {
                            "minLength" | "sys.minLength" => {
                                if let BicepValue::Int(num) = &decorator.argument {
                                    property.min_length = Some(*num);
                                    debug!("Property {} has minLength: {}", name, num);
                                }
                            },
                            "maxLength" | "sys.maxLength" => {
                                if let BicepValue::Int(num) = &decorator.argument {
                                    property.max_length = Some(*num);
                                    debug!("Property {} has maxLength: {}", name, num);
                                }
                            },
                            "minValue" | "sys.minValue" => {
                                if let BicepValue::Int(num) = &decorator.argument {
                                    property.min_value = Some(*num);
                                    debug!("Property {} has minValue: {}", name, num);
                                }
                            },
                            "maxValue" | "sys.maxValue" => {
                                if let BicepValue::Int(num) = &decorator.argument {
                                    property.max_value = Some(*num);
                                    debug!("Property {} has maxValue: {}", name, num);
                                }
                            },
                            "secure" | "sys.secure" => {
                                property.is_secure = true;
                                debug!("Property {} is secure", name);
                            },
                            _ => {}, // Ignore other decorators
                        }
                    }
                }
            }

            if !name.is_empty() {
                properties.insert(name, property);
            }
        }
    }

    Ok(properties)
}

/// Parse a single object property
pub fn parse_object_property(
    node: Node,
    source_code: &str,
) -> Result<(String, BicepParameter), Box<dyn Error>> {
    let mut name = String::new();
    let mut property_type = BicepType::String; // Default type
    let mut description: Option<String> = None;
    let mut is_nullable = false;
    let mut is_secure = false;
    let mut min_length: Option<i64> = None;
    let mut max_length: Option<i64> = None;
    let mut min_value: Option<i64> = None;
    let mut max_value: Option<i64> = None;

    let mut cursor = node.walk();
    let children = node.children(&mut cursor).collect::<Vec<_>>();

    // Find the property name (identifier)
    for child in &children {
        if child.kind() == "identifier" {
            name = get_node_text(*child, source_code);
            debug!("Parsing object property with name: {}", name);
            break;
        }
    }

    // Find the property type and check for decorators
    let mut found_colon = false;
    for child in &children {
        if child.kind() == ":" {
            found_colon = true;
        } else if child.kind() == "type" {
            property_type = parse_property_type(*child, source_code)?;
        } else if child.kind() == "primitive_type" {
            // Handle direct primitive type
            let type_text = get_node_text(*child, source_code);
            property_type = match type_text.as_str() {
                "string" => BicepType::String,
                "int" => BicepType::Int,
                "bool" => BicepType::Bool,
                "object" => BicepType::Object(None),
                _ => BicepType::CustomType(type_text),
            };
        } else if child.kind() == "nullable_type" {
            // Handle nullable type like "string?"
            is_nullable = true;
            debug!("Property {} is nullable", name);
            let mut nullable_cursor = child.walk();
            let nullable_children = child.children(&mut nullable_cursor).collect::<Vec<_>>();
            for nullable_child in nullable_children {
                if nullable_child.kind() == "primitive_type" {
                    let type_text = get_node_text(nullable_child, source_code);
                    property_type = match type_text.as_str() {
                        "string" => BicepType::String,
                        "int" => BicepType::Int,
                        "bool" => BicepType::Bool,
                        "object" => BicepType::Object(None),
                        _ => BicepType::CustomType(type_text),
                    };
                } else if nullable_child.kind() == "identifier" {
                    let type_text = get_node_text(nullable_child, source_code);
                    property_type = BicepType::CustomType(type_text);
                }
            }
        } else if child.kind() == "identifier" && found_colon {
            // This is an identifier that comes after the colon, so it's the type name
            let type_text = get_node_text(*child, source_code);
            property_type = BicepType::CustomType(type_text);
        } else if child.kind() == "object" {
            // Handle direct object definition
            match parse_object_properties(*child, source_code) {
                Ok(props) => {
                    // props is already IndexMap<String, BicepParameter>
                    property_type = BicepType::Object(Some(props));
                },
                Err(e) => {
                    warn!(
                        "Failed to parse object properties for property {}: {}",
                        name, e
                    );
                    property_type = BicepType::Object(None);
                },
            }
        }
    }

    // Check for decorators at the property level
    // Need to check for child decorators nodes
    for child in &children {
        if child.kind() == "property_decorators" || child.kind() == "decorators" {
            let mut dec_cursor = child.walk();
            let decorator_children = child.children(&mut dec_cursor).collect::<Vec<_>>();

            for dec_child in decorator_children {
                if dec_child.kind() == "decorator" {
                    if let Ok(decorator) = parse_decorator(dec_child, source_code) {
                        debug!("Property {} has decorator: {}", name, decorator.name);

                        // Process decorators to extract constraint values and description
                        match decorator.name.as_str() {
                            "description" | "sys.description" => {
                                if let BicepValue::String(desc) = &decorator.argument {
                                    description = Some(desc.clone());
                                }
                            },
                            "minLength" | "sys.minLength" => {
                                if let BicepValue::Int(num) = &decorator.argument {
                                    min_length = Some(*num);
                                    debug!("Property {} has minLength: {}", name, *num);
                                }
                            },
                            "maxLength" | "sys.maxLength" => {
                                if let BicepValue::Int(num) = &decorator.argument {
                                    max_length = Some(*num);
                                    debug!("Property {} has maxLength: {}", name, *num);
                                }
                            },
                            "minValue" | "sys.minValue" => {
                                if let BicepValue::Int(num) = &decorator.argument {
                                    min_value = Some(*num);
                                    debug!("Property {} has minValue: {}", name, *num);
                                }
                            },
                            "maxValue" | "sys.maxValue" => {
                                if let BicepValue::Int(num) = &decorator.argument {
                                    max_value = Some(*num);
                                    debug!("Property {} has maxValue: {}", name, *num);
                                }
                            },
                            "secure" | "sys.secure" => {
                                is_secure = true;
                                debug!("Property {} is secure", name);
                            },
                            _ => {}, // Ignore other decorators
                        }
                    }
                }
            }
        }
    }

    // Check if type is nullable (optional)
    let node_text = get_node_text(node, source_code);
    if node_text.contains("?") {
        is_nullable = true;
        debug!("Property {} is nullable", name);
    }

    Ok((
        name.clone(),
        BicepParameter {
            description,
            metadata: IndexMap::new(),
            parameter_type: property_type,
            default_value: None,
            discriminator: None,
            allowed_values: None,
            is_nullable,
            is_sealed: false,
            is_secure,
            min_length,
            max_length,
            min_value,
            max_value,
        },
    ))
}
