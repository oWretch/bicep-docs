//! Output declaration parsing for Bicep files.
//!
//! This module handles the parsing of output declarations in Bicep files,
//! including output types, values, and validation constraints from decorators.
//!
//! Outputs allow Bicep templates to return values that can be consumed by
//! the calling template or by external systems. They support:
//! - Type constraints and validation
//! - Description and metadata
//! - Length and value constraints
//! - Complex object and array outputs

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::error::Error;
use tracing::debug;
use tree_sitter::Node;

use super::utils::decorators::extract_description_from_decorators;
use super::utils::types::parse_array_type;
use super::utils::values::{parse_array_items, parse_value_node};
use super::{get_node_text, BicepDecorator, BicepParameter, BicepType, BicepValue};

// ---------------------------------------------------------------
// Structs, Enums & Types
// ---------------------------------------------------------------

/// Type alias for complex decorator properties tuple to improve code readability
type DecoratorProperties = (
    Option<String>,                       // discriminator
    Option<i64>,                          // max_length
    Option<i64>,                          // min_length
    Option<i64>,                          // max_value
    Option<i64>,                          // min_value
    Option<IndexMap<String, BicepValue>>, // metadata
    bool,                                 // sealed
    bool,                                 // secure
);

/// Represents an output in a Bicep file
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[skip_serializing_none]
pub struct BicepOutput {
    /// Optional description from decorators
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Type of the output (string, int, etc.)
    #[serde(rename = "type")]
    pub output_type: BicepType,

    /// Value of the output
    pub value: BicepValue,

    /// Discriminator property from decorators
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<String>,

    /// Minimum length constraint from @minLength decorator
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<i64>,

    /// Maximum length constraint from @maxLength decorator
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<i64>,

    /// Minimum value constraint from @minValue decorator
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_value: Option<i64>,

    /// Maximum value constraint from @maxValue decorator
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_value: Option<i64>,

    /// Metadata from @metadata decorator, without the description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<IndexMap<String, BicepValue>>,

    /// Whether the output is sealed from @sealed decorator
    pub sealed: bool,

    /// Whether the output is secure from @secure decorator
    pub secure: bool,
}

// ---------------------------------------------------------------
// Functions
// ---------------------------------------------------------------

/// Extract constraint and property values from decorators
///
/// This function processes decorators to extract various constraint values
/// and properties like minValue, maxValue, minLength, maxLength, etc.
///
/// # Arguments
///
/// * `decorators` - A vector of BicepDecorator objects to process
///
/// # Returns
///
/// A tuple containing (discriminator, max_length, min_length, max_value, min_value, metadata, sealed, secure)
fn extract_decorator_properties(decorators: &[BicepDecorator]) -> DecoratorProperties {
    let mut discriminator = None;
    let mut max_length = None;
    let mut min_length = None;
    let mut max_value = None;
    let mut min_value = None;
    let mut metadata = None;
    let mut sealed = false;
    let mut secure = false;

    for decorator in decorators {
        match decorator.name.as_str() {
            "discriminator" => {
                if let BicepValue::String(value) = &decorator.argument {
                    discriminator = Some(value.clone());
                }
            },
            "maxLength" => {
                if let BicepValue::Int(value) = &decorator.argument {
                    max_length = Some(*value);
                }
            },
            "minLength" => {
                if let BicepValue::Int(value) = &decorator.argument {
                    min_length = Some(*value);
                }
            },
            "maxValue" => {
                if let BicepValue::Int(value) = &decorator.argument {
                    max_value = Some(*value);
                }
            },
            "minValue" => {
                if let BicepValue::Int(value) = &decorator.argument {
                    min_value = Some(*value);
                }
            },
            "metadata" => {
                if let BicepValue::Object(map) = &decorator.argument {
                    metadata = Some(map.clone());
                }
            },
            "sealed" => {
                sealed = true;
            },
            "secure" => {
                secure = true;
            },
            _ => {},
        }
    }

    (
        discriminator,
        max_length,
        min_length,
        max_value,
        min_value,
        metadata,
        sealed,
        secure,
    )
}

/// Parse an output declaration in a Bicep file
///
/// This function parses an output declaration node from a Bicep AST and extracts
/// all relevant information including name, type, value, and constraints from decorators.
///
/// # Arguments
///
/// * `node` - The tree-sitter Node representing the output declaration
/// * `source_code` - The source code text containing the output declaration
/// * `decorators` - A vector of decorators applied to the output
///
/// # Returns
///
/// A Result containing a tuple of (output_name, BicepOutput) if successful, or an error
///
/// # Errors
///
/// Returns an error if:
/// - The output structure is malformed
/// - Required output elements (name, type, value) are missing
/// - The AST structure is unexpected
///
/// # Examples
///
/// ```rust,ignore
/// use bicep_docs::parsing::{parse_output_declaration, BicepDecorator};
/// use tree_sitter::Node;
///
/// // Parse a simple string output
/// let result = parse_output_declaration(node, source_code, vec![]);
/// ```
///
/// Note: This example is marked as ignored in doctests because it requires a valid tree-sitter node.
pub fn parse_output_declaration(
    node: Node,
    source_code: &str,
    decorators: Vec<BicepDecorator>,
) -> Result<(String, BicepOutput), Box<dyn Error>> {
    debug!(
        "Parsing output declaration with {} decorators",
        decorators.len()
    );

    let mut name = String::new();
    let mut output_type = BicepType::String; // Default
    let mut value = BicepValue::String(String::new()); // Default

    // Extract description from decorators
    let description = extract_description_from_decorators(&decorators);

    // Extract constraint and property values from decorators
    let (discriminator, max_length, min_length, max_value, min_value, metadata, sealed, secure) =
        extract_decorator_properties(&decorators);

    // Walk through children to extract output information
    let mut cursor = node.walk();
    let children = node.children(&mut cursor).collect::<Vec<_>>();

    // Find 'output' keyword and output name
    for i in 0..children.len() {
        if children[i].kind() == "output" && i + 1 < children.len() {
            // After 'output' keyword, the next node should be the output name
            if children[i + 1].kind() == "identifier" {
                name = get_node_text(children[i + 1], source_code);

                // After name, the next is typically the type
                if i + 2 < children.len() {
                    if children[i + 2].kind() == "type"
                        || children[i + 2].kind() == "identifier"
                        || children[i + 2].kind() == "array"
                    {
                        let type_text = get_node_text(children[i + 2], source_code);

                        // Handle different output types
                        match type_text.as_str() {
                            "string" => output_type = BicepType::String,
                            "int" => output_type = BicepType::Int,
                            "bool" => output_type = BicepType::Bool,
                            "array" => output_type = BicepType::Array(Box::new(BicepType::String)), // Default to array of strings
                            "object" => output_type = BicepType::Object(None),
                            _ => {
                                // This might be a custom type
                                output_type = BicepType::CustomType(type_text);
                            },
                        }
                    } else if children[i + 2].kind() == "array_type" {
                        // Handle array type with specific element type
                        if let Ok(element_type) = parse_array_type(children[i + 2], source_code) {
                            output_type = element_type;
                        }
                    }
                }

                break;
            }
        }
    }

    // Process the value assignment
    for i in 0..children.len() {
        let child = children[i];

        if child.kind() == "=" {
            // If we find an equals sign, next node should be the value
            if i + 1 < children.len() {
                let value_node = children[i + 1];

                if let Ok(Some(parsed_value)) = parse_value_node(value_node, source_code) {
                    value = parsed_value;
                }

                // Special handling for certain types
                if value_node.kind() == "boolean" {
                    let text = get_node_text(value_node, source_code);
                    if text == "true" {
                        value = BicepValue::Bool(true);
                    } else if text == "false" {
                        value = BicepValue::Bool(false);
                    }
                } else if value_node.kind() == "array" {
                    let array_items = parse_array_items(value_node, source_code)?;
                    value = BicepValue::Array(array_items);
                    // If we have an array, update the output type if it's not already an array
                    if !matches!(output_type, BicepType::Array(_)) {
                        output_type = BicepType::Array(Box::new(BicepType::String));
                        // Default to array of strings
                    }
                } else if value_node.kind() == "object" {
                    // For outputs with object values, only override type if no explicit type was declared
                    if matches!(output_type, BicepType::String) {
                        // Default type means no explicit type
                        output_type = BicepType::Object(None);
                    }
                    // If we already have a custom type, keep it
                } else if value_node.kind() == "conditional_expression" {
                    // Handle conditional expressions (ternary)
                    let mut conditional_cursor = value_node.walk();
                    let conditional_children = value_node
                        .children(&mut conditional_cursor)
                        .collect::<Vec<_>>();

                    // Only infer type from conditional if no explicit type was declared
                    if matches!(output_type, BicepType::String) {
                        // Default type means no explicit type
                        // Try to infer type from the conditional branches
                        if conditional_children.len() >= 5 {
                            // Find the "true" branch (usually the 3rd child after condition and ?)
                            if let Some(true_branch) = conditional_children.get(2) {
                                if let Ok(Some(true_value)) =
                                    parse_value_node(*true_branch, source_code)
                                {
                                    match true_value {
                                        BicepValue::String(_) => output_type = BicepType::String,
                                        BicepValue::Int(_) => output_type = BicepType::Int,
                                        BicepValue::Bool(_) => output_type = BicepType::Bool,
                                        BicepValue::Identifier(_) => {
                                            output_type = BicepType::String
                                        }, // Treat identifiers as strings
                                        BicepValue::Array(_) => {
                                            output_type =
                                                BicepType::Array(Box::new(BicepType::String));
                                        },
                                        BicepValue::Object(obj_props) => {
                                            // Convert BicepValue properties to BicepParameter properties
                                            let mut param_props = IndexMap::new();
                                            for (key, prop_value) in obj_props {
                                                let mut param = BicepParameter::default();

                                                // Determine parameter type based on the BicepValue
                                                match &prop_value {
                                                    BicepValue::String(_) => {
                                                        param.parameter_type = BicepType::String
                                                    },
                                                    BicepValue::Int(_) => {
                                                        param.parameter_type = BicepType::Int
                                                    },
                                                    BicepValue::Bool(_) => {
                                                        param.parameter_type = BicepType::Bool
                                                    },
                                                    BicepValue::Array(_) => {
                                                        param.parameter_type = BicepType::Array(
                                                            Box::new(BicepType::String),
                                                        )
                                                    },
                                                    BicepValue::Object(_) => {
                                                        param.parameter_type =
                                                            BicepType::Object(None)
                                                    },
                                                    BicepValue::Identifier(_) => {
                                                        param.parameter_type = BicepType::String
                                                    },
                                                }

                                                param_props.insert(key.clone(), param);
                                            }
                                            output_type = BicepType::Object(Some(param_props));
                                        },
                                    }
                                }
                            }
                        }
                    }

                    // The full expression text becomes the value
                    let expr_text = get_node_text(value_node, source_code);
                    value = BicepValue::String(expr_text);
                }
            }
        }
    }

    // Create the output
    let output = BicepOutput {
        output_type,
        value,
        description,
        discriminator,
        max_length,
        min_length,
        max_value,
        min_value,
        metadata,
        sealed,
        secure,
    };

    debug!("Successfully parsed output: {}", name);
    Ok((name, output))
}
