/// AsciiDoc export functionality for Bicep documents
///
/// This module provides functions to export parsed Bicep documents
/// to AsciiDoc format with structured documentation layout.
use std::error::Error as StdError;
use std::fs;
use std::path::Path;

use crate::parsing::{
    BicepDocument, BicepFunctionArgument, BicepImport, BicepType, BicepValue, ModuleSource,
};

/// Format options for AsciiDoc output
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsciiDocFormat {
    /// Use tables for displaying properties (default)
    Table,
    /// Use lists for displaying properties
    List,
}

impl Default for AsciiDocFormat {
    fn default() -> Self {
        AsciiDocFormat::Table
    }
}

impl std::str::FromStr for AsciiDocFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "table" => Ok(AsciiDocFormat::Table),
            "list" => Ok(AsciiDocFormat::List),
            _ => Err(format!("Invalid format '{}'. Use 'table' or 'list'", s)),
        }
    }
}

/// Export a Bicep document to an AsciiDoc file
///
/// # Arguments
///
/// * `document` - The BicepDocument to export
/// * `file_path` - Path where the AsciiDoc file should be written
///
/// # Returns
///
/// Result indicating success or failure of the export operation
///
/// # Errors
///
/// Returns an error if file writing fails
pub fn export_to_file<P: AsRef<Path>>(
    document: &BicepDocument,
    file_path: P,
) -> Result<(), Box<dyn StdError>> {
    export_to_file_with_format(document, file_path, AsciiDocFormat::default())
}

/// Export a Bicep document to an AsciiDoc file with specific format
///
/// # Arguments
///
/// * `document` - The BicepDocument to export
/// * `file_path` - Path where the AsciiDoc file should be written
/// * `format` - The AsciiDoc format to use (table or list)
///
/// # Returns
///
/// Result indicating success or failure of the export operation
///
/// # Errors
///
/// Returns an error if file writing fails
pub fn export_to_file_with_format<P: AsRef<Path>>(
    document: &BicepDocument,
    file_path: P,
    format: AsciiDocFormat,
) -> Result<(), Box<dyn StdError>> {
    let asciidoc_content = export_to_string_with_format(document, format)?;
    fs::write(file_path, asciidoc_content)?;
    Ok(())
}

/// Export a Bicep document to an AsciiDoc string
///
/// # Arguments
///
/// * `document` - The BicepDocument to export
///
/// # Returns
///
/// Result containing the AsciiDoc string representation of the document
///
/// # Errors
///
/// Returns an error if serialization fails
pub fn export_to_string(document: &BicepDocument) -> Result<String, Box<dyn StdError>> {
    export_to_string_with_format(document, AsciiDocFormat::default())
}

/// Export a Bicep document to an AsciiDoc string with specific format
///
/// # Arguments
///
/// * `document` - The BicepDocument to export
/// * `format` - The AsciiDoc format to use (table or list)
///
/// # Returns
///
/// Result containing the AsciiDoc string representation of the document
///
/// # Errors
///
/// Returns an error if serialization fails
pub fn export_to_string_with_format(
    document: &BicepDocument,
    format: AsciiDocFormat,
) -> Result<String, Box<dyn StdError>> {
    let mut asciidoc = String::new();

    // Title and overview section
    if let Some(name) = &document.name {
        asciidoc.push_str(&format!("= {}\n\n", name));
    } else {
        asciidoc.push_str("= Bicep Template\n\n");
    }

    // Description
    if let Some(description) = &document.description {
        asciidoc.push_str(&format!("{}\n\n", description));
    }

    if let Some(target_scope) = &document.target_scope {
        asciidoc.push_str(&format!(
            "Target Scope: {}\n",
            escape_asciidoc(target_scope)
        ));
    }

    // Additional metadata
    if !document.metadata.is_empty() {
        asciidoc.push_str("\n=== Additional Metadata\n\n");
        generate_metadata_display(&mut asciidoc, format, &document.metadata);
    }

    asciidoc.push('\n');

    // Imports section
    asciidoc.push_str("== Imports\n\n");
    if !document.imports.is_empty() {
        // Separate namespace and module imports
        let namespace_imports: Vec<_> = document
            .imports
            .iter()
            .filter_map(|imp| match imp {
                BicepImport::Namespace { .. } => Some(imp),
                _ => None,
            })
            .collect();
        let module_imports: Vec<_> = document
            .imports
            .iter()
            .filter_map(|imp| match imp {
                BicepImport::Module { .. } => Some(imp),
                _ => None,
            })
            .collect();

        if !namespace_imports.is_empty() {
            asciidoc.push_str("=== Namespace Imports\n\n");
            asciidoc.push_str("|===\n");
            asciidoc.push_str("| Namespace | Version\n\n");

            for import in namespace_imports {
                if let BicepImport::Namespace { namespace, version } = import {
                    let version_str = version.as_deref().unwrap_or("N/A");
                    asciidoc.push_str(&format!(
                        "| {} | {}\n",
                        escape_asciidoc(namespace),
                        escape_asciidoc(version_str)
                    ));
                }
            }
            asciidoc.push_str("|===\n\n");
        }

        if !module_imports.is_empty() {
            asciidoc.push_str("=== Module Imports\n\n");
            asciidoc.push_str("|===\n");
            asciidoc.push_str("| Source | Symbols | Wildcard Alias\n\n");

            for import in module_imports {
                if let BicepImport::Module {
                    source,
                    symbols,
                    wildcard_alias,
                } = import
                {
                    let symbols_str = if let Some(symbols) = symbols {
                        if symbols.is_empty() {
                            "None".to_string()
                        } else {
                            symbols
                                .iter()
                                .map(|sym| {
                                    if let Some(alias) = &sym.alias {
                                        format!("{} as {}", sym.name, alias)
                                    } else {
                                        sym.name.clone()
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join(", ")
                        }
                    } else {
                        "None".to_string()
                    };

                    let wildcard_str = wildcard_alias.as_deref().unwrap_or("N/A");

                    asciidoc.push_str(&format!(
                        "| {} | {} | {}\n",
                        escape_asciidoc(&source.to_string()),
                        escape_asciidoc(&symbols_str),
                        escape_asciidoc(wildcard_str)
                    ));
                }
            }
            asciidoc.push_str("|===\n\n");
        }
    } else {
        asciidoc.push_str("_No imports defined_\n\n");
    }

    // Types section
    generate_types_section(&mut asciidoc, document, format);

    // Functions section
    generate_functions_section(&mut asciidoc, document, format);

    // Parameters section
    generate_parameters_section(&mut asciidoc, document, format);

    // Variables section
    generate_variables_section(&mut asciidoc, document, format);

    // Resources section
    generate_resources_section(&mut asciidoc, document, format);

    // Modules section
    generate_modules_section(&mut asciidoc, document, format);

    // Outputs section
    generate_outputs_section(&mut asciidoc, document, format);

    Ok(asciidoc)
}

/// Parse a Bicep file and export it to AsciiDoc
///
/// # Arguments
///
/// * `file_path` - Path to the Bicep file to parse
/// * `output_path` - Path where the AsciiDoc file should be written
///
/// # Returns
///
/// Result indicating success or failure of the operation
///
/// # Errors
///
/// Returns an error if parsing or export fails
pub fn parse_and_export<P: AsRef<Path>, Q: AsRef<Path>>(
    file_path: P,
    output_path: Q,
) -> Result<(), Box<dyn StdError>> {
    let content = std::fs::read_to_string(file_path)?;
    let document = crate::parse_bicep_document(&content)?;
    export_to_file(&document, output_path)?;
    Ok(())
}

/// Generate the Types section of the AsciiDoc
fn generate_types_section(asciidoc: &mut String, document: &BicepDocument, format: AsciiDocFormat) {
    asciidoc.push_str("== Types\n\n");

    if document.types.is_empty() {
        asciidoc.push_str("_No custom types defined_\n\n");
        return;
    }

    for (name, custom_type) in &document.types {
        asciidoc.push_str(&format!("=== {}\n\n", escape_asciidoc(name)));

        if let Some(description) = &custom_type.description {
            asciidoc.push_str(&format!("{}\n\n", escape_asciidoc(description)));
        }

        // Basic information table
        let items = vec![
            ("Type", format_bicep_type(&custom_type.definition, format)),
            (
                "Exported",
                if custom_type.is_exported {
                    "Yes".to_string()
                } else {
                    "No".to_string()
                },
            ),
            (
                "Secure",
                if custom_type.is_secure {
                    "Yes".to_string()
                } else {
                    "No".to_string()
                },
            ),
        ];
        generate_key_value_display(asciidoc, format, &items);

        asciidoc.push('\n');
    }
}

/// Generate the Functions section of the AsciiDoc
fn generate_functions_section(
    asciidoc: &mut String,
    document: &BicepDocument,
    format: AsciiDocFormat,
) {
    asciidoc.push_str("== Functions\n\n");

    if document.functions.is_empty() {
        asciidoc.push_str("_No user-defined functions_\n\n");
        return;
    }

    for (name, function) in &document.functions {
        asciidoc.push_str(&format!("=== {}\n\n", escape_asciidoc(name)));

        if let Some(description) = &function.description {
            asciidoc.push_str(&format!("{}\n\n", escape_asciidoc(description)));
        }

        // Basic information table
        let items = vec![
            (
                "Return Type",
                format_bicep_type(&function.return_type, format),
            ),
            (
                "Exported",
                if function.is_exported {
                    "Yes".to_string()
                } else {
                    "No".to_string()
                },
            ),
        ];
        generate_key_value_display(asciidoc, format, &items);

        // Parameters
        if !function.arguments.is_empty() {
            asciidoc.push_str("\n==== Parameters\n\n");
            generate_function_arguments_display(asciidoc, format, &function.arguments);
        }

        if !function.metadata.is_empty() {
            asciidoc.push_str("\n==== Metadata\n\n");
            generate_metadata_display(asciidoc, format, &function.metadata);
        }

        asciidoc.push('\n');
    }
}

/// Generate the Parameters section of the AsciiDoc
fn generate_parameters_section(
    asciidoc: &mut String,
    document: &BicepDocument,
    format: AsciiDocFormat,
) {
    asciidoc.push_str("== Parameters\n\n");

    if document.parameters.is_empty() {
        asciidoc.push_str("_No parameters defined_\n\n");
        return;
    }

    for (name, parameter) in &document.parameters {
        asciidoc.push_str(&format!("=== {}\n\n", escape_asciidoc(name)));

        if let Some(description) = &parameter.description {
            asciidoc.push_str(&format!("{}\n\n", escape_asciidoc(description)));
        }

        // Basic information table
        let mut items = vec![("Type", format_bicep_type(&parameter.parameter_type, format))];

        if parameter.is_nullable {
            items.push(("Nullable", "Yes".to_string()));
        }

        if let Some(default_value) = &parameter.default_value {
            let value_str = format_bicep_value_with_format(default_value, format);
            items.push(("Default Value", value_str));
        }

        if let Some(min_value) = parameter.min_value {
            items.push(("Minimum Value", min_value.to_string()));
        }

        if let Some(max_value) = parameter.max_value {
            items.push(("Maximum Value", max_value.to_string()));
        }

        if let Some(min_length) = parameter.min_length {
            items.push(("Minimum Length", min_length.to_string()));
        }

        if let Some(max_length) = parameter.max_length {
            items.push(("Maximum Length", max_length.to_string()));
        }

        if let Some(allowed_values) = &parameter.allowed_values {
            if !allowed_values.is_empty() {
                let values = allowed_values
                    .iter()
                    .map(|v| format_bicep_value_with_format(v, format))
                    .collect::<Vec<_>>()
                    .join(", ");
                items.push(("Allowed Values", values));
            }
        }

        generate_key_value_display(asciidoc, format, &items);

        if !parameter.metadata.is_empty() {
            asciidoc.push_str("\n==== Metadata\n\n");
            generate_metadata_display(asciidoc, format, &parameter.metadata);
        }

        asciidoc.push('\n');
    }
}

/// Generate the Variables section of the AsciiDoc
fn generate_variables_section(
    asciidoc: &mut String,
    document: &BicepDocument,
    format: AsciiDocFormat,
) {
    asciidoc.push_str("== Variables\n\n");

    if document.variables.is_empty() {
        asciidoc.push_str("_No variables defined_\n\n");
        return;
    }

    for (name, variable) in &document.variables {
        asciidoc.push_str(&format!("=== {}\n\n", escape_asciidoc(name)));

        if let Some(description) = &variable.description {
            asciidoc.push_str(&format!("{}\n\n", escape_asciidoc(description)));
        }

        // Basic information table
        let value_str = format_bicep_value_with_format(&variable.value, format);
        let items = vec![
            ("Value", value_str),
            (
                "Exported",
                if variable.is_exported {
                    "Yes".to_string()
                } else {
                    "No".to_string()
                },
            ),
        ];
        generate_key_value_display(asciidoc, format, &items);

        asciidoc.push('\n');
    }
}

/// Generate the Resources section of the AsciiDoc
fn generate_resources_section(
    asciidoc: &mut String,
    document: &BicepDocument,
    format: AsciiDocFormat,
) {
    asciidoc.push_str("== Resources\n\n");

    if document.resources.is_empty() {
        asciidoc.push_str("_No resources defined_\n\n");
        return;
    }

    for (name, resource) in &document.resources {
        asciidoc.push_str(&format!("=== {}\n\n", escape_asciidoc(name)));

        if let Some(description) = &resource.description {
            asciidoc.push_str(&format!("{}\n\n", escape_asciidoc(description)));
        }

        // Basic information table
        let mut items = vec![
            ("Type", resource.resource_type.clone()),
            ("API Version", resource.api_version.clone()),
        ];

        if let Some(scope) = &resource.scope {
            let scope_str = format_bicep_value_with_format(scope, format);
            items.push(("Scope", scope_str));
        }

        if resource.existing {
            items.push(("Existing", "Yes".to_string()));
        }

        if let Some(parent) = &resource.parent {
            items.push(("Parent", parent.clone()));
        }

        if let Some(depends_on) = &resource.depends_on {
            if !depends_on.is_empty() {
                let deps = depends_on.join(", ");
                items.push(("Depends On", deps));
            }
        }

        if let Some(condition) = &resource.condition {
            items.push(("Condition", condition.clone()));
        }

        if let Some(loop_statement) = &resource.loop_statement {
            items.push(("Loop", loop_statement.clone()));
        }

        if let Some(batch_size) = resource.batch_size {
            items.push(("Batch Size", batch_size.to_string()));
        }

        generate_key_value_display(asciidoc, format, &items);

        asciidoc.push('\n');
    }
}

/// Generate the Modules section of the AsciiDoc
fn generate_modules_section(
    asciidoc: &mut String,
    document: &BicepDocument,
    format: AsciiDocFormat,
) {
    asciidoc.push_str("== Modules\n\n");

    if document.modules.is_empty() {
        asciidoc.push_str("_No modules defined_\n\n");
        return;
    }

    for (name, module) in &document.modules {
        asciidoc.push_str(&format!("=== {}\n\n", escape_asciidoc(name)));

        if let Some(description) = &module.description {
            asciidoc.push_str(&format!("{}\n\n", escape_asciidoc(description)));
        }

        // Basic information table
        let source_str = match &module.source {
            ModuleSource::LocalPath(path) => format!("File: {}", path),
            ModuleSource::Registry {
                alias,
                registry_fqdn,
                path,
                version,
            } => {
                if let Some(alias) = alias {
                    format!("Registry: {}:{} ({})", alias, path, version)
                } else if let Some(fqdn) = registry_fqdn {
                    format!("Registry: {}{}:{}", fqdn, path, version)
                } else {
                    format!("Registry: {}:{}", path, version)
                }
            },
            ModuleSource::TypeSpec {
                alias: _,
                subscription_id,
                resource_group_name,
                template_spec_name,
                version,
            } => {
                if let Some(sub_id) = subscription_id {
                    if let Some(rg) = resource_group_name {
                        format!(
                            "Template Spec: {} in {}/{} ({})",
                            template_spec_name, sub_id, rg, version
                        )
                    } else {
                        format!(
                            "Template Spec: {} in {} ({})",
                            template_spec_name, sub_id, version
                        )
                    }
                } else {
                    format!("Template Spec: {} ({})", template_spec_name, version)
                }
            },
        };

        let items = vec![("Source", source_str)];
        generate_key_value_display(asciidoc, format, &items);

        asciidoc.push('\n');
    }
}

/// Generate the Outputs section of the AsciiDoc
fn generate_outputs_section(
    asciidoc: &mut String,
    document: &BicepDocument,
    format: AsciiDocFormat,
) {
    asciidoc.push_str("== Outputs\n\n");

    if document.outputs.is_empty() {
        asciidoc.push_str("_No outputs defined_\n\n");
        return;
    }

    for (name, output) in &document.outputs {
        asciidoc.push_str(&format!("=== {}\n\n", escape_asciidoc(name)));

        if let Some(description) = &output.description {
            asciidoc.push_str(&format!("{}\n\n", escape_asciidoc(description)));
        }

        // Basic information table
        let mut items = vec![
            ("Type", format_bicep_type(&output.output_type, format)),
            (
                "Value",
                format_bicep_value_with_format(&output.value, format),
            ),
        ];

        if let Some(discriminator) = &output.discriminator {
            items.push(("Discriminator", discriminator.clone()));
        }

        if let Some(min_length) = output.min_length {
            items.push(("Minimum Length", min_length.to_string()));
        }

        if let Some(max_length) = output.max_length {
            items.push(("Maximum Length", max_length.to_string()));
        }

        if let Some(min_value) = output.min_value {
            items.push(("Minimum Value", min_value.to_string()));
        }

        if let Some(max_value) = output.max_value {
            items.push(("Maximum Value", max_value.to_string()));
        }

        if output.sealed {
            items.push(("Sealed", "Yes".to_string()));
        }

        if output.secure {
            items.push(("Secure", "Yes".to_string()));
        }

        generate_key_value_display(asciidoc, format, &items);

        if let Some(metadata) = &output.metadata {
            if !metadata.is_empty() {
                asciidoc.push_str("\n==== Metadata\n\n");
                generate_metadata_display(asciidoc, format, metadata);
            }
        }

        asciidoc.push('\n');
    }
}

/// Format a BicepValue for display in AsciiDoc with format-aware handling
fn format_bicep_value_with_format(value: &BicepValue, format: AsciiDocFormat) -> String {
    match value {
        BicepValue::String(s) => s.clone(),
        BicepValue::Int(n) => n.to_string(),
        BicepValue::Bool(b) => b.to_string(),
        BicepValue::Array(arr) => {
            let items: Vec<String> = arr
                .iter()
                .map(|v| format_bicep_value_with_format(v, format))
                .collect();
            format!("[{}]", items.join(", "))
        },
        BicepValue::Object(obj) => {
            if obj.is_empty() {
                "{}".to_string()
            } else {
                let items: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, format_bicep_value_with_format(v, format)))
                    .collect();
                format!("{{ {} }}", items.join(", "))
            }
        },
        BicepValue::Identifier(id) => format!("${{{}}}", id),
    }
}

/// Format a BicepValue for display in AsciiDoc (legacy function for compatibility)
#[allow(dead_code)]
fn format_bicep_value(value: &BicepValue) -> String {
    format_bicep_value_with_format(value, AsciiDocFormat::Table)
}

/// Format a BicepType for display in AsciiDoc with format-aware escaping
fn format_bicep_type(bicep_type: &BicepType, format: AsciiDocFormat) -> String {
    match bicep_type {
        BicepType::Array(inner) => format!("{}[]", format_bicep_type(inner, format)),
        BicepType::String => "string".to_string(),
        BicepType::Int => "int".to_string(),
        BicepType::Bool => "bool".to_string(),
        BicepType::Object(Some(properties)) => {
            // Format object with properties
            if properties.is_empty() {
                "object".to_string()
            } else {
                let props: Vec<String> = properties
                    .iter()
                    .map(|(key, param)| {
                        format!(
                            "{}: {}",
                            key,
                            format_bicep_type(&param.parameter_type, format)
                        )
                    })
                    .collect();
                format!("{{ {} }}", props.join(", "))
            }
        },
        BicepType::Object(None) => "object".to_string(),
        BicepType::CustomType(name) => name.clone(),
        BicepType::Union(values) => {
            match format {
                AsciiDocFormat::Table => {
                    // Escape | characters for AsciiDoc tables
                    values.join(" \\| ")
                },
                AsciiDocFormat::List => {
                    // No escaping needed for list format
                    values.join(" | ")
                },
            }
        },
    }
}

/// Escape special AsciiDoc characters in text
fn escape_asciidoc(text: &str) -> String {
    text.replace('*', "\\*")
        .replace('_', "\\_")
        .replace('`', "\\`")
        .replace('#', "\\#")
        .replace('\n', " +\n") // Preserve newlines
}

/// Generate property display for BicepValue properties using either table or list format
fn generate_metadata_display(
    asciidoc: &mut String,
    format: AsciiDocFormat,
    metadata: &indexmap::IndexMap<String, BicepValue>,
) {
    match format {
        AsciiDocFormat::Table => {
            asciidoc.push_str("|===\n");
            asciidoc.push_str("| Key | Value\n\n");
            for (key, value) in metadata {
                let value_str = format_bicep_value_with_format(value, format);
                asciidoc.push_str(&format!(
                    "| {} | {}\n",
                    escape_asciidoc(key),
                    escape_asciidoc(&value_str)
                ));
            }
            asciidoc.push_str("|===\n");
        },
        AsciiDocFormat::List => {
            for (key, value) in metadata {
                let value_str = format_bicep_value_with_format(value, format);
                asciidoc.push_str(&format!(
                    "* *{}*: {}\n",
                    escape_asciidoc(key),
                    escape_asciidoc(&value_str)
                ));
            }
        },
    }
}

/// Generate key-value property display with optional values
fn generate_key_value_display(
    asciidoc: &mut String,
    format: AsciiDocFormat,
    items: &[(&str, String)],
) {
    match format {
        AsciiDocFormat::Table => {
            asciidoc.push_str("|===\n");
            asciidoc.push_str("| Property | Value\n\n");
            for (key, value) in items {
                asciidoc.push_str(&format!(
                    "| {} | {}\n",
                    escape_asciidoc(key),
                    escape_asciidoc(value)
                ));
            }
            asciidoc.push_str("|===\n");
        },
        AsciiDocFormat::List => {
            for (key, value) in items {
                asciidoc.push_str(&format!(
                    "* *{}*: {}\n",
                    escape_asciidoc(key),
                    escape_asciidoc(value)
                ));
            }
        },
    }
}

/// Generate display for function arguments in table or list format
///
/// # Arguments
///
/// * `asciidoc` - The string buffer to append AsciiDoc content to
/// * `format` - The format to use (table or list)
/// * `arguments` - The function arguments to display
fn generate_function_arguments_display(
    asciidoc: &mut String,
    format: AsciiDocFormat,
    arguments: &[BicepFunctionArgument],
) {
    match format {
        AsciiDocFormat::Table => {
            asciidoc.push_str("|===\n");
            asciidoc.push_str("| Parameter | Type | Optional\n\n");
            for arg in arguments {
                asciidoc.push_str(&format!(
                    "| {} | {} | {}\n",
                    escape_asciidoc(&arg.name),
                    escape_asciidoc(&format_bicep_type(&arg.argument_type, format)),
                    if arg.is_nullable { "Yes" } else { "No" }
                ));
            }
            asciidoc.push_str("|===\n");
        },
        AsciiDocFormat::List => {
            for arg in arguments {
                asciidoc.push_str(&format!(
                    "* *{}* ({}){}\n",
                    escape_asciidoc(&arg.name),
                    escape_asciidoc(&format_bicep_type(&arg.argument_type, format)),
                    if arg.is_nullable { " - Optional" } else { "" }
                ));
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::{BicepDocument, BicepParameter, BicepType, BicepValue};

    #[test]
    fn test_export_to_string_basic() {
        let mut document = BicepDocument::default();
        document.name = Some("Test Template".to_string());
        document.description = Some("A test template for unit testing".to_string());
        document.target_scope = Some("resourceGroup".to_string());

        let result = export_to_string(&document);
        assert!(result.is_ok());

        let asciidoc = result.unwrap();
        assert!(asciidoc.contains("= Test Template"));
        assert!(asciidoc.contains("A test template for unit testing"));
        assert!(asciidoc.contains("resourceGroup"));
    }

    #[test]
    fn test_export_to_string_with_parameters() {
        let mut document = BicepDocument::default();
        document.name = Some("Test Template".to_string());

        let mut parameter = BicepParameter::default();
        parameter.parameter_type = BicepType::String;
        parameter.description = Some("Test parameter".to_string());
        parameter.default_value = Some(BicepValue::String("default".to_string()));

        document
            .parameters
            .insert("testParam".to_string(), parameter);

        let result = export_to_string(&document);
        assert!(result.is_ok());

        let asciidoc = result.unwrap();
        assert!(asciidoc.contains("== Parameters"));
        assert!(asciidoc.contains("=== testParam"));
        assert!(asciidoc.contains("Test parameter"));
        assert!(asciidoc.contains("default"));
    }

    #[test]
    fn test_escape_asciidoc() {
        let text = "test | with * special _ characters [and] `code` #heading";
        let escaped = escape_asciidoc(text);
        assert_eq!(
            escaped,
            "test | with \\* special \\_ characters [and] \\`code\\` \\#heading"
        );
    }

    #[test]
    fn test_format_bicep_value() {
        assert_eq!(
            format_bicep_value(&BicepValue::String("test".to_string())),
            "test"
        );
        assert_eq!(format_bicep_value(&BicepValue::Int(42)), "42");
        assert_eq!(format_bicep_value(&BicepValue::Bool(true)), "true");
        assert_eq!(
            format_bicep_value(&BicepValue::Identifier("myVar".to_string())),
            "${myVar}"
        );
    }

    #[test]
    fn test_format_bicep_type() {
        assert_eq!(
            format_bicep_type(&BicepType::String, AsciiDocFormat::Table),
            "string"
        );
        assert_eq!(
            format_bicep_type(&BicepType::Int, AsciiDocFormat::Table),
            "int"
        );
        assert_eq!(
            format_bicep_type(&BicepType::Bool, AsciiDocFormat::Table),
            "bool"
        );
        assert_eq!(
            format_bicep_type(
                &BicepType::Array(Box::new(BicepType::String)),
                AsciiDocFormat::Table
            ),
            "string[]"
        );
        assert_eq!(
            format_bicep_type(
                &BicepType::CustomType("MyType".to_string()),
                AsciiDocFormat::Table
            ),
            "MyType"
        );
        assert_eq!(
            format_bicep_type(
                &BicepType::Union(vec!["A".to_string(), "B".to_string()]),
                AsciiDocFormat::Table
            ),
            "A \\| B"
        );

        // Test Object types
        assert_eq!(
            format_bicep_type(&BicepType::Object(None), AsciiDocFormat::Table),
            "object"
        );

        // Test empty object with properties
        use indexmap::IndexMap;
        let empty_props = IndexMap::new();
        assert_eq!(
            format_bicep_type(&BicepType::Object(Some(empty_props)), AsciiDocFormat::Table),
            "object"
        );
    }

    #[test]
    fn test_format_bicep_value_object() {
        use indexmap::IndexMap;
        let mut obj = IndexMap::new();
        obj.insert("key1".to_string(), BicepValue::String("value1".to_string()));
        obj.insert("key2".to_string(), BicepValue::Int(42));

        let result = format_bicep_value(&BicepValue::Object(obj));
        assert!(result.contains("key1: value1"));
        assert!(result.contains("key2: 42"));
    }

    #[test]
    fn test_format_bicep_type_union_formats() {
        let union_type = BicepType::Union(vec!["A".to_string(), "B".to_string()]);

        // Test table format (should escape pipes)
        assert_eq!(
            format_bicep_type(&union_type, AsciiDocFormat::Table),
            "A \\| B"
        );

        // Test list format (should not escape pipes)
        assert_eq!(
            format_bicep_type(&union_type, AsciiDocFormat::List),
            "A | B"
        );
    }

    #[test]
    fn test_format_bicep_value_with_multiline_string() {
        // Test multiline string in table format
        let multiline = BicepValue::String("line1\nline2\nline3".to_string());
        let result = format_bicep_value_with_format(&multiline, AsciiDocFormat::Table);
        assert_eq!(result, "line1\nline2\nline3");

        // Test multiline string in list format
        let result = format_bicep_value_with_format(&multiline, AsciiDocFormat::List);
        assert_eq!(result, "line1\nline2\nline3");
    }
}
