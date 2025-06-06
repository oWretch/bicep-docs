//! Variable declaration parsing for Bicep files.
//!
//! This module handles the parsing of variable declarations in Bicep files.
//! Variables are used to store computed values and constants for reuse throughout
//! the template, improving maintainability and reducing duplication.

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::error::Error;
use tracing::{debug, warn};
use tree_sitter::Node;

use super::utils::decorators::extract_description_from_decorators;
use super::utils::values::parse_value_node;
use super::{get_node_text, BicepDecorator, BicepParserError, BicepValue};

// ---------------------------------------------------------------
// Structs, Enums & Types
// ---------------------------------------------------------------

/// Represents a variable declaration in a Bicep file.
///
/// Variables store computed values and constants that can be reused throughout
/// the template. They help reduce duplication and improve maintainability by
/// centralizing common values and expressions.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[skip_serializing_none]
pub struct BicepVariable {
    /// Optional description extracted from decorators
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The computed or constant value assigned to the variable
    pub value: BicepValue,
    /// Whether this variable is exported for use in other modules
    #[serde(rename = "exported")]
    pub is_exported: bool,
}

// ---------------------------------------------------------------
// Functions
// ---------------------------------------------------------------

/// Parses a variable declaration in a Bicep file.
///
/// This function extracts the variable name, value, and metadata from decorators.
/// Variables can contain complex expressions, object literals, arrays, and function calls.
///
/// # Arguments
///
/// * `node` - The tree-sitter Node representing the variable declaration
/// * `source_code` - The source code text containing the variable declaration
/// * `decorators` - Vector of decorators applied to the variable
///
/// # Returns
///
/// A Result containing a tuple of (variable_name, BicepVariable) if successful
///
/// # Errors
///
/// Returns an error if:
/// - The variable declaration is missing required elements
/// - The variable value cannot be parsed
/// - Invalid syntax is encountered
///
/// # Examples
///
/// ```rust,ignore
/// // Parsing a simple variable:
/// // var storageAccountName = 'mystorageaccount'
/// let (name, variable) = parse_variable_declaration(node, source_code, decorators)?;
/// assert_eq!(name, "storageAccountName");
/// ```
///
/// Note: This example is marked as ignored in doctests because it requires a valid tree-sitter node.
pub fn parse_variable_declaration(
    node: Node,
    source_code: &str,
    decorators: Vec<BicepDecorator>,
) -> Result<(String, BicepVariable), Box<dyn Error>> {
    let mut is_exported = false;

    // Extract description from centralized function
    let description = extract_description_from_decorators(&decorators);

    // Process export status from decorators
    process_variable_export_decorators(&decorators, &mut is_exported);

    // Extract variable name and value from child nodes
    let mut cursor = node.walk();
    let children = node.children(&mut cursor).collect::<Vec<_>>();

    // Validate we have the expected number of children for a variable declaration
    if children.len() < 4 {
        return Err(Box::new(BicepParserError::ParseError(
            "Variable declaration missing required elements".to_string(),
        )));
    }

    // Extract variable name (should be at index 1: var NAME = value)
    let name = get_node_text(children[1], source_code);
    if name.is_empty() {
        return Err(Box::new(BicepParserError::ParseError(
            "Variable declaration missing name".to_string(),
        )));
    }

    debug!("Parsing variable declaration: {}", name);

    // Parse the variable value (should be at index 3: var name = VALUE)
    let value = match parse_value_node(children[3], source_code) {
        Ok(Some(parsed_value)) => parsed_value,
        Ok(None) => {
            warn!("Variable {} has no value, using empty string", name);
            BicepValue::String(String::new())
        },
        Err(e) => {
            warn!("Failed to parse value for variable {}: {}", name, e);
            BicepValue::String(String::new())
        },
    };

    Ok((
        name,
        BicepVariable {
            value,
            description,
            is_exported,
        },
    ))
}

/// Processes decorators to extract export status.
///
/// # Arguments
///
/// * `decorators` - The decorators to process
/// * `is_exported` - Mutable reference to set export status
fn process_variable_export_decorators(decorators: &[BicepDecorator], is_exported: &mut bool) {
    for decorator in decorators {
        match decorator.name.as_str() {
            "export" | "sys.export" => {
                *is_exported = true;
                debug!("Variable marked as exported");
            },
            _ => {
                debug!("Processing decorator: {}", decorator.name);
            },
        }
    }
}
