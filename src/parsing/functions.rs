//! Function declaration parsing for Bicep files.
//!
//! This module handles the parsing of user-defined function declarations in Bicep files.
//! Functions allow developers to create reusable logic that can be called throughout
//! the template, improving code organization and reducing duplication.

use std::error::Error;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use tracing::{debug, warn};
use tree_sitter::Node;

use super::{
    utils::{
        decorators::extract_description_from_decorators,
        get_node_text,
        types::{parse_property_type, parse_type_node},
    },
    BicepDecorator, BicepParserError, BicepType, BicepValue,
};

// ---------------------------------------------------------------
// Structs, Enums & Types
// ---------------------------------------------------------------

/// Represents a user-defined function in a Bicep file.
///
/// Functions provide a way to encapsulate reusable logic and computations
/// that can be called throughout the template. They help improve code
/// organization and reduce duplication.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[skip_serializing_none]
pub struct BicepFunction {
    /// Optional description extracted from decorators
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Metadata associated with the function
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub metadata: IndexMap<String, BicepValue>,

    /// List of function parameters/arguments
    pub arguments: Vec<BicepFunctionArgument>,

    /// The return type of the function
    pub return_type: BicepType,

    /// The function body expression
    pub expression: String,

    /// Whether this function is exported for use in other modules
    #[serde(rename = "exported")]
    pub is_exported: bool,
}

/// Represents a function argument/parameter in a Bicep function.
///
/// Function arguments define the input parameters that the function accepts,
/// including their types and whether they are optional.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[skip_serializing_none]
pub struct BicepFunctionArgument {
    /// The parameter name
    pub name: String,
    /// The parameter type
    #[serde(rename = "type")]
    pub argument_type: BicepType,
    /// Whether the parameter is optional/nullable
    #[serde(rename = "optional")]
    pub is_nullable: bool,
}

// ---------------------------------------------------------------
// Functions
// ---------------------------------------------------------------

/// Parses a function declaration in a Bicep file.
///
/// This function extracts the function name, parameters, return type, body expression,
/// and metadata from decorators. Functions can be exported for use in other modules.
///
/// # Arguments
///
/// * `node` - The tree-sitter Node representing the function declaration
/// * `source_code` - The source code text containing the function declaration
/// * `decorators` - Vector of decorators applied to the function
///
/// # Returns
///
/// A Result containing a tuple of (function_name, BicepFunction) if successful
///
/// # Errors
///
/// Returns an error if:
/// - The function declaration is missing required elements
/// - Function parameters cannot be parsed
/// - Return type parsing fails
/// - Invalid syntax is encountered
///
/// # Examples
///
/// ```rust,ignore
/// // Parsing a simple function:
/// // func getName() string => 'myFunction'
/// let (name, function) = parse_function_declaration(node, source_code, decorators)?;
/// assert_eq!(name, "getName");
/// ```
///
/// Note: This example is marked as ignored in doctests because it requires a valid tree-sitter node.
pub(crate) fn parse_function_declaration(
    node: Node,
    source_code: &str,
    decorators: Vec<BicepDecorator>,
) -> Result<(String, BicepFunction), Box<dyn Error>> {
    let mut metadata: IndexMap<String, BicepValue> = IndexMap::new();
    let mut is_exported = false;

    let mut cursor = node.walk();
    let children = node.children(&mut cursor).collect::<Vec<_>>();

    // Validate we have the expected number of children
    if children.len() < 6 {
        return Err(Box::new(BicepParserError::ParseError(
            "Function declaration missing required elements".to_string(),
        )));
    }

    let name = get_node_text(&children[1], source_code)?;
    if name.is_empty() {
        return Err(Box::new(BicepParserError::ParseError(
            "Function declaration missing name".to_string(),
        )));
    }

    debug!("Parsing function declaration: {}", name);

    let arguments = parse_function_parameters(children[2], source_code)?;
    let return_type = parse_property_type(children[3], source_code)?;
    let expression = get_node_text(&children[5], source_code)?;
    let description = extract_description_from_decorators(&decorators);

    // Process decorators for metadata and export status
    process_function_decorators(&decorators, &mut metadata, &mut is_exported, &name);

    Ok((
        name,
        BicepFunction {
            arguments,
            return_type,
            description,
            metadata,
            expression,
            is_exported,
        },
    ))
}

/// Processes decorators to extract metadata and export status.
fn process_function_decorators(
    decorators: &[BicepDecorator],
    metadata: &mut IndexMap<String, BicepValue>,
    is_exported: &mut bool,
    function_name: &str,
) {
    for decorator in decorators {
        match decorator.name.as_str() {
            "export" | "sys.export" => {
                *is_exported = true;
                debug!("Function {} marked as exported", function_name);
            },
            "metadata" | "sys.metadata" => {
                if let BicepValue::Object(map) = &decorator.argument {
                    *metadata = map.clone();
                    debug!("Function {} has metadata", function_name);
                }
            },
            "description" | "sys.description" => {
                // Handle description decorator by adding it to metadata
                if let BicepValue::String(desc) = &decorator.argument {
                    metadata.insert("description".to_string(), BicepValue::String(desc.clone()));
                    debug!("Function {} has description: {}", function_name, desc);
                }
            },
            _ => {
                warn!(
                    "Unknown decorator {} on function {}",
                    decorator.name, function_name
                );
            },
        }
    }
}

/// Parses function parameters from a parameter list node.
///
/// # Arguments
///
/// * `node` - The tree-sitter Node representing the parameter list
/// * `source_code` - The source code text containing the parameter definitions
///
/// # Returns
///
/// A Result containing a vector of BicepFunctionArgument if successful
///
/// # Errors
///
/// Returns an error if parameter parsing fails
fn parse_function_parameters(
    node: Node,
    source_code: &str,
) -> Result<Vec<BicepFunctionArgument>, Box<dyn Error>> {
    let mut arguments = Vec::new();
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        if child.kind() == "parameter" {
            arguments.push(parse_function_argument(child, source_code)?);
        }
    }

    debug!("Parsed {} function parameters", arguments.len());
    Ok(arguments)
}

/// Parses a single function parameter.
///
/// # Arguments
///
/// * `node` - The tree-sitter Node representing the parameter
/// * `source_code` - The source code text containing the parameter definition
///
/// # Returns
///
/// A Result containing a BicepFunctionArgument if successful
///
/// # Errors
///
/// Returns an error if the parameter cannot be parsed
fn parse_function_argument(
    node: Node,
    source_code: &str,
) -> Result<BicepFunctionArgument, Box<dyn Error>> {
    let mut name = String::new();
    let mut argument_type = BicepType::String;
    let mut is_nullable = false;

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "identifier" => {
                name = get_node_text(&child, source_code)?;
            },
            "type" => {
                (argument_type, is_nullable) = parse_type_node(child, source_code)?;
            },
            _ => {},
        }
    }

    if name.is_empty() {
        return Err(Box::new(BicepParserError::ParseError(
            "Function parameter missing name".to_string(),
        )));
    }

    debug!(
        "Parsed function parameter: {} (type: {:?})",
        name, argument_type
    );

    Ok(BicepFunctionArgument {
        name,
        argument_type,
        is_nullable,
    })
}
