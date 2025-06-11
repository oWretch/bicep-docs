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

use std::{error::Error, fmt};

use indexmap::IndexMap;
use serde::{ser::SerializeMap, Deserialize, Serialize, Serializer};
use serde_with::skip_serializing_none;
use tracing::warn;
use tree_sitter::{Node, Tree};

mod functions;
mod imports;
mod modules;
mod outputs;
mod parameters;
mod resources;
mod types;
pub mod utils;
mod variables;

pub use functions::{BicepFunction, BicepFunctionArgument};
pub use imports::{parse_module_import, parse_namespace_import, BicepImport, BicepImportSymbol};
pub use modules::{parse_module_declaration, BicepModule, ModuleSource};
pub use outputs::{parse_output_declaration, BicepOutput};
pub use parameters::BicepParameter;
pub use resources::BicepResource;
pub use types::BicepCustomType;
pub use utils::decorators::extract_description_from_decorators;
pub use variables::BicepVariable;

// Import commonly used utilities from utils module using direct paths

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
                // Corrected: use 'id' instead of 'reference'
                id.serialize(serializer)
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
        use std::fmt;

        use serde::de::{self, MapAccess, SeqAccess, Visitor};

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
                if map.is_empty() {
                    write!(f, "{{}}")
                } else {
                    write!(f, "{{ ")?;
                    for (i, (key, value)) in map.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}: {}", key, value)?;
                    }
                    write!(f, " }}")
                }
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

/// Parses decorator nodes associated with a main AST node.
///
/// # Arguments
///
/// * `decorator_nodes_opt` - An `Option` containing a reference to a `Vec<Node>` of decorator nodes.
/// * `source_code` - The source code string slice.
///
/// # Returns
///
/// A `Vec<BicepDecorator>` containing all parsed decorators.
fn parse_decorators_from_node_list(
    decorator_nodes_opt: Option<&Vec<Node>>,
    source_code: &str,
) -> Vec<BicepDecorator> {
    let mut all_decorators = Vec::new();
    if let Some(dec_nodes) = decorator_nodes_opt {
        // Pre-allocate based on the number of decorator group nodes.
        // Each group node might contain multiple actual decorators.
        // A small multiplier like 2 is a heuristic, adjust if needed.
        all_decorators.reserve(dec_nodes.len().saturating_mul(2));
        for dec_node in dec_nodes {
            match utils::decorators::parse_decorators(*dec_node, source_code) {
                Ok(parsed_decorators) => {
                    all_decorators.extend(parsed_decorators);
                },
                Err(e) => {
                    // The original code logged warnings for individual decorator parsing failures.
                    // We maintain that behavior here.
                    warn!("Failed to parse decorators from a decorator node: {}", e);
                },
            }
        }
    }
    all_decorators
}

// parse_parameter_declaration function is now defined in parameters.rs

pub fn parse_bicep_document(
    tree: &Tree,
    source_code: &str,
) -> Result<BicepDocument, Box<dyn Error>> {
    let mut document = BicepDocument::default();
    let root_node = tree.root_node();
    // Pre-allocate collections with estimated capacities for better performance
    let mut metadata: IndexMap<String, BicepValue> = IndexMap::with_capacity(8);
    let mut types: IndexMap<String, BicepCustomType> = IndexMap::with_capacity(16);
    let mut parameters: IndexMap<String, BicepParameter> = IndexMap::with_capacity(32);
    let mut variables: IndexMap<String, BicepVariable> = IndexMap::with_capacity(16);
    let mut functions: IndexMap<String, BicepFunction> = IndexMap::with_capacity(8);
    let mut resources: IndexMap<String, BicepResource> = IndexMap::with_capacity(32);
    let mut modules: IndexMap<String, BicepModule> = IndexMap::with_capacity(16);
    let mut imports: Vec<BicepImport> = Vec::with_capacity(8);
    let mut outputs: IndexMap<String, BicepOutput> = IndexMap::with_capacity(16);

    // Walk through all children of the root node
    let mut cursor = root_node.walk();
    let all_nodes = root_node.children(&mut cursor).collect::<Vec<_>>();

    // First pass - collect all decorators and associate them with their declarations
    let mut decorators_map: IndexMap<usize, Vec<Node>> =
        IndexMap::with_capacity(all_nodes.len() / 4);

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
                let decorators_nodes_opt = decorators_map.get(&i);
                let all_decorators =
                    parse_decorators_from_node_list(decorators_nodes_opt, source_code);

                // Parse custom type declaration
                match types::parse_type_declaration(*node, source_code) {
                    Ok((type_name, mut custom_type)) => {
                        // Apply parsed decorators
                        // Extract description if present and not already set
                        if custom_type.description.is_none() {
                            custom_type.description =
                                extract_description_from_decorators(&all_decorators);
                        }

                        // Check for secure decorator
                        custom_type.is_secure = all_decorators.iter().any(|d| d.name == "secure");

                        // Check for export decorator
                        custom_type.is_exported = all_decorators.iter().any(|d| d.name == "export");

                        // Add all decorators to the custom type if it has a field for them
                        // Assuming BicepCustomType might have a field like `decorators: Vec<BicepDecorator>`
                        // If not, this part can be adjusted or removed.
                        // custom_type.decorators = all_decorators;

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
                let decorators_nodes_opt = decorators_map.get(&i);
                let all_decorators =
                    parse_decorators_from_node_list(decorators_nodes_opt, source_code);

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
                let decorators_nodes_opt = decorators_map.get(&i);
                let all_decorators =
                    parse_decorators_from_node_list(decorators_nodes_opt, source_code);

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
                let decorators_nodes_opt = decorators_map.get(&i);
                let all_decorators =
                    parse_decorators_from_node_list(decorators_nodes_opt, source_code);

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
                let decorators_nodes_opt = decorators_map.get(&i);
                let all_decorators =
                    parse_decorators_from_node_list(decorators_nodes_opt, source_code);

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
                let decorators_nodes_opt = decorators_map.get(&i);
                let all_decorators =
                    parse_decorators_from_node_list(decorators_nodes_opt, source_code);

                // Parse module declaration
                match parse_module_declaration(*node, source_code, all_decorators) {
                    Ok(module) => {
                        let name = module.name.clone();
                        modules.insert(name, module);
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
                let decorators_nodes_opt = decorators_map.get(&i);
                let all_decorators =
                    parse_decorators_from_node_list(decorators_nodes_opt, source_code);

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
        document.name = Some(name.to_string());
    }

    if let Some(BicepValue::String(desc)) = metadata.get("description") {
        document.description = Some(desc.to_string());
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
    let mut value: Option<BicepValue> = None;

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "identifier" => {
                name = utils::get_node_text(&child, source_code).unwrap_or_else(|_| "".to_string())
            },
            "string" | "number" | "boolean" => {
                let result = get_primitive_value(child, source_code);
                match result {
                    Ok(val) => value = Some(val),
                    Err(e) => warn!("Could not parse value of metadata {}: {}", name, e),
                }
            },
            "object" => {
                // Handle object values in metadata
                match utils::values::parse_object_properties_for_value(child, source_code) {
                    Ok(properties) => value = Some(BicepValue::Object(properties)),
                    Err(e) => warn!("Could not parse object value for metadata {}: {}", name, e),
                }
            },
            "array" => {
                // Handle array values in metadata
                match utils::values::parse_array_items(child, source_code) {
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
                match utils::values::parse_value_node(child, source_code) {
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

/// Extract a primitive Bicep value from a node
fn get_primitive_value(node: Node, source_code: &str) -> Result<BicepValue, Box<dyn Error>> {
    match node.kind() {
        "string" => {
            // For string nodes, look for string_content child nodes instead of using the entire text
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "string_content" {
                    let content = utils::get_node_text(&child, source_code)?;
                    return Ok(BicepValue::String(content));
                }
            }

            // Fallback to the old method if no string_content is found
            let node_text = utils::get_node_text(&node, source_code)?;
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
        "number" => {
            let node_text = utils::get_node_text(&node, source_code)?;
            match node_text.parse::<i64>() {
                Ok(n) => Ok(BicepValue::Int(n)),
                Err(_) => Err(Box::new(BicepParserError::InvalidValue {
                    kind: "number".to_string(),
                    reason: format!("Could not parse '{}' as integer", node_text),
                })),
            }
        },
        "boolean" => {
            let node_text = utils::get_node_text(&node, source_code)?;
            match node_text.parse::<bool>() {
                Ok(b) => Ok(BicepValue::Bool(b)),
                Err(_) => Err(Box::new(BicepParserError::InvalidValue {
                    kind: "boolean".to_string(),
                    reason: format!("Could not parse '{}' as boolean", node_text),
                })),
            }
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
            scope_text =
                utils::get_node_text(&child, source_code).unwrap_or_else(|_| "".to_string());
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
