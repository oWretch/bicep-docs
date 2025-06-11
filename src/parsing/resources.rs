//! Resource declaration parsing for Bicep files.
//!
//! This module handles the parsing of resource declarations in Bicep files,
//! including resource properties, dependencies, conditions, loops, and scope configuration.
//! Resources represent Azure services and their configuration in Infrastructure as Code.

use std::error::Error;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use tree_sitter::Node;

use super::{get_node_text, utils::values::parse_value_node, BicepDecorator, BicepValue};

// ---------------------------------------------------------------
// Structs, Enums & Types
// ---------------------------------------------------------------

/// Represents a resource declaration in a Bicep file.
///
/// Resources define Azure services and their configuration, including properties,
/// dependencies, conditions, and deployment scope. They are the core building blocks
/// of Infrastructure as Code in Bicep.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[skip_serializing_none]
pub struct BicepResource {
    /// Optional description extracted from decorators
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The Azure resource type (e.g., "Microsoft.Storage/storageAccounts")
    #[serde(rename = "type")]
    pub resource_type: String,

    /// The API version for the resource type
    pub api_version: String,

    /// Whether this references an existing resource rather than creating a new one
    pub existing: bool,

    /// The deployment scope for the resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<BicepValue>,

    /// The name of the resource instance
    pub name: String,

    /// Parent resource reference for nested resources
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,

    /// List of resources this resource depends on
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depends_on: Option<Vec<String>>,

    /// Condition that must be true for the resource to be deployed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,

    /// Loop configuration for creating multiple instances
    #[serde(rename = "loop", skip_serializing_if = "Option::is_none")]
    pub loop_statement: Option<String>,

    /// Batch size for parallel deployment in loops
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_size: Option<i64>,
}

// ---------------------------------------------------------------
// Functions
// ---------------------------------------------------------------

/// Recursively finds object nodes within a tree node.
///
/// This function is used for parsing conditional and loop resources where
/// resource properties may be nested within conditional or loop structures.
///
/// # Arguments
///
/// * `node` - The tree-sitter Node to search
/// * `source_code` - The source code text containing the node
///
/// # Returns
///
/// A vector of IndexMaps representing object properties found in the tree
fn find_object_nodes_recursive(node: Node, source_code: &str) -> Vec<IndexMap<String, BicepValue>> {
    let mut objects = Vec::with_capacity(4); // Pre-allocate for performance
    let mut cursor = node.walk();

    // Check if this node is an object
    if node.kind() == "object" {
        if let Ok(Some(BicepValue::Object(props))) = parse_value_node(node, source_code) {
            objects.push(props);
        }
    }

    // Recursively search children
    for child in node.children(&mut cursor) {
        objects.extend(find_object_nodes_recursive(child, source_code));
    }

    objects
}

/// Parses a resource declaration in a Bicep file.
///
/// This function processes resource declarations, extracting the resource identifier,
/// type information, properties, and any metadata from decorators. It handles both
/// simple and complex resource configurations including loops, conditions, and dependencies.
///
/// # Arguments
///
/// * `node` - The tree-sitter Node representing the resource declaration
/// * `source_code` - The source code text containing the resource declaration
/// * `decorators` - Vector of decorators applied to the resource
///
/// # Returns
///
/// A Result containing a vector of tuples (resource_name, BicepResource). Multiple
/// resources may be returned when parsing loop constructs that create multiple instances.
///
/// # Errors
///
/// Returns an error if:
/// - Required resource elements (type, name) are missing
/// - Invalid syntax is encountered
/// - Resource properties cannot be parsed
///
/// # Examples
///
/// ```rust,ignore
/// // Parsing a simple storage account resource:
/// // resource storageAccount 'Microsoft.Storage/storageAccounts@2021-04-01' = { ... }
/// let resources = parse_resource_declaration(node, source_code, decorators)?;
/// assert_eq!(resources.len(), 1);
/// assert_eq!(resources[0].0, "storageAccount");
/// ```
///
/// Note: This example is marked as ignored in doctests because it requires a valid tree-sitter node.
pub fn parse_resource_declaration(
    node: Node,
    source_code: &str,
    decorators: Vec<BicepDecorator>,
) -> Result<Vec<(String, BicepResource)>, Box<dyn Error>> {
    let mut identifier = String::new();
    let mut resource_type = String::new();
    let mut api_version: Option<String> = None;
    let mut existing = false;
    let mut name = String::new();
    let mut parent: Option<String> = None;
    let mut scope: Option<BicepValue> = None;
    let mut depends_on: Option<Vec<String>> = None;
    let mut conditions: Option<String> = None;
    let mut loop_iterator: Option<String> = None;
    let mut loop_array: Option<String> = None;
    let mut batch_size: Option<i64> = None;
    let mut properties: IndexMap<String, BicepValue> = IndexMap::new();

    // Extract description from decorators
    let mut description = None;

    // Process decorators to extract description and batch size
    for decorator in &decorators {
        match &decorator.name[..] {
            // Handle description decorators
            "description" | "sys.description" => {
                if let BicepValue::String(desc_text) = &decorator.argument {
                    description = Some(desc_text.clone());
                }
            },
            // Handle metadata decorators with description
            "metadata" | "sys.metadata" => {
                if let BicepValue::Object(map) = &decorator.argument {
                    if let Some(BicepValue::String(desc_text)) = map.get("description") {
                        description = Some(desc_text.clone());
                    }
                }
            },
            // Handle batchSize decorator
            "batchSize" | "sys.batchSize" => {
                if let BicepValue::Int(size) = &decorator.argument {
                    batch_size = Some(*size);
                }
            },
            _ => {},
        }
    }

    let full_source_text = get_node_text(node, source_code);

    // Walk through children to extract resource information
    let mut cursor = node.walk();
    let children = node.children(&mut cursor).collect::<Vec<_>>();

    // Parse all child nodes to find information about this resource
    for i in 0..children.len() {
        match children[i].kind() {
            "resource" => {
                // Resource keyword found, check for name and type
                if i + 2 < children.len() {
                    // Next should be identifier and string (type)
                    if children[i + 1].kind() == "identifier" {
                        identifier = get_node_text(children[i + 1], source_code);
                    }

                    if children[i + 2].kind() == "string" {
                        let resource_type_with_api = get_node_text(children[i + 2], source_code);
                        // Strip quotes
                        let resource_type_str = resource_type_with_api.trim_matches('\'');

                        // Split resource type and API version
                        if resource_type_str.contains('@') {
                            let parts: Vec<&str> = resource_type_str.split('@').collect();
                            resource_type = parts[0].to_string();
                            if parts.len() > 1 {
                                api_version = Some(parts[1].to_string());
                            }
                        } else {
                            resource_type = resource_type_str.to_string();
                        }
                    }
                }
            },
            "existing" => {
                existing = true;
            },
            "object" => {
                // This is the resource properties object
                if let Ok(Some(BicepValue::Object(props))) =
                    parse_value_node(children[i], source_code)
                {
                    // Copy all properties to preserve identifiers and other values
                    for (key, value) in props.iter() {
                        properties.insert(key.clone(), value.clone());
                    }

                    // Extract the name property from the object
                    if let Some(name_value) = props.get("name") {
                        match name_value {
                            BicepValue::String(resource_name) => {
                                name = resource_name.clone();
                            },
                            BicepValue::Identifier(identifier) => {
                                // Use the identifier name but mark it as a reference for YAML output
                                name = format!("${{{}}}", identifier);
                            },
                            _ => {
                                // For other types, convert to string
                                name = format!("{}", name_value);
                            },
                        }
                    }

                    // Check for parent property
                    if let Some(parent_value) = props.get("parent") {
                        match parent_value {
                            BicepValue::String(parent_name) => {
                                parent = Some(parent_name.clone());
                            },
                            BicepValue::Identifier(identifier) => {
                                // Use the identifier as the parent
                                parent = Some(identifier.clone());
                            },
                            _ => {
                                // For other types, convert to string
                                parent = Some(format!("{}", parent_value));
                            },
                        }
                    }

                    // Check for scope property
                    if let Some(scope_value) = props.get("scope") {
                        match scope_value {
                            BicepValue::Identifier(identifier) => {
                                scope = Some(BicepValue::Identifier(identifier.clone()));
                            },
                            _ => {
                                // Even if not an identifier, store the scope value
                                scope = Some(scope_value.clone());
                            },
                        }
                    }

                    // Check for dependsOn property
                    if let Some(dep_val) = props.get("dependsOn") {
                        match dep_val {
                            // Handle array of dependencies
                            BicepValue::Array(deps) => {
                                // First, try to handle it as a vector of BicepValue
                                let mut dep_names = Vec::new();
                                for dep_val in deps.iter() {
                                    match dep_val {
                                        BicepValue::String(dep_name) => {
                                            dep_names.push(dep_name.clone());
                                        },
                                        BicepValue::Identifier(identifier) => {
                                            // Keep the reference nature but extract the name for the depends_on field
                                            dep_names.push(identifier.clone());
                                        },
                                        _ => {
                                            // For other types, try to convert to string
                                            dep_names.push(format!("{}", dep_val));
                                        },
                                    }
                                }
                                if !dep_names.is_empty() {
                                    depends_on = Some(dep_names);
                                }
                            },
                            // Handle a single dependency
                            BicepValue::String(dep_name) => {
                                depends_on = Some(vec![dep_name.clone()]);
                            },
                            BicepValue::Identifier(identifier) => {
                                depends_on = Some(vec![identifier.clone()]);
                            },
                            _ => {
                                // For other types, try to convert to string
                                depends_on = Some(vec![format!("{}", dep_val)]);
                            },
                        }
                    }
                }
            },
            "array" => {
                // This might be a resource loop

                // Look for direct loop array specification in the node text
                let node_text = get_node_text(children[i], source_code);

                // For arrays with string literals like ['alice', 'bob', 'charlie']
                if node_text.contains("[") && node_text.contains("]") {
                    let mut items = Vec::new();
                    let mut start_content = false;
                    let mut in_quote = false;
                    let mut current_item = String::new();

                    for c in node_text.chars() {
                        if c == '[' && !start_content {
                            start_content = true;
                            continue;
                        }

                        if start_content {
                            if c == ']' && !in_quote {
                                if !current_item.trim().is_empty() {
                                    items.push(current_item.trim().to_string());
                                }
                                break;
                            } else if c == '\'' || c == '"' {
                                in_quote = !in_quote;
                                current_item.push(c);
                            } else if c == ',' && !in_quote {
                                if !current_item.trim().is_empty() {
                                    items.push(current_item.trim().to_string());
                                }
                                current_item = String::new();
                            } else {
                                current_item.push(c);
                            }
                        }
                    }

                    if !items.is_empty() {
                        loop_array = Some(format!("[{}]", items.join(", ")));
                        // Store the loop array in properties as well
                        properties.insert(
                            "loopArray".to_string(),
                            BicepValue::String(format!("[{}]", items.join(", "))),
                        );
                    }
                }

                // Try to extract loop details from full_source_text
                if full_source_text.contains("for") {
                    // Store the full for loop expression in properties
                    if let Some(for_start) = full_source_text.find("for ") {
                        if let Some(bracket_end) = full_source_text[for_start..].find(':') {
                            let for_expression =
                                full_source_text[for_start..for_start + bracket_end].trim();
                            properties.insert(
                                "forExpression".to_string(),
                                BicepValue::String(for_expression.to_string()),
                            );
                        }
                    }

                    // Try to parse loop iterator and array from the text
                    if let Some(for_idx) = full_source_text.find("for") {
                        if let Some(in_idx) = full_source_text.find("in") {
                            if for_idx < in_idx {
                                // Extract iterator variable
                                let iterator_text = full_source_text[for_idx + 3..in_idx].trim();
                                if !iterator_text.is_empty() {
                                    // Check if it might be an identifier reference
                                    if iterator_text.contains(":") || iterator_text.contains(".") {
                                        // This is likely a complex expression or object property access
                                        loop_iterator = Some(iterator_text.to_string());
                                    } else {
                                        loop_iterator = Some(iterator_text.to_string());
                                    }
                                    // Store the loop iterator in properties as well
                                    properties.insert(
                                        "loopIterator".to_string(),
                                        BicepValue::String(iterator_text.to_string()),
                                    );
                                }

                                // Only try to extract array expression if we didn't already find it above
                                if loop_array.is_none() {
                                    if let Some(colon_idx) = full_source_text[in_idx..].find(':') {
                                        let array_text =
                                            full_source_text[in_idx + 2..in_idx + colon_idx].trim();
                                        if !array_text.is_empty() {
                                            // Check if it might be an identifier reference
                                            // Store as is, whether it's an array literal or a variable reference
                                            loop_array = Some(array_text.to_string());
                                            properties.insert(
                                                "loopArray".to_string(),
                                                BicepValue::String(array_text.to_string()),
                                            );
                                        }
                                    } else {
                                        // If we can't find a colon, try to extract until the next '{'
                                        if let Some(brace_idx) =
                                            full_source_text[in_idx..].find('{')
                                        {
                                            let array_text = full_source_text
                                                [in_idx + 2..in_idx + brace_idx]
                                                .trim();
                                            if !array_text.is_empty() {
                                                // Store the array expression as is, whether it's an array literal or variable reference
                                                loop_array = Some(array_text.to_string());
                                                properties.insert(
                                                    "loopArray".to_string(),
                                                    BicepValue::String(array_text.to_string()),
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "parent" => {
                // Parent property found
                if i + 1 < children.len() {
                    let parent_node = children[i + 1];
                    let parent_text = get_node_text(parent_node, source_code);

                    // Handle the parent::child syntax
                    if parent_text.contains("::") {
                        // Split parent and child paths
                        let parts: Vec<&str> = parent_text.split("::").collect();
                        if !parts.is_empty() {
                            parent = Some(parts[0].trim().to_string());
                        }
                    } else {
                        parent = Some(parent_text.trim().to_string());
                    }
                }
            },
            "if_statement" => {
                // Conditional resource - extract the condition and nested object
                let node_text = get_node_text(children[i], source_code);

                // Extract condition from the if statement
                if let Some(if_start) = node_text.find("if ") {
                    if let Some(condition_end) = node_text[if_start + 3..].find(") {") {
                        let condition_text =
                            node_text[if_start + 3..if_start + 3 + condition_end + 1].trim(); // Include the closing parenthesis
                        if !condition_text.is_empty() {
                            conditions = Some(condition_text.to_string());
                            properties.insert(
                                "condition".to_string(),
                                BicepValue::String(condition_text.to_string()),
                            );
                        }
                    }
                }
            },
            "for_statement" => {
                // Loop resource - extract the loop details and nested object
                let node_text = get_node_text(children[i], source_code);

                // Extract loop details from the for statement
                if let Some(for_start) = node_text.find("for ") {
                    if let Some(colon_idx) = node_text[for_start..].find(':') {
                        let for_expression = node_text[for_start..for_start + colon_idx].trim();
                        properties.insert(
                            "forExpression".to_string(),
                            BicepValue::String(for_expression.to_string()),
                        );

                        // Parse iterator and array from the expression
                        if let Some(in_idx) = for_expression.find(" in ") {
                            let iterator = for_expression[4..in_idx].trim(); // Skip "for "
                            let array = for_expression[in_idx + 4..].trim(); // Skip " in "

                            if !iterator.is_empty() {
                                loop_iterator = Some(iterator.to_string());
                                properties.insert(
                                    "loopIterator".to_string(),
                                    BicepValue::String(iterator.to_string()),
                                );
                            }

                            if !array.is_empty() {
                                loop_array = Some(array.to_string());
                                properties.insert(
                                    "loopArray".to_string(),
                                    BicepValue::String(array.to_string()),
                                );
                            }
                        }
                    }
                }
            },
            _ => {},
        }
    }

    // For child resources with an explicit parent property in the syntax
    if full_source_text.contains("parent:") {
        if let Some(parent_idx) = full_source_text.find("parent:") {
            let after_parent = &full_source_text[parent_idx + 7..]; // 7 is length of "parent:"

            // Find the next non-whitespace character after parent:
            let mut start_idx = 0;
            for (i, c) in after_parent.char_indices() {
                if !c.is_whitespace() {
                    start_idx = i;
                    break;
                }
            }

            // Find the end of the parent reference (could be a newline, comma, etc.)
            let mut end_idx = after_parent.len();
            for (i, c) in after_parent[start_idx..].char_indices() {
                if c == '\n' || c == ',' || c == '}' {
                    end_idx = start_idx + i;
                    break;
                }
            }

            if start_idx < end_idx {
                let parent_text = after_parent[start_idx..end_idx].trim();
                parent = Some(parent_text.to_string());
            }
        }
    }

    // If this is a loop resource, try harder to extract the array elements
    if loop_iterator.is_some() && loop_array.is_none() {
        // Try to find array literal directly in the source text
        if let Some(array_start) = full_source_text.find('[') {
            if let Some(array_end) = full_source_text[array_start..].find(']') {
                let array_content = &full_source_text[array_start + 1..array_start + array_end];

                // Handle quoted strings in array
                let mut items = Vec::new();
                let mut in_quote = false;
                let mut current_item = String::new();
                for c in array_content.chars() {
                    if c == '\'' || c == '"' {
                        in_quote = !in_quote;
                        current_item.push(c);
                    } else if c == ',' && !in_quote {
                        if !current_item.trim().is_empty() {
                            items.push(current_item.trim().to_string());
                        }
                        current_item = String::new();
                    } else {
                        current_item.push(c);
                    }
                }

                // Add the last item
                if !current_item.trim().is_empty() {
                    items.push(current_item.trim().to_string());
                }

                if !items.is_empty() {
                    loop_array = Some(format!("[{}]", items.join(", ")));
                    properties.insert(
                        "loopArray".to_string(),
                        BicepValue::String(format!("[{}]", items.join(", "))),
                    );
                }
            }
        }
    }

    // Try to extract all properties from the object node or nested objects
    // First try direct object children
    for child in &children {
        if child.kind() == "object" {
            if let Ok(Some(BicepValue::Object(props))) = parse_value_node(*child, source_code) {
                // Copy all properties to preserve identifiers and other values
                for (key, value) in props.iter() {
                    // Keep the original BicepValue to preserve identifiers
                    properties.insert(key.clone(), value.clone());

                    // Only extract name if it hasn't been set yet (to avoid overwriting)
                    if key == "name" && name.is_empty() {
                        match value {
                            BicepValue::String(s) => {
                                name = s.clone();
                            },
                            BicepValue::Identifier(id) => {
                                // Use the identifier name but mark it as a reference
                                name = format!("${{{}}}", id);
                            },
                            _ => {
                                name = format!("{}", value);
                            },
                        }
                    } else if key == "scope" && scope.is_none() {
                        // Extract scope property from nested objects too
                        scope = Some(value.clone());
                    } else if key == "scope" && scope.is_none() {
                        // Extract scope property
                        scope = Some(value.clone());
                    }
                }
            }
        }
    }

    // If no direct object found, try to find nested objects (for conditional/loop resources)
    if name.is_empty() {
        let nested_objects = find_object_nodes_recursive(node, source_code);
        for props in nested_objects {
            // Copy all properties to preserve identifiers and other values
            for (key, value) in props.iter() {
                // Keep the original BicepValue to preserve identifiers
                properties.insert(key.clone(), value.clone());

                // Only extract name if it hasn't been set yet (to avoid overwriting)
                if key == "name" && name.is_empty() {
                    match value {
                        BicepValue::String(s) => {
                            name = s.clone();
                        },
                        BicepValue::Identifier(id) => {
                            // Use the identifier name but mark it as a reference
                            name = format!("${{{}}}", id);
                        },
                        _ => {
                            name = format!("{}", value);
                        },
                    }
                }
            }
            // Break after first object to avoid overwriting
            if !name.is_empty() {
                break;
            }
        }
    }

    // Create the loop statement from iterator and array
    let loop_statement = if loop_iterator.is_some() || loop_array.is_some() {
        match (loop_iterator, loop_array) {
            (Some(iterator), Some(array)) => Some(format!("for {} in {}", iterator, array)),
            (Some(iterator), None) => Some(format!("for {}", iterator)),
            (None, Some(array)) => Some(format!("for _ in {}", array)),
            (None, None) => None,
        }
    } else {
        None
    };

    // Create the main resource
    let main_resource = BicepResource {
        name: name.clone(),
        resource_type,
        api_version: api_version.unwrap_or_default(),
        existing,
        description,
        scope,
        parent,
        depends_on,
        condition: conditions,
        loop_statement,
        batch_size,
    };

    // Collect child resources from the node
    let mut all_resources = Vec::new();

    // Add the main resource first - use identifier as key, and store name in the resource
    all_resources.push((identifier.clone(), main_resource.clone()));

    // Find nested child resources
    fn collect_child_resources(
        node: Node,
        source_code: &str,
        parent_identifier: &str,
        parent_resource: &BicepResource,
        collected: &mut Vec<(String, BicepResource)>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "resource_declaration" {
                // Parse the child resource
                if let Ok(child_resources) =
                    parse_resource_declaration(child, source_code, Vec::new())
                {
                    for (child_identifier, mut child_resource) in child_resources {
                        // For child resources declared within their parent, DON'T set the parent property
                        // The parent relationship is implicit from the nested declaration
                        // Only set parent if it was explicitly provided as a direct property in the resource

                        // Prefix the resource type with the parent's resource type
                        child_resource.resource_type = format!(
                            "{}/{}",
                            parent_resource.resource_type, child_resource.resource_type
                        );

                        // Inherit API version from parent if not specified
                        if child_resource.api_version.is_empty() {
                            child_resource.api_version = parent_resource.api_version.clone();
                        }

                        // Create prefixed identifier
                        let prefixed_identifier =
                            format!("{}::{}", parent_identifier, child_identifier);
                        collected.push((prefixed_identifier, child_resource));
                    }
                }
            } else {
                // Recursively search in non-resource nodes
                collect_child_resources(
                    child,
                    source_code,
                    parent_identifier,
                    parent_resource,
                    collected,
                );
            }
        }
    }

    // Collect child resources with the parent identifier as prefix
    collect_child_resources(
        node,
        source_code,
        &identifier,
        &main_resource,
        &mut all_resources,
    );

    Ok(all_resources)
}
