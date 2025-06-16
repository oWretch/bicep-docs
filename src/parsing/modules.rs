//! Module declaration parsing for Bicep files.
//!
//! This module handles the parsing of Bicep module declarations, including:
//! - Local file path modules (e.g., './modules/storage.bicep')
//! - Registry-based modules (e.g., 'br:mcr.microsoft.com/bicep/storage:v1.0')
//! - TypeSpec modules (e.g., 'ts:mysubscription/mygroup/myspec:v1.0')
//! - Module parameters and properties
//! - Module decorators and metadata
//!
//! The module supports different module source formats:
//! - **Local Path**: Direct file references
//! - **Registry**: Azure Container Registry or other OCI registries with aliases or FQDNs
//! - **TypeSpec**: Template specifications with subscription and resource group references

use std::error::Error;

use serde::{ser::SerializeMap, Deserialize, Serialize, Serializer};
use serde_with::skip_serializing_none;
use tracing::debug;
use tree_sitter::Node;

use super::{
    utils::{
        decorators::extract_description_from_decorators, get_node_text, values::parse_value_node,
    },
    BicepDecorator, BicepParserError, BicepValue,
};

// ---------------------------------------------------------------
// Structs, Enums & Types
// ---------------------------------------------------------------

/// Represents the source of a Bicep module
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[skip_serializing_none]
pub enum ModuleSource {
    /// Local file path module source
    LocalPath(String),

    /// Registry-based module source
    Registry {
        /// Optional registry alias (for br/<alias>:<path>:<version>)
        alias: Option<String>,
        /// Optional registry FQDN (for br:<registry fqdn>/<path>:<version>)
        #[serde(rename = "registry")]
        registry_fqdn: Option<String>,
        /// Required path to the module
        path: String,
        /// Required version of the module
        version: String,
    },

    /// TypeSpec module source
    TypeSpec {
        /// Optional TypeSpec alias (for ts/<alias>:<template-spec-name>:<version>)
        alias: Option<String>,
        /// Optional subscription ID (for ts:<subscription-id>/<resource-group-name>/<template-spec-name>:<version>)
        subscription_id: Option<String>,
        /// Optional resource group name (used with subscription_id)
        resource_group_name: Option<String>,
        /// Required template spec name
        template_spec_name: String,
        /// Required version of the template spec
        version: String,
    },
}

// Custom Display implementation for ModuleSource for better debug output
impl std::fmt::Display for ModuleSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleSource::LocalPath(path) => write!(f, "{path}"),
            ModuleSource::Registry {
                alias,
                registry_fqdn,
                path,
                version,
            } => {
                if let Some(alias) = alias {
                    write!(f, "br/{alias}:{path}:{version}")
                } else if let Some(fqdn) = registry_fqdn {
                    write!(f, "br:{fqdn}{path}:{version}")
                } else {
                    write!(f, "br:{path}:{version}")
                }
            },
            ModuleSource::TypeSpec {
                alias,
                subscription_id,
                resource_group_name,
                template_spec_name,
                version,
            } => {
                if let Some(alias) = alias {
                    write!(f, "ts/{alias}:{template_spec_name}:{version}")
                } else if let Some(sub_id) = subscription_id {
                    if let Some(rg) = resource_group_name {
                        write!(f, "ts:{sub_id}/{rg}/{template_spec_name}:{version}")
                    } else {
                        write!(f, "ts:{sub_id}//{template_spec_name}:{version}")
                    }
                } else {
                    write!(f, "ts:{template_spec_name}:{version}")
                }
            },
        }
    }
}

// Custom serialization for ModuleSource to prevent YAML tags
impl Serialize for ModuleSource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ModuleSource::LocalPath(path) => {
                // For local path, serialize just the string
                path.serialize(serializer)
            },
            ModuleSource::Registry {
                alias,
                registry_fqdn,
                path,
                version,
            } => {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("type", "registry")?;

                if let Some(alias) = alias {
                    map.serialize_entry("alias", alias)?;
                }
                if let Some(fqdn) = registry_fqdn {
                    map.serialize_entry("registry", fqdn)?;
                }

                map.serialize_entry("path", path)?;
                map.serialize_entry("version", version)?;
                map.end()
            },
            ModuleSource::TypeSpec {
                alias,
                subscription_id,
                resource_group_name,
                template_spec_name,
                version,
            } => {
                let mut map = serializer.serialize_map(Some(5))?;
                map.serialize_entry("type", "typespec")?;

                if let Some(alias) = alias {
                    map.serialize_entry("alias", alias)?;
                }
                if let Some(sub_id) = subscription_id {
                    map.serialize_entry("subscription", sub_id)?;
                }
                if let Some(rg) = resource_group_name {
                    map.serialize_entry("resourceGroup", rg)?;
                }

                map.serialize_entry("name", template_spec_name)?;
                map.serialize_entry("version", version)?;
                map.end()
            },
        }
    }
}

impl ModuleSource {
    /// Parses a module source string and returns the appropriate ModuleSource enum variant
    ///
    /// # Arguments
    ///
    /// * `source` - The source string to parse
    ///
    /// # Returns
    ///
    /// A Result containing the parsed ModuleSource if successful, or an error
    ///
    /// # Examples
    ///
    /// ```
    /// use bicep_docs::parsing::ModuleSource;
    ///
    /// let local_path = ModuleSource::parse("./modules/storage.bicep").unwrap();
    /// let registry = ModuleSource::parse("br:mcr.microsoft.com/bicep/storage:v1.0").unwrap();
    /// ```
    pub fn parse(source: &str) -> Result<Self, Box<dyn Error>> {
        // Check if it's a local file path (doesn't contain :)
        if !source.contains(":") {
            return Ok(ModuleSource::LocalPath(source.to_string()));
        }

        let source_parts: Vec<&str> = source.split(':').collect();
        let has_alias = source_parts[0].contains('/');

        match source_parts[0][..2].to_lowercase().as_str() {
            "br" => {
                let source_without_prefix = &source[3..]; // Skip "br:"
                if has_alias {
                    return Self::parse_br_alias_format(source_without_prefix, source);
                } else {
                    return Self::parse_br_fqdn_format(source_without_prefix, source);
                }
            },
            "ts" => {
                let source_without_prefix = &source[3..]; // Skip "ts:"
                if has_alias {
                    return Self::parse_ts_alias_format(source_without_prefix, source);
                } else {
                    return Self::parse_ts_subscription_format(source_without_prefix, source);
                }
            },
            _ => {},
        }

        Err(Box::new(BicepParserError::ParseError(format!(
            "Unknown module source format: {source}"
        ))))
    }

    /// Parse br/<alias>:<path>:<version> format module source
    ///
    /// Parses registry module sources with aliases like 'br/myalias:storage/account:v1.0'
    ///
    /// # Arguments
    ///
    /// * `source_without_prefix` - The source string without the 'br:' prefix
    /// * `full_source` - The complete original source string for error reporting
    ///
    /// # Returns
    ///
    /// A Result containing the parsed ModuleSource::Registry variant
    fn parse_br_alias_format(
        source_without_prefix: &str,
        full_source: &str,
    ) -> Result<Self, Box<dyn Error>> {
        if let Some(colon_idx) = source_without_prefix.find(':') {
            let alias = source_without_prefix[0..colon_idx].to_string();
            let remaining = &source_without_prefix[colon_idx + 1..];

            if let Some(version_idx) = remaining.rfind(':') {
                let path = remaining[0..version_idx].to_string();
                let version = remaining[version_idx + 1..].to_string();

                return Ok(ModuleSource::Registry {
                    alias: Some(alias),
                    registry_fqdn: None,
                    path,
                    version,
                });
            }
        }

        Err(Box::new(BicepParserError::ParseError(format!(
            "Invalid registry module format with alias: {full_source}"
        ))))
    }

    /// Parse br:<registry fqdn>/<path>:<version> format module source
    ///
    /// Parses registry module sources with FQDNs like 'br:mcr.microsoft.com/bicep/storage:v1.0'
    ///
    /// # Arguments
    ///
    /// * `source_without_prefix` - The source string without the 'br:' prefix
    /// * `full_source` - The complete original source string for error reporting
    ///
    /// # Returns
    ///
    /// A Result containing the parsed ModuleSource::Registry variant
    fn parse_br_fqdn_format(
        source_without_prefix: &str,
        full_source: &str,
    ) -> Result<Self, Box<dyn Error>> {
        if let Some(slash_idx) = source_without_prefix.find('/') {
            let fqdn = source_without_prefix[0..slash_idx].to_string();
            let remaining = &source_without_prefix[slash_idx + 1..];

            if let Some(version_idx) = remaining.rfind(':') {
                let path = remaining[0..version_idx].to_string();
                let version = remaining[version_idx + 1..].to_string();

                return Ok(ModuleSource::Registry {
                    alias: None,
                    registry_fqdn: Some(fqdn),
                    path,
                    version,
                });
            }
        }

        Err(Box::new(BicepParserError::ParseError(format!(
            "Invalid registry module format with FQDN: {full_source}"
        ))))
    }

    /// Parse ts/<alias>:<template-spec-name>:<version> format module source
    ///
    /// Parses TypeSpec module sources with aliases like 'ts/myalias:myspec:v1.0'
    ///
    /// # Arguments
    ///
    /// * `source_without_prefix` - The source string without the 'ts:' prefix
    /// * `full_source` - The complete original source string for error reporting
    ///
    /// # Returns
    ///
    /// A Result containing the parsed ModuleSource::TypeSpec variant
    fn parse_ts_alias_format(
        source_without_prefix: &str,
        full_source: &str,
    ) -> Result<Self, Box<dyn Error>> {
        if let Some(colon_idx) = source_without_prefix.find(':') {
            let alias = source_without_prefix[0..colon_idx].to_string();
            let remaining = &source_without_prefix[colon_idx + 1..];

            if let Some(version_idx) = remaining.rfind(':') {
                let template_spec_name = remaining[0..version_idx].to_string();
                let version = remaining[version_idx + 1..].to_string();

                return Ok(ModuleSource::TypeSpec {
                    alias: Some(alias),
                    subscription_id: None,
                    resource_group_name: None,
                    template_spec_name,
                    version,
                });
            }
        }

        Err(Box::new(BicepParserError::ParseError(format!(
            "Invalid TypeSpec module format with alias: {full_source}"
        ))))
    }

    /// Parse ts:<subscription-id>/<resource-group-name>/<template-spec-name>:<version> format
    ///
    /// Parses TypeSpec module sources with subscription format like
    /// 'ts:mysubscription/mygroup/myspec:v1.0'
    ///
    /// # Arguments
    ///
    /// * `source_without_prefix` - The source string without the 'ts:' prefix
    /// * `full_source` - The complete original source string for error reporting
    ///
    /// # Returns
    ///
    /// A Result containing the parsed ModuleSource::TypeSpec variant
    fn parse_ts_subscription_format(
        source_without_prefix: &str,
        full_source: &str,
    ) -> Result<Self, Box<dyn Error>> {
        let parts: Vec<&str> = source_without_prefix.split('/').collect();
        if parts.len() >= 3 {
            let subscription_id = parts[0].to_string();
            let resource_group_name = parts[1].to_string();

            // The last part should contain template-spec-name:version
            if let Some(version_idx) = parts[2].rfind(':') {
                let template_spec_name = parts[2][0..version_idx].to_string();
                let version = parts[2][version_idx + 1..].to_string();

                return Ok(ModuleSource::TypeSpec {
                    alias: None,
                    subscription_id: Some(subscription_id),
                    resource_group_name: Some(resource_group_name),
                    template_spec_name,
                    version,
                });
            }
        }

        Err(Box::new(BicepParserError::ParseError(format!(
            "Invalid TypeSpec module format with subscription: {full_source}"
        ))))
    }
}

/// Represents a module in a Bicep file
///
/// Modules in Bicep allow you to organize and reuse code. They can be local files,
/// registry-based, or TypeSpec modules.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[skip_serializing_none]
pub struct BicepModule {
    /// Optional description from decorators
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Name of the module
    pub name: String,
    /// Source of the module (local path, registry, or TypeSpec)
    pub source: ModuleSource,
    /// List of resources this module depends on
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depends_on: Option<Vec<String>>,
    /// Condition for conditional deployment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    /// Loop statement for loop deployment
    #[serde(skip_serializing_if = "Option::is_none", rename = "loop")]
    pub loop_statement: Option<String>,
    /// Batch size for deployment (from @batchSize decorator)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_size: Option<i64>,
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
/// Parse a module declaration in a Bicep file
///
/// This function parses a module declaration node from a Bicep AST and extracts
/// all relevant information including name, source, dependencies, conditions,
/// and loop statements.
///
/// # Arguments
///
/// * `node` - The tree-sitter Node representing the module declaration
/// * `source_code` - The source code text containing the module declaration
/// * `decorators` - A vector of decorators applied to the module
///
/// # Returns
///
/// A Result containing the parsed BicepModule if successful, or an error
///
/// # Errors
///
/// Returns an error if:
/// - The module source format is invalid
/// - Required module elements (name, source) are missing
/// - The AST structure is unexpected
///
/// # Examples
///
/// ```rust,ignore
/// use bicep_docs::parsing::{parse_module_declaration, BicepDecorator};
/// use tree_sitter::Node;
///
/// // Parse a simple local module
/// let result = parse_module_declaration(node, source_code, vec![]);
/// ```
///
/// Note: This example is marked as ignored in doctests because it requires a valid tree-sitter node.
pub fn parse_module_declaration(
    node: Node,
    source_code: &str,
    decorators: Vec<BicepDecorator>,
) -> Result<BicepModule, Box<dyn Error>> {
    debug!(
        "Parsing module declaration with {} decorators",
        decorators.len()
    );

    let mut name = String::new();
    let mut source: ModuleSource = ModuleSource::LocalPath(String::new());
    let mut depends_on: Option<Vec<String>> = None;
    let mut condition: Option<String> = None;
    let mut loop_iterator: Option<String> = None;
    let mut loop_array: Option<String> = None;
    let mut batch_size: Option<i64> = None;

    // Extract description from decorators
    let description = extract_description_from_decorators(&decorators);

    // Extract batch size from decorators
    for decorator in &decorators {
        if decorator.name == "batchSize" || decorator.name == "sys.batchSize" {
            if let BicepValue::Int(size) = &decorator.argument {
                batch_size = Some(*size);
            }
        }
    }

    let full_source_text = get_node_text(&node, source_code)?;

    // Walk through children to extract module information
    let mut cursor = node.walk();
    let children = node.children(&mut cursor).collect::<Vec<_>>();

    for i in 0..children.len() {
        match children[i].kind() {
            "module" => {
                // Module keyword found, check for name and source
                if i + 2 < children.len() {
                    // Next should be identifier and string (source)
                    if children[i + 1].kind() == "identifier" {
                        name = get_node_text(&children[i + 1], source_code)?;
                    }

                    if children[i + 2].kind() == "string" {
                        let source_str = get_node_text(&children[i + 2], source_code)?;
                        // Strip quotes if present
                        let source_without_quotes = source_str.trim_matches('\'').trim_matches('"');

                        // Parse the source to determine the source type
                        match ModuleSource::parse(source_without_quotes) {
                            Ok(parsed_source) => {
                                source = parsed_source;
                            },
                            Err(e) => {
                                return Err(Box::new(BicepParserError::ParseError(format!(
                                    "Failed to parse module source: {e}"
                                ))));
                            },
                        }
                    }
                }
            },
            "object" => {
                // This is the module properties object - only extract dependsOn
                if let Ok(Some(BicepValue::Object(props))) =
                    parse_value_node(children[i], source_code)
                {
                    // Look for dependsOn property
                    if let Some(depends_value) = props.get("dependsOn") {
                        match depends_value {
                            BicepValue::Array(deps) => {
                                let mut dep_names = Vec::new();
                                for dep in deps {
                                    match dep {
                                        BicepValue::String(dep_name) => {
                                            dep_names.push(dep_name.to_string());
                                        },
                                        BicepValue::Identifier(identifier) => {
                                            dep_names.push(identifier.to_string());
                                        },
                                        _ => {
                                            dep_names.push(format!("{dep}"));
                                        },
                                    }
                                }
                                if !dep_names.is_empty() {
                                    depends_on = Some(dep_names);
                                }
                            },
                            BicepValue::String(dep_name) => {
                                depends_on = Some(vec![dep_name.to_string()]);
                            },
                            BicepValue::Identifier(identifier) => {
                                depends_on = Some(vec![identifier.to_string()]);
                            },
                            _ => {
                                depends_on = Some(vec![format!("{}", depends_value)]);
                            },
                        }
                    }
                }
            },
            "if_statement" => {
                // Conditional module - extract the condition and nested object
                let node_text = get_node_text(&children[i], source_code)?;

                // Extract condition from the if statement
                if let Some(if_start) = node_text.find("if ") {
                    if let Some(condition_end) = node_text[if_start + 3..].find(") {") {
                        let condition_text =
                            node_text[if_start + 3..if_start + 3 + condition_end + 1].trim(); // Include the closing parenthesis
                        if !condition_text.is_empty() {
                            condition = Some(condition_text.to_string());
                        }
                    }
                }
            },
            "for_statement" => {
                // Loop module - extract the loop details and nested object
                let node_text = get_node_text(&children[i], source_code)?;

                // Extract loop details from the for statement
                if let Some(for_start) = node_text.find("for ") {
                    if let Some(colon_idx) = node_text[for_start..].find(':') {
                        let for_expression = node_text[for_start..for_start + colon_idx].trim();

                        // Parse iterator and array from the expression
                        if let Some(in_idx) = for_expression.find(" in ") {
                            let iterator = for_expression[4..in_idx].trim(); // Skip "for "
                            let array = for_expression[in_idx + 4..].trim(); // Skip " in "

                            if !iterator.is_empty() {
                                loop_iterator = Some(iterator.to_string());
                            }

                            if !array.is_empty() {
                                loop_array = Some(array.to_string());
                            }
                        }
                    }
                }
            },
            "array" => {
                // This might be a module loop with array literal
                let node_text = get_node_text(&children[i], source_code)?;

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
                    }
                }

                // Try to extract loop details from full_source_text
                if full_source_text.contains("for") {
                    // Try to parse loop iterator and array from the text
                    if let Some(for_idx) = full_source_text.find("for") {
                        if let Some(in_idx) = full_source_text.find("in") {
                            if for_idx < in_idx {
                                // Extract iterator variable
                                let iterator_text = full_source_text[for_idx + 3..in_idx].trim();
                                if !iterator_text.is_empty() {
                                    loop_iterator = Some(iterator_text.to_string());
                                }

                                // Only try to extract array expression if we didn't already find it above
                                if loop_array.is_none() {
                                    if let Some(colon_idx) = full_source_text[in_idx..].find(':') {
                                        let array_text =
                                            full_source_text[in_idx + 2..in_idx + colon_idx].trim();
                                        if !array_text.is_empty() {
                                            loop_array = Some(array_text.to_string());
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
                                                loop_array = Some(array_text.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            _ => {},
        }
    }

    // Create the loop statement from iterator and array
    let loop_statement = if loop_iterator.is_some() || loop_array.is_some() {
        match (loop_iterator, loop_array) {
            (Some(iterator), Some(array)) => Some(format!("for {iterator} in {array}")),
            (Some(iterator), None) => Some(format!("for {iterator}")),
            (None, Some(array)) => Some(format!("for _ in {array}")),
            (None, None) => None,
        }
    } else {
        None
    };

    // Create the module
    let module = BicepModule {
        name: name.clone(),
        source,
        description,
        depends_on,
        condition,
        loop_statement,
        batch_size,
    };

    debug!("Successfully parsed module: {}", name);
    Ok(module)
}
