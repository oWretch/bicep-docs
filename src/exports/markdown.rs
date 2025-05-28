/// Markdown export functionality for Bicep documents
///
/// This module provides functions to export parsed Bicep documents
/// to Markdown format with structured documentation layout.
use std::error::Error as StdError;
use std::fs;
use std::path::Path;

use crate::parsing::{
    BicepDocument, BicepFunctionArgument, BicepImport, BicepType, BicepValue, ModuleSource,
};

/// Format options for markdown output
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MarkdownFormat {
    /// Use tables for displaying properties (default)
    #[default]
    Table,
    /// Use lists for displaying properties
    List,
}

impl std::str::FromStr for MarkdownFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "table" => Ok(MarkdownFormat::Table),
            "list" => Ok(MarkdownFormat::List),
            _ => Err(format!("Invalid format '{}'. Use 'table' or 'list'", s)),
        }
    }
}

/// Export a Bicep document to a Markdown file
///
/// # Arguments
///
/// * `document` - The BicepDocument to export
/// * `file_path` - Path where the Markdown file should be written
/// * `format` - The markdown format to use (table or list)
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
    export_to_file_with_format(document, file_path, MarkdownFormat::default())
}

/// Export a Bicep document to a Markdown file with specific format
///
/// # Arguments
///
/// * `document` - The BicepDocument to export
/// * `file_path` - Path where the Markdown file should be written
/// * `format` - The markdown format to use (table or list)
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
    format: MarkdownFormat,
) -> Result<(), Box<dyn StdError>> {
    let markdown_content = export_to_string_with_format(document, format)?;
    fs::write(file_path, markdown_content)?;
    Ok(())
}

/// Export a Bicep document to a Markdown string
///
/// # Arguments
///
/// * `document` - The BicepDocument to export
///
/// # Returns
///
/// Result containing the Markdown string representation of the document
///
/// # Errors
///
/// Returns an error if serialization fails
pub fn export_to_string(document: &BicepDocument) -> Result<String, Box<dyn StdError>> {
    export_to_string_with_format(document, MarkdownFormat::default())
}

/// Export a Bicep document to a Markdown string with specific format
///
/// # Arguments
///
/// * `document` - The BicepDocument to export
/// * `format` - The markdown format to use (table or list)
///
/// # Returns
///
/// Result containing the Markdown string representation of the document
///
/// # Errors
///
/// Returns an error if serialization fails
pub fn export_to_string_with_format(
    document: &BicepDocument,
    format: MarkdownFormat,
) -> Result<String, Box<dyn StdError>> {
    let mut markdown = String::new();

    // Title and overview section
    if let Some(name) = &document.name {
        markdown.push_str(&format!("# {}\n\n", name));
    } else {
        markdown.push_str("# Bicep Template\n\n");
    }

    // Description
    if let Some(description) = &document.description {
        markdown.push_str(&format!("{}\n\n", description));
    }

    if let Some(target_scope) = &document.target_scope {
        markdown.push_str(&format!(
            "Target Scope: {}\n",
            escape_markdown_table(target_scope)
        ));
    }

    // Additional metadata
    if !document.metadata.is_empty() {
        markdown.push_str("\n### Additional Metadata\n\n");

        generate_metadata_display(&mut markdown, format, &document.metadata);
    }

    markdown.push('\n');

    // Imports section
    markdown.push_str("## Imports\n\n");
    if !document.imports.is_empty() {
        // Separate namespace and module imports
        let namespace_imports: Vec<_> = document
            .imports
            .iter()
            .filter(|imp| matches!(imp, BicepImport::Namespace { .. }))
            .collect();
        let module_imports: Vec<_> = document
            .imports
            .iter()
            .filter(|imp| matches!(imp, BicepImport::Module { .. }))
            .collect();

        if !namespace_imports.is_empty() {
            markdown.push_str("### Namespace Imports\n\n");
            markdown.push_str("| Namespace | Version |\n");
            markdown.push_str("|-----------|----------|\n");

            for import in namespace_imports {
                if let BicepImport::Namespace { namespace, version } = import {
                    let version_str = version.as_deref().unwrap_or("N/A");
                    markdown.push_str(&format!(
                        "| {} | {} |\n",
                        escape_markdown_table(namespace),
                        escape_markdown_table(version_str)
                    ));
                }
            }
            markdown.push('\n');
        }

        if !module_imports.is_empty() {
            markdown.push_str("### Module Imports\n\n");
            markdown.push_str("| Source | Symbols | Wildcard Alias |\n");
            markdown.push_str("|--------|---------|----------------|\n");

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

                    markdown.push_str(&format!(
                        "| {} | {} | {} |\n",
                        escape_markdown_table(&source.to_string()),
                        escape_markdown_table(&symbols_str),
                        escape_markdown_table(wildcard_str)
                    ));
                }
            }
            markdown.push('\n');
        }
    } else {
        markdown.push_str("*No imports defined*\n\n");
    }

    // Types section
    generate_types_section(&mut markdown, document, format);

    // Functions section
    generate_functions_section(&mut markdown, document, format);

    // Parameters section
    generate_parameters_section(&mut markdown, document, format);

    // Variables section
    generate_variables_section(&mut markdown, document, format);

    // Resources section
    generate_resources_section(&mut markdown, document, format);

    // Modules section
    generate_modules_section(&mut markdown, document, format);

    // Outputs section
    generate_outputs_section(&mut markdown, document, format);

    Ok(markdown)
}

/// Parse a Bicep file and export it to Markdown
///
/// # Arguments
///
/// * `file_path` - Path to the Bicep file to parse
/// * `output_path` - Path where the Markdown file should be written
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

/// Generate the Types section of the markdown
fn generate_types_section(markdown: &mut String, document: &BicepDocument, format: MarkdownFormat) {
    markdown.push_str("## Types\n\n");

    if document.types.is_empty() {
        markdown.push_str("*No custom types defined*\n\n");
        return;
    }

    for (name, custom_type) in &document.types {
        markdown.push_str(&format!("### {}\n\n", escape_markdown_table(name)));

        if let Some(description) = &custom_type.description {
            markdown.push_str(&format!("{}\n\n", escape_markdown_table(description)));
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
        generate_key_value_display(markdown, format, &items);

        markdown.push('\n');
    }
}

/// Generate the Functions section of the markdown
fn generate_functions_section(
    markdown: &mut String,
    document: &BicepDocument,
    format: MarkdownFormat,
) {
    markdown.push_str("## Functions\n\n");

    if document.functions.is_empty() {
        markdown.push_str("*No user-defined functions*\n\n");
        return;
    }

    for (name, function) in &document.functions {
        markdown.push_str(&format!("### {}\n\n", escape_markdown_table(name)));

        if let Some(description) = &function.description {
            markdown.push_str(&format!("{}\n\n", escape_markdown_table(description)));
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
        generate_key_value_display(markdown, format, &items);

        // Parameters
        if !function.arguments.is_empty() {
            markdown.push_str("\n#### Parameters\n\n");
            generate_function_arguments_display(markdown, format, &function.arguments);
        }

        if !function.metadata.is_empty() {
            markdown.push_str("\n#### Metadata\n\n");
            generate_metadata_display(markdown, format, &function.metadata);
        }

        markdown.push('\n');
    }
}

/// Generate the Parameters section of the markdown
fn generate_parameters_section(
    markdown: &mut String,
    document: &BicepDocument,
    format: MarkdownFormat,
) {
    markdown.push_str("## Parameters\n\n");

    if document.parameters.is_empty() {
        markdown.push_str("*No parameters defined*\n\n");
        return;
    }

    for (name, parameter) in &document.parameters {
        markdown.push_str(&format!("### {}\n\n", escape_markdown_table(name)));

        if let Some(description) = &parameter.description {
            markdown.push_str(&format!("{}\n\n", escape_markdown_table(description)));
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

        generate_key_value_display(markdown, format, &items);

        if !parameter.metadata.is_empty() {
            markdown.push_str("\n#### Metadata\n\n");
            generate_metadata_display(markdown, format, &parameter.metadata);
        }

        markdown.push('\n');
    }
}

/// Generate the Variables section of the markdown
fn generate_variables_section(
    markdown: &mut String,
    document: &BicepDocument,
    format: MarkdownFormat,
) {
    markdown.push_str("## Variables\n\n");

    if document.variables.is_empty() {
        markdown.push_str("*No variables defined*\n\n");
        return;
    }

    for (name, variable) in &document.variables {
        markdown.push_str(&format!("### {}\n\n", escape_markdown_table(name)));

        if let Some(description) = &variable.description {
            markdown.push_str(&format!("{}\n\n", escape_markdown_table(description)));
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
        generate_key_value_display(markdown, format, &items);

        markdown.push('\n');
    }
}

/// Generate the Resources section of the markdown
fn generate_resources_section(
    markdown: &mut String,
    document: &BicepDocument,
    format: MarkdownFormat,
) {
    markdown.push_str("## Resources\n\n");

    if document.resources.is_empty() {
        markdown.push_str("*No resources defined*\n\n");
        return;
    }

    for (name, resource) in &document.resources {
        markdown.push_str(&format!("### {}\n\n", escape_markdown_table(name)));

        if let Some(description) = &resource.description {
            markdown.push_str(&format!("{}\n\n", escape_markdown_table(description)));
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

        generate_key_value_display(markdown, format, &items);

        markdown.push('\n');
    }
}

/// Generate the Modules section of the markdown
fn generate_modules_section(
    markdown: &mut String,
    document: &BicepDocument,
    format: MarkdownFormat,
) {
    markdown.push_str("## Modules\n\n");

    if document.modules.is_empty() {
        markdown.push_str("*No modules defined*\n\n");
        return;
    }

    for (name, module) in &document.modules {
        markdown.push_str(&format!("### {}\n\n", escape_markdown_table(name)));

        if let Some(description) = &module.description {
            markdown.push_str(&format!("{}\n\n", escape_markdown_table(description)));
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
        let mut items = vec![("Source", source_str), ("Name", module.name.clone())];

        if let Some(depends_on) = &module.depends_on {
            if !depends_on.is_empty() {
                let deps = depends_on.join(", ");
                items.push(("Depends On", deps));
            }
        }

        if let Some(condition) = &module.condition {
            items.push(("Condition", condition.clone()));
        }

        if let Some(loop_statement) = &module.loop_statement {
            items.push(("Loop", loop_statement.clone()));
        }

        if let Some(batch_size) = module.batch_size {
            items.push(("Batch Size", batch_size.to_string()));
        }

        generate_key_value_display(markdown, format, &items);

        markdown.push('\n');
    }
}

/// Generate the Outputs section of the markdown
fn generate_outputs_section(
    markdown: &mut String,
    document: &BicepDocument,
    format: MarkdownFormat,
) {
    markdown.push_str("## Outputs\n\n");

    if document.outputs.is_empty() {
        markdown.push_str("*No outputs defined*\n\n");
        return;
    }

    for (name, output) in &document.outputs {
        markdown.push_str(&format!("### {}\n\n", escape_markdown_table(name)));

        if let Some(description) = &output.description {
            markdown.push_str(&format!("{}\n\n", escape_markdown_table(description)));
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

        generate_key_value_display(markdown, format, &items);

        if let Some(metadata) = &output.metadata {
            if !metadata.is_empty() {
                markdown.push_str("\n#### Metadata\n\n");
                generate_metadata_display(markdown, format, metadata);
            }
        }

        markdown.push('\n');
    }
}

/// Format a BicepValue for display in markdown with format awareness
#[allow(clippy::only_used_in_recursion)]
fn format_bicep_value_with_format(value: &BicepValue, format: MarkdownFormat) -> String {
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

/// Format a BicepType for display in markdown with format-aware escaping
fn format_bicep_type(bicep_type: &BicepType, format: MarkdownFormat) -> String {
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
                MarkdownFormat::Table => {
                    // Escape | characters for markdown tables
                    values.join(" \\| ")
                },
                MarkdownFormat::List => {
                    // No escaping needed for list format
                    values.join(" | ")
                },
            }
        },
    }
}

/// Escape special markdown characters for use in table cells
fn escape_markdown_table(text: &str) -> String {
    escape_markdown(text)
        .replace('|', "\\|")
        .replace("  \n", "\\n")
}

/// Escape special markdown characters in text
fn escape_markdown(text: &str) -> String {
    text.replace('*', "\\*")
        .replace('_', "\\_")
        .replace('`', "\\`")
        .replace('#', "\\#")
        .replace('\\', "\\\\")
        .replace('\n', "  \n") // Preserve newlines
}

/// Generate property display for BicepValue properties using either table or list format
fn generate_metadata_display(
    markdown: &mut String,
    format: MarkdownFormat,
    metadata: &indexmap::IndexMap<String, BicepValue>,
) {
    match format {
        MarkdownFormat::Table => {
            markdown.push_str("| Key | Value |\n");
            markdown.push_str("|-----|-------|\n");
            for (key, value) in metadata {
                let value_str = format_bicep_value_with_format(value, format);
                markdown.push_str(&format!(
                    "| {} | {} |\n",
                    escape_markdown_table(key),
                    escape_markdown_table(&value_str)
                ));
            }
        },
        MarkdownFormat::List => {
            for (key, value) in metadata {
                let value_str = format_bicep_value_with_format(value, format);
                markdown.push_str(&format!(
                    "- **{}**: {}\n",
                    escape_markdown(key),
                    escape_markdown(&value_str)
                ));
            }
        },
    }
}

/// Generate key-value property display with optional values
fn generate_key_value_display(
    markdown: &mut String,
    format: MarkdownFormat,
    items: &[(&str, String)],
) {
    match format {
        MarkdownFormat::Table => {
            markdown.push_str("| Property | Value |\n");
            markdown.push_str("|----------|-------|\n");
            for (key, value) in items {
                markdown.push_str(&format!(
                    "| {} | {} |\n",
                    escape_markdown_table(key),
                    escape_markdown_table(value)
                ));
            }
        },
        MarkdownFormat::List => {
            for (key, value) in items {
                markdown.push_str(&format!(
                    "- **{}**: {}\n",
                    escape_markdown(key),
                    escape_markdown(value)
                ));
            }
        },
    }
}

/// Generate display for function arguments in table or list format
///
/// # Arguments
///
/// * `markdown` - The string buffer to append markdown content to
/// * `format` - The format to use (table or list)
/// * `arguments` - The function arguments to display
fn generate_function_arguments_display(
    markdown: &mut String,
    format: MarkdownFormat,
    arguments: &[BicepFunctionArgument],
) {
    match format {
        MarkdownFormat::Table => {
            markdown.push_str("| Parameter | Type | Optional |\n");
            markdown.push_str("|-----------|------|----------|\n");
            for arg in arguments {
                markdown.push_str(&format!(
                    "| {} | {} | {} |\n",
                    escape_markdown_table(&arg.name),
                    escape_markdown_table(&format_bicep_type(&arg.argument_type, format)),
                    if arg.is_nullable { "Yes" } else { "No" }
                ));
            }
        },
        MarkdownFormat::List => {
            for arg in arguments {
                markdown.push_str(&format!(
                    "- **{}** ({}){}\n",
                    escape_markdown(&arg.name),
                    escape_markdown(&format_bicep_type(&arg.argument_type, format)),
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
        let document = BicepDocument {
            name: Some("Test Template".to_string()),
            description: Some("A test template for unit testing".to_string()),
            target_scope: Some("resourceGroup".to_string()),
            ..Default::default()
        };

        let result = export_to_string(&document);
        assert!(result.is_ok());

        let markdown = result.unwrap();
        assert!(markdown.contains("# Test Template"));
        assert!(markdown.contains("A test template for unit testing"));
        assert!(markdown.contains("resourceGroup"));
    }

    #[test]
    fn test_export_to_string_with_parameters() {
        let parameter = BicepParameter {
            parameter_type: BicepType::String,
            description: Some("Test parameter".to_string()),
            default_value: Some(BicepValue::String("default".to_string())),
            ..Default::default()
        };

        let mut document = BicepDocument {
            name: Some("Test Template".to_string()),
            ..Default::default()
        };
        document
            .parameters
            .insert("testParam".to_string(), parameter);

        let result = export_to_string(&document);
        assert!(result.is_ok());

        let markdown = result.unwrap();
        assert!(markdown.contains("## Parameters"));
        assert!(markdown.contains("### testParam"));
        assert!(markdown.contains("Test parameter"));
        assert!(markdown.contains("default"));
    }

    #[test]
    fn test_escape_markdown_table() {
        let text = "test | with * special _ characters [and] `code` #heading";
        let escaped = escape_markdown(text);
        assert_eq!(
            escaped,
            "test | with \\\\* special \\\\_ characters [and] \\\\`code\\\\` \\\\#heading"
        );
    }

    #[test]
    fn test_format_bicep_value() {
        // Test basic values with default table format
        assert_eq!(
            format_bicep_value_with_format(
                &BicepValue::String("test".to_string()),
                MarkdownFormat::Table
            ),
            "test"
        );
        assert_eq!(
            format_bicep_value_with_format(&BicepValue::Int(42), MarkdownFormat::Table),
            "42"
        );
        assert_eq!(
            format_bicep_value_with_format(&BicepValue::Bool(true), MarkdownFormat::Table),
            "true"
        );
        assert_eq!(
            format_bicep_value_with_format(
                &BicepValue::Identifier("myVar".to_string()),
                MarkdownFormat::Table
            ),
            "${myVar}"
        );

        // Test multiline strings in table format
        assert_eq!(
            format_bicep_value_with_format(
                &BicepValue::String("line1\nline2".to_string()),
                MarkdownFormat::Table
            ),
            "line1\nline2"
        );

        // Test multiline strings in list format
        assert_eq!(
            format_bicep_value_with_format(
                &BicepValue::String("line1\nline2".to_string()),
                MarkdownFormat::List
            ),
            "line1\nline2"
        );

        // Test backslash handling in both formats
        assert_eq!(
            format_bicep_value_with_format(
                &BicepValue::String("Has\\backslash".to_string()),
                MarkdownFormat::Table
            ),
            "Has\\backslash"
        );
        assert_eq!(
            format_bicep_value_with_format(
                &BicepValue::String("Has\\backslash".to_string()),
                MarkdownFormat::List
            ),
            "Has\\backslash"
        );
    }

    #[test]
    fn test_format_bicep_type() {
        assert_eq!(
            format_bicep_type(&BicepType::String, MarkdownFormat::Table),
            "string"
        );
        assert_eq!(
            format_bicep_type(&BicepType::Int, MarkdownFormat::Table),
            "int"
        );
        assert_eq!(
            format_bicep_type(&BicepType::Bool, MarkdownFormat::Table),
            "bool"
        );
        assert_eq!(
            format_bicep_type(
                &BicepType::Array(Box::new(BicepType::String)),
                MarkdownFormat::Table
            ),
            "string[]"
        );
        assert_eq!(
            format_bicep_type(
                &BicepType::CustomType("MyType".to_string()),
                MarkdownFormat::Table
            ),
            "MyType"
        );
        assert_eq!(
            format_bicep_type(
                &BicepType::Union(vec!["A".to_string(), "B".to_string()]),
                MarkdownFormat::Table
            ),
            "A \\| B"
        );

        // Test Object types
        assert_eq!(
            format_bicep_type(&BicepType::Object(None), MarkdownFormat::Table),
            "object"
        );

        // Test empty object with properties
        use indexmap::IndexMap;
        let empty_props = IndexMap::new();
        assert_eq!(
            format_bicep_type(&BicepType::Object(Some(empty_props)), MarkdownFormat::Table),
            "object"
        );

        // Test object with properties
        let mut props = IndexMap::new();
        let param = BicepParameter {
            parameter_type: BicepType::String,
            description: None,
            metadata: IndexMap::new(),
            default_value: None,
            discriminator: None,
            allowed_values: None,
            is_nullable: false,
            is_sealed: false,
            is_secure: false,
            max_length: None,
            min_length: None,
            max_value: None,
            min_value: None,
        };
        props.insert("name".to_string(), param);
        assert_eq!(
            format_bicep_type(&BicepType::Object(Some(props)), MarkdownFormat::Table),
            "{ name: string }"
        );
    }
    #[test]
    fn test_format_bicep_value_object() {
        use indexmap::IndexMap;

        // Test empty object
        let empty_obj = IndexMap::new();
        assert_eq!(
            format_bicep_value_with_format(&BicepValue::Object(empty_obj), MarkdownFormat::Table),
            "{}"
        );

        // Test object with properties
        let mut obj = IndexMap::new();
        obj.insert("key1".to_string(), BicepValue::String("value1".to_string()));
        obj.insert("key2".to_string(), BicepValue::Int(42));
        assert_eq!(
            format_bicep_value_with_format(&BicepValue::Object(obj), MarkdownFormat::Table),
            "{ key1: value1, key2: 42 }"
        );

        // Test object with multiline string in different formats
        let mut obj_ml = IndexMap::new();
        obj_ml.insert(
            "text".to_string(),
            BicepValue::String("line1\nline2".to_string()),
        );

        // Table format should escape newlines
        assert_eq!(
            format_bicep_value_with_format(
                &BicepValue::Object(obj_ml.clone()),
                MarkdownFormat::Table
            ),
            "{ text: line1\nline2 }"
        );

        // List format should preserve newlines
        assert_eq!(
            format_bicep_value_with_format(&BicepValue::Object(obj_ml), MarkdownFormat::List),
            "{ text: line1\nline2 }"
        );
    }

    #[test]
    fn test_format_bicep_type_union_formats() {
        // Test that union types are formatted differently for table vs list format
        let union_type = BicepType::Union(vec!["string".to_string(), "int".to_string()]);

        // Table format should escape | characters
        assert_eq!(
            format_bicep_type(&union_type, MarkdownFormat::Table),
            "string \\| int"
        );

        // List format should not escape | characters
        assert_eq!(
            format_bicep_type(&union_type, MarkdownFormat::List),
            "string | int"
        );
    }
}
