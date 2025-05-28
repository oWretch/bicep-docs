//! Bicep file parsing module
//!
//! This module provides functionality to parse Azure Bicep files using tree-sitter
//! and convert them into structured data representations.
//!
//! # Architecture
//!
//! - `mod.rs` - Core types, utilities, and document parsing
//! - `parameters.rs` - Parameter declaration parsing
//! - `resources.rs` - Resource declaration parsing  
//! - `types.rs` - Type definition parsing
//! - `variables.rs` - Variable declaration parsing
//! - `functions.rs` - Function declaration parsing
//! - `modules.rs` - Module declaration parsing
//! - `outputs.rs` - Output declaration parsing
//! - `imports.rs` - Import statement parsing

use indexmap::IndexMap;
use serde::{ser::SerializeMap, Deserialize, Serialize, Serializer};
use serde_with::skip_serializing_none;
use std::{error::Error, fmt};
use tracing::warn;
use tree_sitter::{Node, Tree};

mod functions;
mod imports;
mod modules;
mod outputs;
mod parameters;
mod resources;
mod types;
mod variables;
pub use functions::{BicepFunction, BicepFunctionArgument};
pub use imports::{parse_module_import, parse_namespace_import, BicepImport, BicepImportSymbol};
pub use modules::{parse_module_declaration, BicepModule, ModuleSource};
pub use outputs::{parse_output_declaration, BicepOutput};
pub use parameters::BicepParameter;
pub use resources::BicepResource;
pub use types::BicepCustomType;
pub use variables::BicepVariable;

// ---------------------------------------------------------------
// Structs, Enums & Types
// ---------------------------------------------------------------

/// Custom error types for Bicep parsing operations
#[derive(Debug, Clone, PartialEq)]
pub enum BicepParserError {
    /// Unknown node kind encountered during parsing
    UnknownKind(String),
    /// Invalid value found for a specific type
    InvalidValue { kind: String, reason: String },
    /// General parsing error
    ParseError(String),
}

impl fmt::Display for BicepParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BicepParserError::UnknownKind(kind) => write!(f, "Unknown kind: {}", kind),
            BicepParserError::InvalidValue { kind, reason } => {
                write!(f, "Invalid {} value: {}", kind, reason)
            },
            BicepParserError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl Error for BicepParserError {}

/// A complete Bicep document containing all parsed components
///
/// This structure represents the complete contents of a Bicep file after parsing,
/// including all declarations, imports, and metadata.
///
/// # Structure
///
/// - Metadata and scope information
/// - Import statements
/// - Type definitions
/// - Function definitions  
/// - Parameter declarations
/// - Variable declarations
/// - Resource declarations
/// - Module declarations
/// - Output declarations
#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
#[skip_serializing_none]
pub struct BicepDocument {
    /// Optional name of the document/template
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Optional description of the template's purpose
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Template metadata
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub metadata: IndexMap<String, BicepValue>,
    /// Target deployment scope (subscription, resourceGroup, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_scope: Option<String>,
    /// Import statements
    pub imports: Vec<BicepImport>,
    /// Custom type definitions
    pub types: IndexMap<String, BicepCustomType>,
    /// User-defined functions
    pub functions: IndexMap<String, BicepFunction>,
    /// Template parameters
    pub parameters: IndexMap<String, BicepParameter>,
    /// Template variables
    pub variables: IndexMap<String, BicepVariable>,
    /// Resource declarations
    pub resources: IndexMap<String, BicepResource>,
    /// Module declarations
    pub modules: IndexMap<String, BicepModule>,
    /// Template outputs
    pub outputs: IndexMap<String, BicepOutput>,
}

/// Type system for Bicep parameters and variables
///
/// Represents the various types available in Bicep, including:
/// - Primitive types (string, int, bool)
/// - Complex types (arrays, objects)  
/// - Custom type references
/// - Union types for multiple allowed values
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum BicepType {
    /// Array type with element type specification
    Array(Box<BicepType>),
    /// String primitive type
    String,
    /// Numeric primitive type (int in Bicep)
    Int,
    /// Boolean primitive type
    Bool,
    /// Object type - None for generic objects, Some for structured objects
    Object(Option<IndexMap<String, BicepParameter>>),
    /// Reference to a custom type by name
    CustomType(String),
    /// Union type allowing multiple specific values
    Union(Vec<String>),
}

// Implement Display trait for BicepType for debugging and string conversion
impl std::fmt::Display for BicepType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BicepType::Array(inner_type) => write!(f, "{}[]", inner_type),
            BicepType::String => write!(f, "string"),
            BicepType::Int => write!(f, "int"),
            BicepType::Bool => write!(f, "bool"),
            BicepType::Object(Some(_params)) => write!(f, "object"),
            BicepType::Object(None) => write!(f, "object"),
            BicepType::CustomType(name) => write!(f, "{}", name),
            BicepType::Union(values) => {
                // Join values with " | " for display
                write!(f, "{}", values.join(" | "))
            },
        }
    }
}

// Custom serialize implementation for BicepType that will allow us
// to handle special cases like Union types correctly
impl Serialize for BicepType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            // Case 1: Generic object type with no properties serializes as "object"
            BicepType::Object(None) => "object".serialize(serializer),
            // Case 3: Inline specific object type with properties serializes as its structure
            BicepType::Object(Some(properties)) => {
                // For inline object definitions, serialize the structure
                let mut map = serializer.serialize_map(Some(properties.len()))?;
                for (key, param) in properties {
                    map.serialize_entry(key, &param)?;
                }
                map.end()
            },
            // Case 2: Custom type references serialize as their name string
            BicepType::CustomType(name) => name.clone().serialize(serializer),
            // Handle union types specially - just output the joined string without "type:" prefix
            BicepType::Union(values) => values.join(" | ").serialize(serializer),
            // All other types serialize as strings
            _ => self.to_string().serialize(serializer),
        }
    }
}

/// Value types that can be stored in Bicep variables and parameters
///
/// Represents runtime values in Bicep templates, including:
/// - Primitive values (strings, numbers, booleans)
/// - Complex values (arrays, objects)
/// - Identifier references to other template elements
#[derive(Debug, Clone, PartialEq)]
pub enum BicepValue {
    /// Array of values
    Array(Vec<BicepValue>),
    /// String literal value
    String(String),
    /// Numeric value (integer)
    Int(i64),
    /// Boolean value
    Bool(bool),
    /// Object with key-value pairs
    Object(IndexMap<String, BicepValue>),
    /// Reference to another identifier in the template
    Identifier(String),
}

// Implement a custom serializer for BicepValue to avoid YAML tags
impl Serialize for BicepValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            BicepValue::Array(arr) => arr.serialize(serializer),
            BicepValue::String(s) => s.serialize(serializer),
            BicepValue::Int(n) => n.serialize(serializer),
            BicepValue::Bool(b) => b.serialize(serializer),
            BicepValue::Object(map) => map.serialize(serializer),
            BicepValue::Identifier(id) => {
                // Serialize identifiers as references with a special format that makes it clear this is a reference
                let reference = format!("{{reference:{}}}", id);
                reference.serialize(serializer)
            },
        }
    }
}

// Implement a custom deserializer for BicepValue
impl<'de> Deserialize<'de> for BicepValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, SeqAccess, Visitor};
        use std::fmt;

        struct BicepValueVisitor;

        impl<'de> Visitor<'de> for BicepValueVisitor {
            type Value = BicepValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter
                    .write_str("a string, number, boolean, sequence, map or identifier reference")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(BicepValue::Bool(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(BicepValue::Int(value))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(BicepValue::Int(value as i64))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                // Check if this is an identifier reference in our special format
                if value.starts_with("{reference:") && value.ends_with("}") {
                    let id = value[11..value.len() - 1].to_string();
                    return Ok(BicepValue::Identifier(id));
                }
                Ok(BicepValue::String(value.to_string()))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                // Check if this is an identifier reference in our special format
                if value.starts_with("{reference:") && value.ends_with("}") {
                    let id = value[11..value.len() - 1].to_string();
                    return Ok(BicepValue::Identifier(id));
                }
                Ok(BicepValue::String(value))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut values = Vec::new();
                while let Some(value) = seq.next_element()? {
                    values.push(value);
                }
                Ok(BicepValue::Array(values))
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut values = IndexMap::new();
                while let Some((key, value)) = map.next_entry()? {
                    values.insert(key, value);
                }
                Ok(BicepValue::Object(values))
            }
        }

        deserializer.deserialize_any(BicepValueVisitor)
    }
}

// Implement Display trait for BicepValue to provide human-readable string representation
impl std::fmt::Display for BicepValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BicepValue::String(s) => write!(f, "{}", s),
            BicepValue::Int(n) => write!(f, "{}", n),
            BicepValue::Bool(b) => write!(f, "{}", b),
            BicepValue::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            },
            BicepValue::Object(map) => {
                write!(f, "{{")?;
                for (i, (key, value)) in map.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, value)?;
                }
                write!(f, "}}")
            },
            BicepValue::Identifier(id) => write!(f, "${{{}}}", id),
        }
    }
}

/// Represents a decorator applied to Bicep declarations
///
/// Decorators provide metadata and constraints for parameters, resources,
/// and other declarations. Examples include @description(), @secure, @minLength().
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BicepDecorator {
    /// The decorator name (e.g., "description", "secure")
    pub name: String,
    /// The decorator's argument value
    pub argument: BicepValue,
}

// ---------------------------------------------------------------
// Functions
// ---------------------------------------------------------------

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
pub(crate) fn extract_description_from_decorators(
    decorators: &Vec<BicepDecorator>,
) -> Option<String> {
    // First, prioritize explicit description decorators
    for decorator in decorators {
        match decorator.name.as_str() {
            "description" | "sys.description" => {
                if let BicepValue::String(desc_text) = &decorator.argument {
                    return Some(desc_text.clone());
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
                        return Some(desc_text.clone());
                    }
                }
            },
            _ => {},
        }
    }

    None
}

/// Parse a property type from a type node
pub(crate) fn parse_property_type(
    node: Node,
    source_code: &str,
) -> Result<BicepType, Box<dyn Error>> {
    let mut type_value: Option<BicepType> = None;
    let mut cursor = node.walk();
    let children = node.children(&mut cursor).collect::<Vec<_>>();

    for child in children {
        match child.kind() {
            "identifier" => {
                // Handle custom type references like "NetworkConfig"
                let type_name = get_node_text(child, source_code);
                // Check primitive types represented as identifiers
                match type_name.as_str() {
                    "string" => type_value = Some(BicepType::String),
                    "int" => type_value = Some(BicepType::Int),
                    "bool" => type_value = Some(BicepType::Bool),
                    "object" => type_value = Some(BicepType::Object(None)),
                    _ => type_value = Some(BicepType::CustomType(type_name)),
                }
            },
            "primitive_type" => {
                // Fixed from "primative_type"
                let type_name = get_node_text(child, source_code);
                match type_name.as_str() {
                    "string" => type_value = Some(BicepType::String),
                    "int" => type_value = Some(BicepType::Int),
                    "bool" => type_value = Some(BicepType::Bool),
                    _ => Err(Box::new(BicepParserError::ParseError(format!(
                        "Unknown primitive type: {}",
                        type_name
                    ))))?,
                }
            },
            "array_type" => {
                // Handle array types like string[]
                let inner_type = parse_array_type(child, source_code)?;
                type_value = Some(BicepType::Array(Box::new(inner_type)));
            },
            "object_type" => {
                // Handle inline object type definitions (Case 3: Inline object parameter)
                let properties = types::parse_object_properties(child, source_code)?;
                // Create an object type with the parsed properties
                let mut param_props = IndexMap::new();
                for (name, prop) in &properties {
                    let param = BicepParameter {
                        parameter_type: prop.parameter_type.clone(),
                        description: prop.description.clone(),
                        is_nullable: prop.is_nullable,
                        ..Default::default()
                    };
                    param_props.insert(name.clone(), param);
                }
                type_value = Some(BicepType::Object(Some(param_props)));
            },
            _ => Err(Box::new(BicepParserError::ParseError(format!(
                "Unknown type node: {}",
                child.kind()
            ))))?,
        }
    }

    if let Some(tv) = type_value {
        return Ok(tv);
    }
    Err(Box::new(BicepParserError::ParseError(
        "Failed to parse property type".to_string(),
    )))
}

/// Parse a union type (like 'A' | 'B' | 'C')
pub(crate) fn parse_union_type(node: Node, source_code: &str) -> Result<BicepType, Box<dyn Error>> {
    let mut values = Vec::new();
    let mut cursor = node.walk();
    let children = node.children(&mut cursor).collect::<Vec<_>>();

    for child in children {
        if child.kind() == "string" {
            let value = get_node_text(child, source_code);
            // Strip quotes if present
            let stripped = if value.starts_with('\'') && value.ends_with('\'') {
                value[1..value.len() - 1].to_string()
            } else {
                value
            };
            values.push(stripped);
        }
    }

    Ok(BicepType::Union(values))
}

/// Parse an array type (like string[])
pub(crate) fn parse_array_type(node: Node, source_code: &str) -> Result<BicepType, Box<dyn Error>> {
    let mut inner_type = BicepType::String; // Default
    let mut cursor = node.walk();
    let children = node.children(&mut cursor).collect::<Vec<_>>();

    // Find the element type (the type before the [])
    for child in &children {
        if child.kind() == "identifier" {
            let type_name = get_node_text(*child, source_code);
            match type_name.as_str() {
                "string" => inner_type = BicepType::String,
                "int" => inner_type = BicepType::Int,
                "boolean" | "bool" => inner_type = BicepType::Bool,
                "object" => inner_type = BicepType::Object(None),
                _ => inner_type = BicepType::CustomType(type_name),
            }
        } else if child.kind() == "type" {
            // Handle embedded type node (e.g., customObject[])
            let (element_type, _) = parse_type_node(*child, source_code)?;
            inner_type = element_type;
        } else if child.kind() == "array_type" {
            // Nested array (like string[][])
            inner_type = parse_array_type(*child, source_code)?;
        } else if child.kind() == "object_type" {
            // Array of objects
            let properties = types::parse_object_properties(*child, source_code)?;
            // properties is already IndexMap<String, BicepParameter>
            inner_type = BicepType::Object(Some(properties));
        }
    }

    Ok(inner_type)
}

/// Helper function to parse any value node
///
/// # Arguments
///
/// * `node` - The tree-sitter Node representing a value
/// * `source_code` - The source code text
///
/// # Returns
///
/// A Result containing an Option with a BicepValue if successful, or an error
pub(crate) fn parse_value_node(
    node: Node,
    source_code: &str,
) -> Result<Option<BicepValue>, Box<dyn Error>> {
    match node.kind() {
        "string" => {
            let text = get_node_text(node, source_code);
            if text.is_empty() {
                return Ok(Some(BicepValue::String(String::new())));
            }

            // Handle different string formats
            let clean_text = if text.starts_with('\'') && text.ends_with('\'') && text.len() >= 2 {
                // Single-quoted strings
                let content = &text[1..text.len() - 1];
                process_escape_sequences(content)
            } else if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
                // Double-quoted strings
                let content = &text[1..text.len() - 1];
                process_escape_sequences(content)
            } else if text.starts_with("'''") && text.ends_with("'''") && text.len() >= 6 {
                // Multi-line strings - already processed by get_node_text
                // Just strip the triple quotes
                let content = &text[3..text.len() - 3];
                process_escape_sequences(content)
            } else {
                // For other formats, use the text as is
                text
            };

            Ok(Some(BicepValue::String(clean_text)))
        },
        "number" => {
            let text = get_node_text(node, source_code);
            match text.parse::<i64>() {
                Ok(num) => Ok(Some(BicepValue::Int(num))),
                Err(e) => {
                    warn!("Failed to parse number '{}': {}", text, e);
                    // Fallback to string if number parsing fails
                    Ok(Some(BicepValue::String(text)))
                },
            }
        },
        "boolean" => {
            let text = get_node_text(node, source_code);
            match text.to_lowercase().as_str() {
                "true" => Ok(Some(BicepValue::Bool(true))),
                "false" => Ok(Some(BicepValue::Bool(false))),
                _ => {
                    warn!("Invalid boolean value: {}", text);
                    Ok(Some(BicepValue::String(text)))
                },
            }
        },
        "array" => {
            // Process array values
            match parse_array_items(node, source_code) {
                Ok(array_items) => Ok(Some(BicepValue::Array(array_items))),
                Err(e) => {
                    warn!("Failed to parse array: {}", e);
                    // Return empty array on error instead of failing
                    Ok(Some(BicepValue::Array(Vec::new())))
                },
            }
        },
        "object" => {
            // Process object properties
            match parse_object_properties_for_value(node, source_code) {
                Ok(obj_props) => Ok(Some(BicepValue::Object(obj_props))),
                Err(e) => {
                    warn!("Failed to parse object: {}", e);
                    // Return empty object on error instead of failing
                    Ok(Some(BicepValue::Object(IndexMap::new())))
                },
            }
        },
        "identifier" => {
            // This is a reference to another identifier/variable
            let text = get_node_text(node, source_code);
            if !text.is_empty() {
                return Ok(Some(BicepValue::Identifier(text)));
            }
            Ok(Some(BicepValue::String(String::new())))
        },
        _ => {
            // For unknown types, just use the node text
            let text = get_node_text(node, source_code);
            if text.is_empty() {
                warn!("Unknown node kind '{}' with empty text", node.kind());
            }
            Ok(Some(BicepValue::String(text)))
        },
    }
}

/// Helper function to parse array items
///
/// # Arguments
///
/// * `node` - The tree-sitter Node representing an array
/// * `source_code` - The source code text
///
/// # Returns
///
/// A Result containing a vector of BicepValue items if successful, or an error
pub(crate) fn parse_array_items(
    node: Node,
    source_code: &str,
) -> Result<Vec<BicepValue>, Box<dyn Error>> {
    let mut items = Vec::new();
    let mut cursor = node.walk();

    // Iterate through all child nodes
    for child in node.children(&mut cursor) {
        // Skip brackets and commas, process only value nodes
        match child.kind() {
            "[" | "]" | "," => continue,
            _ => {
                if let Ok(Some(value)) = parse_value_node(child, source_code) {
                    items.push(value);
                } else {
                    // Log a warning but continue parsing other items
                    warn!(
                        "Could not parse array item: {}",
                        get_node_text(child, source_code)
                    );
                }
            },
        }
    }

    Ok(items)
}

/// Helper function to parse object properties for BicepValue
///
/// # Arguments
///
/// * `node` - The tree-sitter Node representing an object
/// * `source_code` - The source code text
///
/// # Returns
///
/// A Result containing an IndexMap of property name to BicepValue if successful, or an error
pub(crate) fn parse_object_properties_for_value(
    node: Node,
    source_code: &str,
) -> Result<IndexMap<String, BicepValue>, Box<dyn Error>> {
    let mut properties = IndexMap::new();
    let mut cursor = node.walk();

    // Object node has child nodes that form key-value pairs
    let mut current_key: Option<String> = None;

    for child in node.children(&mut cursor) {
        match child.kind() {
            "{" | "}" | "," => continue, // Skip punctuation
            "object_property" => {
                // Each object property has an identifier (key) and a value
                let mut prop_cursor = child.walk();
                let mut key = String::new();
                let mut value: Option<BicepValue> = None;
                let mut found_colon = false;

                for prop_child in child.children(&mut prop_cursor) {
                    match prop_child.kind() {
                        "identifier" => {
                            if key.is_empty() {
                                // This is the key (first identifier)
                                key = get_node_text(prop_child, source_code);
                            } else if found_colon {
                                // This is an identifier value (after colon)
                                let identifier_text = get_node_text(prop_child, source_code);
                                value = Some(BicepValue::Identifier(identifier_text));
                            }
                        },
                        ":" => {
                            found_colon = true;
                            continue; // Skip colon
                        },
                        _ => {
                            // This should be the value (non-identifier)
                            if found_colon {
                                if let Ok(Some(parsed_value)) =
                                    parse_value_node(prop_child, source_code)
                                {
                                    value = Some(parsed_value);
                                } else {
                                    warn!("Failed to parse value for property '{}' in object", key);
                                }
                            }
                        },
                    }
                }

                // Add the key-value pair to our properties map
                if !key.is_empty() {
                    if let Some(val) = value {
                        properties.insert(key, val);
                    } else {
                        // If we have a key but no value, use an empty string value
                        warn!(
                            "Missing value for property '{}' in object, using empty string",
                            key
                        );
                        properties.insert(key, BicepValue::String(String::new()));
                    }
                }
            },
            "identifier" => {
                // This is a key in a key-value pair
                current_key = Some(get_node_text(child, source_code));
            },
            ":" => continue, // Skip colon
            _ => {
                // This is a value in a key-value pair
                if let Some(key) = current_key.take() {
                    if let Ok(Some(value)) = parse_value_node(child, source_code) {
                        properties.insert(key, value);
                    } else {
                        // If parsing fails, use an empty string value
                        warn!(
                            "Failed to parse value for property '{}' in object, using empty string",
                            key
                        );
                        properties.insert(key, BicepValue::String(String::new()));
                    }
                }
            },
        }
    }

    Ok(properties)
}

// parse_parameter_declaration function is now defined in parameters.rs

/// Parse decorators from a decorators node
pub(crate) fn parse_decorators(
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
fn parse_decorator(node: Node, source_code: &str) -> Result<BicepDecorator, Box<dyn Error>> {
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

    Ok(BicepDecorator { name, argument })
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
            let values = parse_array_items(arg_node, source_code)?;
            Ok(BicepValue::Array(values))
        },
        "object" => {
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

pub fn parse_bicep_document(
    tree: &Tree,
    source_code: &str,
) -> Result<BicepDocument, Box<dyn Error>> {
    let mut document = BicepDocument::default();
    let root_node = tree.root_node();
    let mut metadata: IndexMap<String, BicepValue> = IndexMap::new();
    let mut types: IndexMap<String, BicepCustomType> = IndexMap::new();
    let mut parameters: IndexMap<String, BicepParameter> = IndexMap::new();
    let mut variables: IndexMap<String, BicepVariable> = IndexMap::new();
    let mut functions: IndexMap<String, BicepFunction> = IndexMap::new();
    let mut resources: IndexMap<String, BicepResource> = IndexMap::new();
    let mut modules: IndexMap<String, BicepModule> = IndexMap::new();
    let mut imports: Vec<BicepImport> = Vec::new();
    let mut outputs: IndexMap<String, BicepOutput> = IndexMap::new();

    // Walk through all children of the root node
    let mut cursor = root_node.walk();
    let all_nodes = root_node.children(&mut cursor).collect::<Vec<_>>();

    // First pass - collect all decorators and associate them with their declarations
    let mut decorators_map: IndexMap<usize, Vec<Node>> = IndexMap::new();

    let mut i = 0;
    while i < all_nodes.len() {
        let node = all_nodes[i];
        if node.kind() == "decorators" {
            // Look ahead for the next non-decorator node (could be multiple decorators in a row)
            let mut j = i + 1;
            while j < all_nodes.len() && all_nodes[j].kind() == "decorators" {
                j += 1;
            }

            if j < all_nodes.len() {
                // We found a declaration after the decorators
                decorators_map.entry(j).or_default().push(node);
            }
        }
        i += 1;
    }

    // Second pass - process all nodes
    for (i, node) in all_nodes.iter().enumerate() {
        match node.kind() {
            "metadata_declaration" => {
                let (k, v) = parse_metadata(*node, source_code);
                if !k.is_empty() {
                    if let Some(val) = v {
                        metadata.insert(k, val);
                    }
                }
            },
            "target_scope_assignment" => {
                let scope_text = extract_target_scope(*node, source_code);
                if !scope_text.is_empty() {
                    document.target_scope = Some(scope_text);
                }
            },
            "type_declaration" => {
                // Get any decorators for this node
                let decorators_nodes = decorators_map.get(&i).cloned();

                // Parse custom type declaration
                match types::parse_type_declaration(*node, source_code) {
                    Ok((type_name, mut custom_type)) => {
                        // If we found decorators, parse them and add to the type
                        if let Some(dec_nodes) = decorators_nodes {
                            let mut all_decorators = Vec::new();

                            for dec_node in dec_nodes {
                                match parse_decorators(dec_node, source_code) {
                                    Ok(decorators) => {
                                        all_decorators.extend(decorators);
                                    },
                                    Err(e) => {
                                        warn!("Failed to parse decorator: {}", e);
                                    },
                                }
                            }

                            // Extract description if present
                            if custom_type.description.is_none() {
                                custom_type.description =
                                    extract_description_from_decorators(&all_decorators);
                            }

                            // Check for secure decorator
                            custom_type.is_secure =
                                all_decorators.iter().any(|d| d.name == "secure");

                            // Check for export decorator
                            custom_type.is_exported =
                                all_decorators.iter().any(|d| d.name == "export");
                        }

                        // Fix definition type for standard types
                        if let BicepType::CustomType(ref name) = custom_type.definition {
                            match name.as_str() {
                                "string" => custom_type.definition = BicepType::String,
                                "int" => custom_type.definition = BicepType::Int,
                                "boolean" => custom_type.definition = BicepType::Bool,
                                "object" => custom_type.definition = BicepType::Object(None),
                                _ => {}, // Keep as custom type
                            }
                        }

                        types.insert(type_name, custom_type);
                    },
                    Err(e) => {
                        warn!("Failed to parse type declaration: {}", e);
                    },
                }
            },
            "parameter_declaration" => {
                // Get any decorators for this node
                let decorators_nodes = decorators_map.get(&i).cloned();

                // Convert decorator nodes to BicepDecorator structs
                let mut all_decorators = Vec::new();
                if let Some(dec_nodes) = decorators_nodes {
                    for dec_node in dec_nodes {
                        match parse_decorators(dec_node, source_code) {
                            Ok(decorators) => {
                                all_decorators.extend(decorators);
                            },
                            Err(e) => {
                                warn!("Failed to parse decorator: {}", e);
                            },
                        }
                    }
                }

                // Parse parameter declaration
                match parameters::parse_parameter_declaration(*node, source_code, all_decorators) {
                    Ok((param_name, parameter)) => {
                        parameters.insert(param_name, parameter);
                    },
                    Err(e) => {
                        warn!("Failed to parse parameter declaration: {}", e);
                    },
                }
            },
            "variable_declaration" => {
                // Get any decorators for this node
                let decorators_nodes = decorators_map.get(&i).cloned();

                // Convert decorator nodes to BicepDecorator structs
                let mut all_decorators = Vec::new();
                if let Some(dec_nodes) = decorators_nodes {
                    for dec_node in dec_nodes {
                        match parse_decorators(dec_node, source_code) {
                            Ok(decorators) => {
                                all_decorators.extend(decorators);
                            },
                            Err(e) => {
                                warn!("Failed to parse decorator: {}", e);
                            },
                        }
                    }
                }

                // Parse variable declaration
                match variables::parse_variable_declaration(*node, source_code, all_decorators) {
                    Ok((var_name, variable)) => {
                        variables.insert(var_name, variable);
                    },
                    Err(e) => {
                        warn!("Failed to parse variable declaration: {}", e);
                    },
                }
            },
            "user_defined_function" | "function_declaration" => {
                // Get any decorators for this node
                let decorators_nodes = decorators_map.get(&i).cloned();

                // Convert decorator nodes to BicepDecorator structs
                let mut all_decorators = Vec::new();
                if let Some(dec_nodes) = decorators_nodes {
                    for dec_node in dec_nodes {
                        match parse_decorators(dec_node, source_code) {
                            Ok(decorators) => {
                                all_decorators.extend(decorators);
                            },
                            Err(e) => {
                                warn!("Failed to parse decorator: {}", e);
                            },
                        }
                    }
                }

                // Parse function declaration
                match functions::parse_function_declaration(*node, source_code, all_decorators) {
                    Ok((func_name, function)) => {
                        functions.insert(func_name, function);
                    },
                    Err(e) => {
                        warn!("Failed to parse function declaration: {}", e);
                    },
                }
            },
            "resource_declaration" => {
                // Get any decorators for this node
                let decorators_nodes = decorators_map.get(&i).cloned();

                // Convert decorator nodes to BicepDecorator structs
                let mut all_decorators = Vec::new();
                if let Some(dec_nodes) = decorators_nodes {
                    for dec_node in dec_nodes {
                        match parse_decorators(dec_node, source_code) {
                            Ok(decorators) => {
                                all_decorators.extend(decorators);
                            },
                            Err(e) => {
                                warn!("Failed to parse decorator: {}", e);
                            },
                        }
                    }
                }

                // Parse resource declaration
                match resources::parse_resource_declaration(*node, source_code, all_decorators) {
                    Ok(resource_list) => {
                        // Add all resources (main and child) to the document
                        for (resource_name, resource) in resource_list {
                            resources.insert(resource_name, resource);
                        }
                    },
                    Err(e) => {
                        warn!("Failed to parse resource declaration: {}", e);
                    },
                }
            },
            "module_declaration" => {
                // Get any decorators for this node
                let decorators_nodes = decorators_map.get(&i).cloned();

                // Convert decorator nodes to BicepDecorator structs
                let mut all_decorators = Vec::new();
                if let Some(dec_nodes) = decorators_nodes {
                    for dec_node in dec_nodes {
                        match parse_decorators(dec_node, source_code) {
                            Ok(decorators) => {
                                all_decorators.extend(decorators);
                            },
                            Err(e) => {
                                warn!("Failed to parse decorator: {}", e);
                            },
                        }
                    }
                }

                // Parse module declaration
                match parse_module_declaration(*node, source_code, all_decorators) {
                    Ok(module) => {
                        modules.insert(module.name.clone(), module);
                    },
                    Err(e) => {
                        warn!("Failed to parse module declaration: {}", e);
                    },
                }
            },
            "import_statement" => {
                // Parse namespace import statement
                match parse_namespace_import(*node, source_code) {
                    Ok(import) => {
                        imports.push(import);
                    },
                    Err(e) => {
                        warn!("Failed to parse namespace import: {}", e);
                    },
                }
            },
            "import_functionality" => {
                // Parse module import statement
                match parse_module_import(*node, source_code) {
                    Ok(import) => {
                        imports.push(import);
                    },
                    Err(e) => {
                        warn!("Failed to parse module import: {}", e);
                    },
                }
            },
            "output_declaration" => {
                // Get any decorators for this node
                let decorators_nodes = decorators_map.get(&i).cloned();

                // Convert decorator nodes to BicepDecorator structs
                let mut all_decorators = Vec::new();
                if let Some(dec_nodes) = decorators_nodes {
                    for dec_node in dec_nodes {
                        match parse_decorators(dec_node, source_code) {
                            Ok(decorators) => {
                                all_decorators.extend(decorators);
                            },
                            Err(e) => {
                                warn!("Failed to parse decorator: {}", e);
                            },
                        }
                    }
                }

                // Parse output declaration
                match parse_output_declaration(*node, source_code, all_decorators) {
                    Ok((name, output)) => {
                        outputs.insert(name, output);
                    },
                    Err(e) => {
                        warn!("Failed to parse output declaration: {}", e);
                    },
                }
            },
            _ => {},
        }
    }

    // Set document name and description from metadata if available
    if let Some(BicepValue::String(name)) = metadata.get("name") {
        document.name = Some(name.clone());
    }

    if let Some(BicepValue::String(desc)) = metadata.get("description") {
        document.description = Some(desc.clone());
    }

    // Remove name and description from metadata to avoid duplication
    metadata.shift_remove("name");
    metadata.shift_remove("description");

    // Set document metadata and types
    document.metadata = metadata;
    document.types = types;
    document.parameters = parameters;
    document.variables = variables;
    document.functions = functions;
    document.resources = resources;
    document.modules = modules;
    document.imports = imports;
    document.outputs = outputs;

    Ok(document)
}

/// Parse metadata nodes
fn parse_metadata(node: Node, source_code: &str) -> (String, Option<BicepValue>) {
    let mut name = String::new();
    let mut value = None;

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "identifier" => name = get_node_text(child, source_code),
            "string" | "number" | "boolean" => {
                let result = get_primitive_value(child, source_code);
                match result {
                    Ok(val) => value = Some(val),
                    Err(e) => warn!("Could not parse value of metadata {}: {}", name, e),
                }
            },
            "object" => {
                // Handle object values in metadata
                match parse_object_properties_for_value(child, source_code) {
                    Ok(properties) => value = Some(BicepValue::Object(properties)),
                    Err(e) => warn!("Could not parse object value for metadata {}: {}", name, e),
                }
            },
            "array" => {
                // Handle array values in metadata
                match parse_array_items(child, source_code) {
                    Ok(items) => value = Some(BicepValue::Array(items)),
                    Err(e) => warn!("Could not parse array value for metadata {}: {}", name, e),
                }
            },
            "=" | "metadata" => {
                // Skip the equals sign and metadata keyword
                continue;
            },
            _ => {
                // Try to parse as a general value node
                match parse_value_node(child, source_code) {
                    Ok(Some(parsed_value)) => value = Some(parsed_value),
                    Ok(None) => {
                        // No value parsed, but no error - continue
                        continue;
                    },
                    Err(_) => {
                        warn!("Unknown metadata value type {}", child.kind());
                    },
                }
            },
        }
    }

    (name, value)
}

/// Extract text from a tree-sitter Node safely with bounds checking
///
/// This function extracts text from a tree-sitter Node and handles common transformations
/// like stripping surrounding quotes and replacing consecutive duplicate quotes with single quotes.
///
/// # Security
///
/// - Validates UTF-8 encoding
/// - Performs bounds checking on source code access
/// - Limits recursion depth for nested string content
///
/// # Arguments
///
/// * `node` - The tree-sitter Node to extract text from
/// * `source_code` - The source code containing the node
///
/// # Returns
///
/// A String containing the node's text, possibly with transformations applied.
/// Returns an empty string if extraction fails with appropriate logging.
pub(crate) fn get_node_text(node: Node, source_code: &str) -> String {
    // Validate bounds before accessing source code
    if node.start_byte() > source_code.len() || node.end_byte() > source_code.len() {
        warn!("Node bounds exceed source code length");
        return String::new();
    }

    // For string nodes, handle multiline and escape sequences properly
    // For string nodes, we want to get the full string including quotes
    // so we can properly determine if it's a multiline string
    if node.kind() == "string" {
        if let Ok(text) = node.utf8_text(source_code.as_bytes()) {
            if text.starts_with("'''") && text.ends_with("'''") {
                // This is a multiline string
                return process_escape_sequences(text);
            } else if (text.starts_with('\'') && text.ends_with('\''))
                || (text.starts_with('"') && text.ends_with('"'))
            {
                // Single or double quoted string
                return process_escape_sequences(text);
            }
        }
    }

    // Check for string_content specifically
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "string_content" {
            if let Ok(text) = child.utf8_text(source_code.as_bytes()) {
                return process_escape_sequences(text);
            }
        }
    }

    // Fallback to the node's own text if no string_content child was found
    if let Ok(text) = node.utf8_text(source_code.as_bytes()) {
        text.to_string()
    } else {
        warn!("Failed to get UTF-8 text from node");
        String::new()
    }
}

/// Process escape sequences in a string according to Bicep's rules
///
/// # Performance
///
/// - Pre-allocates result string capacity when possible
/// - Uses efficient character iteration
/// - Avoids unnecessary string allocations
///
/// # Arguments
///
/// * `text` - The string content to process
///
/// # Returns
///
/// A String with escape sequences properly processed
fn process_escape_sequences(text: &str) -> String {
    // Determine the string format
    let is_multiline = text.starts_with("'''") && text.ends_with("'''");
    let is_single_quote = text.starts_with('\'') && text.ends_with('\'') && !is_multiline;
    let is_double_quote = text.starts_with('"') && text.ends_with('"');

    // Extract the content without quotes
    let content = if is_multiline && text.len() >= 6 {
        &text[3..text.len() - 3]
    } else if (is_single_quote || is_double_quote) && text.len() >= 2 {
        &text[1..text.len() - 1]
    } else {
        text // No quotes to remove
    };

    // Pre-allocate result string with content length as estimate
    let mut result = String::with_capacity(content.len());
    let mut chars = content.char_indices();

    while let Some((_, ch)) = chars.next() {
        // Check for escape sequences
        if ch == '\\' {
            if let Some((_, next_ch)) = chars.next() {
                match next_ch {
                    't' => result.push('\t'),
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    '\'' => result.push('\''),
                    'u' => {
                        // Unicode escape - check if it's in the format \u{...}
                        if let Some((_, '{')) = chars.next() {
                            let mut hex_digits = String::new();
                            let mut found_closing_brace = false;

                            // Collect hex digits until we find '}'
                            for (_, hex_ch) in chars.by_ref() {
                                if hex_ch == '}' {
                                    found_closing_brace = true;
                                    break;
                                } else if hex_ch.is_ascii_hexdigit() {
                                    hex_digits.push(hex_ch);
                                } else {
                                    // Invalid hex digit, break
                                    break;
                                }
                            }

                            if found_closing_brace && !hex_digits.is_empty() {
                                // Convert hex to Unicode character
                                if let Ok(code_point) = u32::from_str_radix(&hex_digits, 16) {
                                    if let Some(unicode_char) = std::char::from_u32(code_point) {
                                        result.push(unicode_char);
                                        continue;
                                    }
                                }
                            }
                        }

                        // If we get here, it's not a valid unicode escape
                        result.push('\\');
                        result.push('u');
                    },
                    _ => {
                        // Unknown escape sequence, treat as literal
                        result.push('\\');
                        result.push(next_ch);
                    },
                }
            } else {
                // Trailing backslash
                result.push('\\');
            }
        } else {
            // Regular character - properly handle UTF-8
            result.push(ch);
        }
    }

    result
}

/// Extract a primitive Bicep value from a node
fn get_primitive_value(node: Node, source_code: &str) -> Result<BicepValue, Box<dyn Error>> {
    let node_text = get_node_text(node, source_code);
    match node.kind() {
        "string" => {
            // Remove the surrounding quotes for string values
            let text = if node_text.len() >= 2 {
                let first_char = node_text.chars().next().unwrap();
                let last_char = node_text.chars().last().unwrap();

                if (first_char == '\'' && last_char == '\'')
                    || (first_char == '"' && last_char == '"')
                {
                    node_text[1..node_text.len() - 1].to_string()
                } else {
                    node_text
                }
            } else {
                node_text
            };
            Ok(BicepValue::String(text))
        },
        "number" => match node_text.parse::<i64>() {
            Ok(n) => Ok(BicepValue::Int(n)),
            Err(_) => Err(Box::new(BicepParserError::InvalidValue {
                kind: "number".to_string(),
                reason: format!("Could not parse '{}' as integer", node_text),
            })),
        },
        "boolean" => match node_text.parse::<bool>() {
            Ok(b) => Ok(BicepValue::Bool(b)),
            Err(_) => Err(Box::new(BicepParserError::InvalidValue {
                kind: "boolean".to_string(),
                reason: format!("Could not parse '{}' as boolean", node_text),
            })),
        },
        _ => Err(Box::new(BicepParserError::UnknownKind(format!(
            "Unknown primitive value type: {}",
            node.kind()
        )))),
    }
}

/// Extract target scope from declaration
fn extract_target_scope(node: Node, source_code: &str) -> String {
    let mut scope_text = String::new();

    let mut cursor = node.walk();
    let children = node.children(&mut cursor).collect::<Vec<_>>();
    for child in children {
        // For target_scope_assignment, the value is in a string node
        if child.kind() == "string" {
            scope_text = get_node_text(child, source_code);
            // Remove quotes if present
            if (scope_text.starts_with('\'') && scope_text.ends_with('\''))
                || (scope_text.starts_with('"') && scope_text.ends_with('"'))
            {
                scope_text = scope_text[1..scope_text.len() - 1].to_string();
            }
            break;
        }
    }

    scope_text
}

pub(crate) fn parse_type_node(
    node: Node,
    source_code: &str,
) -> Result<(BicepType, bool), Box<dyn Error>> {
    // Debug output to trace the node kind and text

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "identifier" => {
                // Handle custom type references (like NetworkConfig) or object
                let type_name = get_node_text(child, source_code);

                // Check if it's a primitive type with an identifier representation
                match type_name.as_str() {
                    // Case 1: Generic object parameter without properties
                    "object" => return Ok((BicepType::Object(None), false)),
                    "array" => return Ok((BicepType::Array(Box::new(BicepType::String)), false)),
                    "string" => return Ok((BicepType::String, false)),
                    "int" => return Ok((BicepType::Int, false)),
                    "bool" => return Ok((BicepType::Bool, false)),
                    // Case 2: Custom type reference
                    _ => return Ok((BicepType::CustomType(type_name), false)),
                }
            },
            "union_type" => {
                // Handle union type
                let union_type = parse_union_type(child, source_code)?;
                return Ok((union_type, false));
            },
            "primitive_type" => {
                let type_name = get_node_text(child, source_code);
                match type_name.as_str() {
                    "string" => return Ok((BicepType::String, false)),
                    "int" => return Ok((BicepType::Int, false)),
                    "bool" => return Ok((BicepType::Bool, false)),
                    "object" => return Ok((BicepType::Object(None), false)),
                    "array" => return Ok((BicepType::Array(Box::new(BicepType::String)), false)),
                    _ => {
                        return Err(Box::new(BicepParserError::ParseError(format!(
                            "Unknown primitive type: {}",
                            type_name
                        ))));
                    },
                }
            },
            "array_type" => {
                // Handle array type
                let element_type = parse_array_type(child, source_code)?;
                return Ok((BicepType::Array(Box::new(element_type)), false));
            },
            "object" => {
                // Handle object type
                let properties = types::parse_object_properties(child, source_code)?;
                // properties is already IndexMap<String, BicepParameter>
                return Ok((BicepType::Object(Some(properties)), false));
            },
            "nullable_type" => {
                // Handle nullable type
                // Process the child of the nullable_type node, which contains the actual type
                let mut nullable_cursor = child.walk();
                let nullable_children = child.children(&mut nullable_cursor).collect::<Vec<_>>();

                if let Some(type_child) = nullable_children.first() {
                    match type_child.kind() {
                        "primitive_type" => {
                            let type_name = get_node_text(*type_child, source_code);
                            match type_name.as_str() {
                                "string" => return Ok((BicepType::String, true)),
                                // Explicitly use Integer type for int
                                "int" => {
                                    return Ok((BicepType::Int, true));
                                },
                                "bool" => return Ok((BicepType::Bool, true)),
                                "object" => return Ok((BicepType::Object(None), true)),
                                _ => {
                                    return Err(Box::new(BicepParserError::ParseError(format!(
                                        "Unknown nullable primitive type: {}",
                                        type_name
                                    ))));
                                },
                            }
                        },
                        "identifier" => {
                            let type_name = get_node_text(*type_child, source_code);
                            match type_name.as_str() {
                                "string" => return Ok((BicepType::String, true)),
                                "int" => return Ok((BicepType::Int, true)),
                                "bool" => return Ok((BicepType::Bool, true)),
                                "object" => return Ok((BicepType::Object(None), true)),
                                _ => return Ok((BicepType::CustomType(type_name), true)),
                            }
                        },
                        _ => {
                            let (base_type, _) = parse_type_node(*type_child, source_code)?;
                            return Ok((base_type, true));
                        },
                    }
                } else {
                    return Err(Box::new(BicepParserError::ParseError(
                        "Nullable type has no child type node".to_string(),
                    )));
                }
            },
            // Handle node directly if it's a type specification
            "type" => {
                return parse_type_node(child, source_code);
            },
            "member_expression" => {
                // Handle qualified type references like types.environmentCodes
                let type_name = get_node_text(child, source_code);
                return Ok((BicepType::CustomType(type_name), false));
            },
            _ => {
                // Don't immediately error - check if any other children match
                // Continue with next child
            },
        }
    }

    // // If we get here without returning, we need to handle the case where the node itself might be a primitive type
    // if node.kind() == "primitive_type" {
    //     let type_name = get_node_text(node, source_code);
    //     match type_name.as_str() {
    //         "string" => return Ok((BicepType::String, false)),
    //         "int" => return Ok((BicepType::Int, false)),
    //         "bool" => return Ok((BicepType::Bool, false)),
    //         "object" => return Ok((BicepType::Object(None), false)),
    //         "array" => return Ok((BicepType::Array(Box::new(BicepType::String)), false)),
    //         _ => {}
    //     }
    // } else if node.kind() == "identifier" {
    //     // Handle the case where the node itself is an identifier
    //     let type_name = get_node_text(node, source_code);
    //     match type_name.as_str() {
    //         "object" => return Ok((BicepType::Object(None), false)),
    //         "array" => return Ok((BicepType::Array(Box::new(BicepType::String)), false)),
    //         _ => return Ok((BicepType::CustomType(type_name), false)),
    //     }
    // }

    Err(Box::new(BicepParserError::ParseError(format!(
        "Failed to parse type node: {}",
        node.kind()
    ))))
}
