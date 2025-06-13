/// Markdown export functionality for Bicep documents
///
/// This module provides functions to export parsed Bicep documents
/// to Markdown format with structured documentation layout.
use std::error::Error as StdError;
use std::{fs, path::Path};

use crate::{
    exports::utils::{
        common::{format_yes_no, generate_metadata_display_markdown},
        formatting::{
            escape_markdown, format_bicep_array_as_list, format_bicep_type_with_backticks,
        },
    },
    parsing::{BicepDocument, BicepFunctionArgument, BicepImport, BicepType},
};

/// Export a Bicep document to a Markdown file
///
/// # Arguments
///
/// * `document` - The BicepDocument to export
/// * `file_path` - Path where the Markdown file should be written
/// * `use_emoji` - Whether to use emoji symbols (✅/❌) for Yes/No values
/// * `exclude_empty` - Whether to exclude empty sections from the output
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
    use_emoji: bool,
    exclude_empty: bool,
) -> Result<(), Box<dyn StdError>> {
    let markdown_content = export_to_string(document, use_emoji, exclude_empty)?;
    fs::write(file_path, markdown_content)?;
    Ok(())
}

/// Export a Bicep document to a Markdown string
///
/// # Arguments
///
/// * `document` - The BicepDocument to export
/// * `use_emoji` - Whether to use emoji symbols (✅/❌) for Yes/No values
/// * `exclude_empty` - Whether to exclude empty sections from the output
///
/// # Returns
///
/// Result containing the Markdown string representation of the document
///
/// # Errors
///
/// Returns an error if serialization fails
pub fn export_to_string(
    document: &BicepDocument,
    use_emoji: bool,
    exclude_empty: bool,
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
        markdown.push_str(&format!("{}\n\n", escape_markdown(description)));
    }

    if let Some(target_scope) = &document.target_scope {
        markdown.push_str(&format!("**Target Scope:** `{}`\n\n", target_scope));
    }

    // Additional metadata
    if !document.metadata.is_empty() {
        markdown.push_str("## Additional Metadata\n\n");

        generate_metadata_display_markdown(&mut markdown, &document.metadata);
    }

    // Imports section
    if !document.imports.is_empty() || !exclude_empty {
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
                            escape_markdown(namespace),
                            escape_markdown(version_str)
                        ));
                    }
                }
                markdown.push('\n');
            }

            if !module_imports.is_empty() {
                markdown.push_str("### Module Imports\n\n");
                markdown.push_str("| Source | Symbols |\n");
                markdown.push_str("|--------|---------|\n");

                for import in module_imports {
                    if let BicepImport::Module {
                        source,
                        symbols,
                        wildcard_alias,
                    } = import
                    {
                        let symbols_str = if let Some(symbols) = symbols {
                            symbols
                                .iter()
                                .map(|sym| {
                                    if let Some(alias) = &sym.alias {
                                        format!("`{}` as `{}`", sym.name, alias)
                                    } else {
                                        format!("`{}`", sym.name)
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join(", ")
                        } else {
                            String::new()
                        };
                        let wildcard_str = if let Some(alias) = wildcard_alias {
                            format!("`*` as `{}`", alias)
                        } else {
                            String::new()
                        };
                        markdown.push_str(&format!(
                            "| {} | {}{} | \n",
                            escape_markdown(&source.to_string()),
                            escape_markdown(&symbols_str),
                            escape_markdown(&wildcard_str)
                        ));
                    }
                }
                markdown.push('\n');
            }
        } else if !exclude_empty {
            markdown.push_str("No imports defined.\n\n");
        }
    }

    // Types section
    if !document.types.is_empty() || !exclude_empty {
        generate_types_section(&mut markdown, document, use_emoji, exclude_empty);
    }

    // Functions section
    if !document.functions.is_empty() || !exclude_empty {
        generate_functions_section(&mut markdown, document, use_emoji, exclude_empty);
    }

    // Parameters section
    if !document.parameters.is_empty() || !exclude_empty {
        generate_parameters_section(&mut markdown, document, use_emoji, exclude_empty);
    }

    // Variables section
    if !document.variables.is_empty() || !exclude_empty {
        generate_variables_section(&mut markdown, document, use_emoji, exclude_empty);
    }

    // Resources section
    if !document.resources.is_empty() || !exclude_empty {
        generate_resources_section(&mut markdown, document, use_emoji, exclude_empty);
    }

    // Modules section
    if !document.modules.is_empty() || !exclude_empty {
        generate_modules_section(&mut markdown, document, use_emoji, exclude_empty);
    }

    // Outputs section
    if !document.outputs.is_empty() || !exclude_empty {
        generate_outputs_section(&mut markdown, document, use_emoji, exclude_empty);
    }

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
    exclude_empty: bool,
) -> Result<(), Box<dyn StdError>> {
    let content = std::fs::read_to_string(file_path)?;
    let document = crate::parse_bicep_document(&content)?;
    export_to_file(&document, output_path, true, exclude_empty)?;
    Ok(())
}

#[cfg(test)]
pub fn test_parse_and_export<P: AsRef<Path>, Q: AsRef<Path>>(
    file_path: P,
    output_path: Q,
    exclude_empty: bool,
) -> Result<(), Box<dyn StdError>> {
    parse_and_export(file_path, output_path, exclude_empty)
}

/// Generate the Types section of the markdown
fn generate_types_section(
    markdown: &mut String,
    document: &BicepDocument,
    use_emoji: bool,
    exclude_empty: bool,
) {
    markdown.push_str("## Types\n\n");

    if document.types.is_empty() {
        if !exclude_empty {
            markdown.push_str("*No custom types defined*\n\n");
        }
        return;
    }

    for (name, custom_type) in &document.types {
        markdown.push_str(&format!("### `{}`\n\n", name));

        if let Some(description) = &custom_type.description {
            markdown.push_str(&format!("{}\n\n", escape_markdown(description)));
        }

        // Basic information table
        let items = vec![
            ("Type", format!("`{}`", custom_type.definition)),
            (
                "Exported",
                format_yes_no(custom_type.is_exported, use_emoji),
            ),
            (
                "Nullable",
                format_yes_no(false, use_emoji), // Types are not nullable
            ),
            ("Secure", format_yes_no(custom_type.is_secure, use_emoji)),
        ];
        generate_key_value_display(markdown, &items);

        // Check if this is an object type with properties and add object properties section
        if let BicepType::Object(Some(properties)) = &custom_type.definition {
            if !properties.is_empty() {
                markdown.push_str("\n**Object Definition**\n\n");

                for (prop_name, prop_param) in properties {
                    markdown.push_str(&format!("#### `{}`\n\n", prop_name));

                    if let Some(description) = &prop_param.description {
                        markdown.push_str(&format!("{}\n\n", escape_markdown(description)));
                    }

                    let mut prop_items = vec![("Type", format!("`{}`", prop_param.parameter_type))];

                    prop_items.push(("Nullable", format_yes_no(prop_param.is_nullable, use_emoji)));

                    prop_items.push(("Secure", format_yes_no(prop_param.is_secure, use_emoji)));

                    generate_key_value_display(markdown, &prop_items);

                    // Handle constraints separately
                    let mut constraints = Vec::new();
                    if let Some(min_value) = prop_param.min_value {
                        constraints.push((
                            "Minimum Value",
                            format_constraint_value(&min_value.to_string()),
                        ));
                    }

                    if let Some(max_value) = prop_param.max_value {
                        constraints.push((
                            "Maximum Value",
                            format_constraint_value(&max_value.to_string()),
                        ));
                    }

                    if let Some(min_length) = prop_param.min_length {
                        constraints.push((
                            "Minimum Length",
                            format_constraint_value(&min_length.to_string()),
                        ));
                    }

                    if let Some(max_length) = prop_param.max_length {
                        constraints.push((
                            "Maximum Length",
                            format_constraint_value(&max_length.to_string()),
                        ));
                    }

                    if let Some(allowed_values) = &prop_param.allowed_values {
                        if !allowed_values.is_empty() {
                            constraints.push((
                                "Allowed Values",
                                format_bicep_array_as_list(allowed_values),
                            ));
                        }
                    }

                    if !constraints.is_empty() {
                        markdown.push_str("\n**Constraints**\n\n");
                        generate_key_value_display(markdown, &constraints);
                    }

                    if let Some(default_value) = &prop_param.default_value {
                        markdown.push_str("\n**Default Value**\n\n");
                        markdown.push_str(&format_code_block(&default_value.pretty_format()));
                    }

                    // Handle nested object properties recursively
                    if let BicepType::Object(Some(nested_props)) = &prop_param.parameter_type {
                        if !nested_props.is_empty() {
                            markdown.push_str("\n**Object Definition**\n\n");
                            generate_nested_object_properties(markdown, nested_props, 5, use_emoji);
                        }
                    }

                    if !prop_param.metadata.is_empty() {
                        markdown.push_str("\n**Metadata**\n\n");
                        generate_metadata_display_markdown(markdown, &prop_param.metadata);
                    }

                    markdown.push('\n');
                }
            }
        }

        markdown.push('\n');
    }
}

/// Generate the Functions section of the markdown
fn generate_functions_section(
    markdown: &mut String,
    document: &BicepDocument,
    use_emoji: bool,
    exclude_empty: bool,
) {
    markdown.push_str("## Functions\n\n");

    if document.functions.is_empty() {
        if !exclude_empty {
            markdown.push_str("*No functions defined*\n\n");
        }
        return;
    }

    for (name, function) in &document.functions {
        markdown.push_str(&format!("### `{}`\n\n", name));

        if let Some(description) = &function.description {
            markdown.push_str(&format!("{}\n\n", escape_markdown(description)));
        }

        // Basic information table
        let items = vec![
            ("Return Type", format!("`{}`", function.return_type)),
            ("Exported", format_yes_no(function.is_exported, use_emoji)),
        ];
        generate_key_value_display(markdown, &items);

        // Parameters
        if !function.arguments.is_empty() {
            markdown.push_str("\n**Parameters**\n\n");
            generate_function_arguments_display(markdown, &function.arguments);
        }

        // Definition
        if !function.expression.is_empty() {
            markdown.push_str("\n**Definition**\n\n");
            markdown.push_str(&format_code_block(&function.expression));
        }

        if !function.metadata.is_empty() {
            markdown.push_str("\n**Metadata**\n\n");
            generate_metadata_display_markdown(markdown, &function.metadata);
        }

        markdown.push('\n');
    }
}

/// Generate the Parameters section of the markdown
fn generate_parameters_section(
    markdown: &mut String,
    document: &BicepDocument,
    use_emoji: bool,
    exclude_empty: bool,
) {
    markdown.push_str("## Parameters\n\n");

    if document.parameters.is_empty() {
        if !exclude_empty {
            markdown.push_str("*No parameters defined*\n\n");
        }
        return;
    }

    for (name, parameter) in &document.parameters {
        markdown.push_str(&format!("### `{}`\n\n", name));

        if let Some(description) = &parameter.description {
            markdown.push_str(&format!("{}\n\n", escape_markdown(description)));
        }

        // Metadata comes first if present
        if !parameter.metadata.is_empty() {
            markdown.push_str("**Metadata**\n\n");
            generate_metadata_display_markdown(markdown, &parameter.metadata);
            markdown.push('\n');
        }

        // Basic information table
        let mut items = vec![(
            "Type",
            format_bicep_type_with_backticks(&parameter.parameter_type),
        )];

        items.push(("Nullable", format_yes_no(parameter.is_nullable, use_emoji)));

        items.push(("Secure", format_yes_no(parameter.is_secure, use_emoji)));

        items.push(("Sealed", format_yes_no(parameter.is_sealed, use_emoji)));

        generate_key_value_display(markdown, &items);

        // Handle constraints separately
        let mut constraints = Vec::new();
        if let Some(min_value) = parameter.min_value {
            constraints.push((
                "Minimum Value",
                format_constraint_value(&min_value.to_string()),
            ));
        }

        if let Some(max_value) = parameter.max_value {
            constraints.push((
                "Maximum Value",
                format_constraint_value(&max_value.to_string()),
            ));
        }

        if let Some(min_length) = parameter.min_length {
            constraints.push((
                "Minimum Length",
                format_constraint_value(&min_length.to_string()),
            ));
        }

        if let Some(max_length) = parameter.max_length {
            constraints.push((
                "Maximum Length",
                format_constraint_value(&max_length.to_string()),
            ));
        }

        if let Some(allowed_values) = &parameter.allowed_values {
            if !allowed_values.is_empty() {
                constraints.push(("Allowed Values", format_bicep_array_as_list(allowed_values)));
            }
        }

        if !constraints.is_empty() {
            markdown.push_str("\n**Constraints**\n\n");
            generate_key_value_display(markdown, &constraints);
        }

        if let Some(default_value) = &parameter.default_value {
            markdown.push_str("\n**Default Value**\n\n");
            markdown.push_str(&format_code_block(&default_value.pretty_format()));
        }

        // Object properties for object types
        if let BicepType::Object(Some(properties)) = &parameter.parameter_type {
            if !properties.is_empty() {
                markdown.push_str("\n**Object Definition**\n\n");
                generate_nested_object_properties(markdown, properties, 4, use_emoji);
            }
        }

        markdown.push('\n');
    }
}

/// Generate nested object properties recursively for Markdown
///
/// # Arguments
///
/// * `markdown` - The string buffer to append Markdown content to
/// * `properties` - The object properties to document
/// * `header_level` - The header level to use (4 for #### level, 5 for ##### level, etc.)
fn generate_nested_object_properties(
    markdown: &mut String,
    properties: &indexmap::IndexMap<String, crate::parsing::BicepParameter>,
    header_level: usize,
    use_emoji: bool,
) {
    let header_prefix = "#".repeat(header_level);

    for (prop_name, prop_param) in properties {
        markdown.push_str(&format!("{} `{}`\n\n", header_prefix, prop_name));

        if let Some(description) = &prop_param.description {
            markdown.push_str(&format!("{}\n\n", escape_markdown(description)));
        }

        let mut prop_items = vec![(
            "Type",
            format_bicep_type_with_backticks(&prop_param.parameter_type),
        )];

        prop_items.push(("Nullable", format_yes_no(prop_param.is_nullable, use_emoji)));
        prop_items.push(("Secure", format_yes_no(prop_param.is_secure, use_emoji)));

        generate_key_value_display(markdown, &prop_items);

        // Handle constraints separately
        let mut constraints = Vec::new();
        if let Some(min_value) = prop_param.min_value {
            constraints.push((
                "Minimum Value",
                format_constraint_value(&min_value.to_string()),
            ));
        }
        if let Some(max_value) = prop_param.max_value {
            constraints.push((
                "Maximum Value",
                format_constraint_value(&max_value.to_string()),
            ));
        }
        if let Some(min_length) = prop_param.min_length {
            constraints.push((
                "Minimum Length",
                format_constraint_value(&min_length.to_string()),
            ));
        }
        if let Some(max_length) = prop_param.max_length {
            constraints.push((
                "Maximum Length",
                format_constraint_value(&max_length.to_string()),
            ));
        }
        if let Some(allowed_values) = &prop_param.allowed_values {
            if !allowed_values.is_empty() {
                prop_items.push(("Allowed Values", format_bicep_array_as_list(allowed_values)));
            }
        }

        if !constraints.is_empty() {
            markdown.push_str("\n**Constraints**\n\n");
            generate_key_value_display(markdown, &constraints);
        }

        if let Some(default_value) = &prop_param.default_value {
            markdown.push_str("\n**Default Value**\n\n");
            markdown.push_str(&format_code_block(&default_value.pretty_format()));
        }

        // Recursively handle nested object properties (limit depth to avoid infinite recursion)
        if header_level < 7 {
            if let BicepType::Object(Some(nested_properties)) = &prop_param.parameter_type {
                if !nested_properties.is_empty() {
                    markdown.push_str("\n**Object Definition**\n\n");
                    generate_nested_object_properties(
                        markdown,
                        nested_properties,
                        header_level + 1,
                        use_emoji,
                    );
                }
            }
        }

        markdown.push('\n');
    }
}

/// Generate the Variables section of the markdown
fn generate_variables_section(
    markdown: &mut String,
    document: &BicepDocument,
    use_emoji: bool,
    exclude_empty: bool,
) {
    markdown.push_str("## Variables\n\n");

    if document.variables.is_empty() {
        if !exclude_empty {
            markdown.push_str("*No variables defined*\n\n");
        }
        return;
    }

    for (name, variable) in &document.variables {
        markdown.push_str(&format!("### `{}`\n\n", name));

        if let Some(description) = &variable.description {
            markdown.push_str(&format!("{}\n\n", escape_markdown(description)));
        }

        // Basic information table
        let items = vec![("Exported", format_yes_no(variable.is_exported, use_emoji))];
        generate_key_value_display(markdown, &items);

        // Value
        markdown.push_str("\n**Value**\n\n");
        markdown.push_str(&format_code_block(&variable.value.pretty_format()));

        markdown.push('\n');
    }
}

/// Generate the Resources section of the markdown
fn generate_resources_section(
    markdown: &mut String,
    document: &BicepDocument,
    use_emoji: bool,
    exclude_empty: bool,
) {
    markdown.push_str("## Resources\n\n");

    if document.resources.is_empty() {
        if !exclude_empty {
            markdown.push_str("*No resources defined*\n\n");
        }
        return;
    }

    for (name, resource) in &document.resources {
        markdown.push_str(&format!("### `{}`\n\n", name));

        if let Some(description) = &resource.description {
            markdown.push_str(&format!("{}\n\n", escape_markdown(description)));
        }

        // Basic information table
        let mut items = vec![
            ("Name", format!("`{}`", resource.name)),
            ("Type", format!("`{}`", resource.resource_type)),
            ("API Version", format!("`{}`", resource.api_version)),
        ];

        if let Some(scope) = &resource.scope {
            let scope_str = scope.to_string();
            items.push(("Scope", format!("`{}`", scope_str)));
        }

        if resource.existing {
            items.push(("Existing", format_yes_no(true, use_emoji)));
        }

        if let Some(parent) = &resource.parent {
            items.push(("Parent", format!("`{}`", parent.clone())));
        }

        if let Some(depends_on) = &resource.depends_on {
            if !depends_on.is_empty() {
                let deps = depends_on
                    .iter()
                    .map(|v| format!("`{}`", v))
                    .collect::<Vec<_>>()
                    .join("  \n");
                items.push(("Depends On", deps));
            }
        }

        if let Some(batch_size) = resource.batch_size {
            items.push(("Batch Size", format!("`{}`", batch_size)));
        }

        if let Some(condition) = &resource.condition {
            items.push(("Condition", format!("  \n{}", format_code_block(condition))));
        }

        if let Some(loop_statement) = &resource.loop_statement {
            items.push(("Loop", format!("  \n{}", format_code_block(loop_statement))));
        }

        generate_key_value_display(markdown, &items);

        markdown.push('\n');
    }
}

/// Generate the Modules section of the markdown
fn generate_modules_section(
    markdown: &mut String,
    document: &BicepDocument,
    _use_emoji: bool,
    exclude_empty: bool,
) {
    markdown.push_str("## Modules\n\n");

    if document.modules.is_empty() {
        if !exclude_empty {
            markdown.push_str("*No modules defined*\n\n");
        }
        return;
    }

    for (name, module) in &document.modules {
        markdown.push_str(&format!("### {}\n\n", name));

        if let Some(description) = &module.description {
            markdown.push_str(&format!("{}\n\n", escape_markdown(description)));
        }

        // Basic information table
        let mut items = vec![
            ("Source", format!(" `{}`", module.source)),
            ("Name", module.name.clone()),
        ];

        if let Some(depends_on) = &module.depends_on {
            if !depends_on.is_empty() {
                let deps = depends_on.join(", ");
                items.push(("Depends On", deps));
            }
        }

        if let Some(batch_size) = module.batch_size {
            items.push(("Batch Size", format!("`{}`", batch_size)));
        }

        if let Some(condition) = &module.condition {
            items.push(("Condition", format!("  \n{}", format_code_block(condition))));
        }

        if let Some(loop_statement) = &module.loop_statement {
            items.push(("Loop", format!("  \n{}", format_code_block(loop_statement))));
        }

        generate_key_value_display(markdown, &items);

        markdown.push('\n');
    }
}

/// Generate the Outputs section of the markdown
fn generate_outputs_section(
    markdown: &mut String,
    document: &BicepDocument,
    use_emoji: bool,
    exclude_empty: bool,
) {
    markdown.push_str("## Outputs\n\n");

    if document.outputs.is_empty() {
        if !exclude_empty {
            markdown.push_str("*No outputs defined*\n\n");
        }
        return;
    }

    for (name, output) in &document.outputs {
        markdown.push_str(&format!("### `{}`\n\n", name));

        if let Some(description) = &output.description {
            markdown.push_str(&format!("{}\n\n", escape_markdown(description)));
        }

        // Basic information table
        let mut items = vec![(
            "Type",
            format_bicep_type_with_backticks(&output.output_type),
        )];

        if let Some(discriminator) = &output.discriminator {
            items.push(("Discriminator", discriminator.clone()));
        }

        items.push(("Sealed", format_yes_no(output.sealed, use_emoji)));
        items.push(("Secure", format_yes_no(output.secure, use_emoji)));

        generate_key_value_display(markdown, &items);

        // Handle constraints separately
        let mut constraints = Vec::new();
        if let Some(min_length) = output.min_length {
            constraints.push((
                "Minimum Length",
                format_constraint_value(&min_length.to_string()),
            ));
        }
        if let Some(max_length) = output.max_length {
            constraints.push((
                "Maximum Length",
                format_constraint_value(&max_length.to_string()),
            ));
        }
        if let Some(min_value) = output.min_value {
            constraints.push((
                "Minimum Value",
                format_constraint_value(&min_value.to_string()),
            ));
        }
        if let Some(max_value) = output.max_value {
            constraints.push((
                "Maximum Value",
                format_constraint_value(&max_value.to_string()),
            ));
        }

        if !constraints.is_empty() {
            markdown.push_str("\n**Constraints**\n\n");
            generate_key_value_display(markdown, &constraints);
        }

        // Value in code block
        markdown.push_str("\n**Value**\n\n");
        markdown.push_str(&format_code_block(&output.value.pretty_format()));

        if let Some(metadata) = &output.metadata {
            if !metadata.is_empty() {
                markdown.push_str("\n**Metadata**\n\n");
                generate_metadata_display_markdown(markdown, metadata);
            }
        }

        markdown.push('\n');
    }
}

/// Format a constraint value with backticks for display in markdown
fn format_constraint_value(value: &str) -> String {
    format!("`{}`", value)
}

/// Format a value as acode block for display in Markdown
fn format_code_block(value: &str) -> String {
    format!("```bicep\n{}\n```\n", value)
}

/// Generate key-value property display
fn generate_key_value_display(markdown: &mut String, items: &[(&str, String)]) {
    for (key, value) in items {
        markdown.push_str(&format!("**{}:** {}  \n", key, value));
    }
}

/// Generate display for function arguments
///
/// # Arguments
///
/// * `markdown` - The string buffer to append markdown content to
/// * `arguments` - The function arguments to display
fn generate_function_arguments_display(markdown: &mut String, arguments: &[BicepFunctionArgument]) {
    for arg in arguments {
        let optional_text = if arg.is_nullable { " (Optional)" } else { "" };
        markdown.push_str(&format!(
            "**{}:** {}{}\n",
            &arg.name,
            format_bicep_type_with_backticks(&arg.argument_type),
            optional_text
        ));
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

        let result = export_to_string(&document, true, false);
        assert!(result.is_ok());

        let markdown = result.unwrap();
        assert!(markdown.contains("# Test Template"));
        assert!(markdown.contains("A test template for unit testing"));
        assert!(markdown.contains("resourceGroup"));

        // When exclude_empty is false, empty sections should be present
        assert!(markdown.contains("## Parameters"));
        assert!(markdown.contains("*No parameters defined*"));
        assert!(markdown.contains("## Resources"));
        assert!(markdown.contains("*No resources defined*"));
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

        let result = export_to_string(&document, true, false);
        assert!(result.is_ok());

        let markdown = result.unwrap();
        assert!(markdown.contains("## Parameters"));
        assert!(markdown.contains("### `testParam`"));
        assert!(markdown.contains("Test parameter"));
        assert!(markdown.contains("default"));
    }

    #[test]
    fn test_export_to_string_with_exclude_empty() {
        // Create a document with some empty collections and one non-empty collection
        let mut document = BicepDocument {
            name: Some("Test Template".to_string()),
            description: Some("A test template".to_string()),
            ..Default::default()
        };

        // Add one parameter to make that collection non-empty
        let parameter = BicepParameter {
            parameter_type: BicepType::String,
            description: Some("Test parameter".to_string()),
            ..Default::default()
        };
        document
            .parameters
            .insert("testParam".to_string(), parameter);

        // Test with exclude_empty = true
        let result = export_to_string(&document, true, true).unwrap();

        // Should contain the document name and the parameter section
        assert!(result.contains("# Test Template"));
        assert!(result.contains("## Parameters"));
        assert!(result.contains("### `testParam`"));

        // Should NOT contain empty sections
        assert!(!result.contains("## Resources"));
        assert!(!result.contains("*No resources defined*"));
        assert!(!result.contains("## Variables"));
        assert!(!result.contains("*No variables defined*"));
        assert!(!result.contains("## Modules"));
        assert!(!result.contains("*No modules defined*"));
        assert!(!result.contains("## Outputs"));
        assert!(!result.contains("*No outputs defined*"));
    }

    #[test]
    fn test_format_bicep_value() {
        // Test basic values with default list format
        assert_eq!(BicepValue::String("test".to_string()).to_string(), "test");
        assert_eq!(BicepValue::Int(42).to_string(), "42");
        assert_eq!(BicepValue::Bool(true).to_string(), "true");
        assert_eq!(
            BicepValue::Identifier("myVar".to_string()).to_string(),
            "${myVar}"
        );
        assert_eq!(
            BicepValue::String("line1\nline2".to_string()).to_string(),
            "line1\nline2"
        );
        assert_eq!(
            BicepValue::String("line1\nline2".to_string()).to_string(),
            "line1\nline2"
        );
        assert_eq!(
            BicepValue::String("Has\\backslash".to_string()).to_string(),
            "Has\\backslash"
        );
        assert_eq!(
            BicepValue::String("Has\\backslash".to_string()).to_string(),
            "Has\\backslash"
        );
    }

    #[test]
    fn test_format_bicep_type() {
        assert_eq!(BicepType::String.to_string(), "string");
        assert_eq!(BicepType::Int.to_string(), "int");
        assert_eq!(BicepType::Bool.to_string(), "bool");
        assert_eq!(
            BicepType::Array(Box::new(BicepType::String)).to_string(),
            "string[]"
        );
        assert_eq!(
            BicepType::CustomType("MyType".to_string()).to_string(),
            "MyType"
        );
        assert_eq!(
            BicepType::Union(vec!["A".to_string(), "B".to_string()]).to_string(),
            "A | B"
        );

        // Test Object types
        assert_eq!(BicepType::Object(None).to_string(), "object");

        // Test empty object with properties
        use indexmap::IndexMap;
        let empty_props = IndexMap::new();
        assert_eq!(BicepType::Object(Some(empty_props)).to_string(), "object");

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
        assert_eq!(BicepType::Object(Some(props)).to_string(), "object");
    }
    #[test]
    fn test_format_bicep_value_object() {
        use indexmap::IndexMap;

        // Test empty object
        let empty_obj = IndexMap::new();
        assert_eq!(BicepValue::Object(empty_obj).to_string(), "{}");

        // Test object with properties
        let mut obj = IndexMap::new();
        obj.insert("key1".to_string(), BicepValue::String("value1".to_string()));
        obj.insert("key2".to_string(), BicepValue::Int(42));
        assert_eq!(
            BicepValue::Object(obj).to_string(),
            "{ key1: value1, key2: 42 }"
        );

        // Test object with multiline string in list format
        let mut obj_ml = IndexMap::new();
        obj_ml.insert(
            "text".to_string(),
            BicepValue::String("line1\nline2".to_string()),
        );

        // List format should preserve newlines
        assert_eq!(
            BicepValue::Object(obj_ml.clone()).to_string(),
            "{ text: line1\nline2 }"
        );

        // List format should preserve newlines
        assert_eq!(
            BicepValue::Object(obj_ml).to_string(),
            "{ text: line1\nline2 }"
        );
    }

    #[test]
    fn test_format_bicep_type_union_formats() {
        // Test that union types are formatted for list format
        let union_type = BicepType::Union(vec!["string".to_string(), "int".to_string()]);

        // List format should not escape | characters
        assert_eq!(union_type.to_string(), "string | int");
    }
}
