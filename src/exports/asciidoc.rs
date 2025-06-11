/// AsciiDoc export functionality for Bicep documents
///
/// This module provides functions to export parsed Bicep documents
/// to AsciiDoc format with structured documentation layout.
use std::error::Error as StdError;
use std::{fs, path::Path};

use crate::{
    exports::utils::{
        common::{format_yes_no, generate_metadata_display_asciidoc},
        formatting::escape_asciidoc,
    },
    parsing::{BicepDocument, BicepFunctionArgument, BicepImport, BicepType, ModuleSource},
};

/// Export a Bicep document to an AsciiDoc file
///
/// # Arguments
///
/// * `document` - The BicepDocument to export
/// * `file_path` - Path where the AsciiDoc file should be written
/// * `use_emoji` - Whether to use emoji symbols (✅/❌) for Yes/No values
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
    let asciidoc_content = export_to_string(document, use_emoji, exclude_empty)?;
    fs::write(file_path, asciidoc_content)?;
    Ok(())
}

/// Export a Bicep document to an AsciiDoc string
///
/// # Arguments
///
/// * `document` - The BicepDocument to export
/// * `use_emoji` - Whether to use emoji symbols (✅/❌) for Yes/No values
/// * `exclude_empty` - Whether to exclude empty sections from the output
///
/// # Returns
///
/// Result containing the AsciiDoc string representation of the document
///
/// # Errors
///
/// Returns an error if serialization fails
pub fn export_to_string(
    document: &BicepDocument,
    use_emoji: bool,
    exclude_empty: bool,
) -> Result<String, Box<dyn StdError>> {
    let mut asciidoc = String::new();

    // Title and document attributes
    if let Some(name) = &document.name {
        asciidoc.push_str(&format!("= {}\n", name));
    } else {
        asciidoc.push_str("= Bicep Template\n");
    }

    // Document attributes
    asciidoc.push_str(":noheader:\n");
    asciidoc.push_str(":source-language: bicep\n");
    asciidoc.push_str(":table-caption!:\n");
    asciidoc.push_str(":toc: preamble\n");
    asciidoc.push_str(":toclevels: 2\n\n");

    // Description
    if let Some(description) = &document.description {
        asciidoc.push_str(&format!("{}\n\n", description));
    }

    // Target scope in table format
    if let Some(target_scope) = &document.target_scope {
        asciidoc.push_str("[%autowidth,cols=\"h,1\",frame=none]\n");
        asciidoc.push_str("|===\n");
        asciidoc.push_str("| Target Scope\n");
        asciidoc.push_str(&format!("| {}\n", escape_asciidoc(target_scope)));
        asciidoc.push_str("|===\n\n");
    }

    // Additional metadata
    if !document.metadata.is_empty() {
        asciidoc.push_str(".Additional Metadata\n");
        asciidoc.push_str("[%autowidth,cols=\"h,1\",frame=none]\n");
        generate_metadata_display_asciidoc(&mut asciidoc, &document.metadata);
    }

    asciidoc.push('\n');

    // Imports section
    if !document.imports.is_empty() || !exclude_empty {
        asciidoc.push_str("== Imports\n\n");
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
                asciidoc.push('\n');
            }
        } else if !exclude_empty {
            asciidoc.push_str("_No imports defined_\n\n");
        }
    }

    // Types section
    if !document.types.is_empty() || !exclude_empty {
        generate_types_section(&mut asciidoc, document, use_emoji, exclude_empty);
    }

    // Functions section
    if !document.functions.is_empty() || !exclude_empty {
        generate_functions_section(&mut asciidoc, document, use_emoji, exclude_empty);
    }

    // Parameters section
    if !document.parameters.is_empty() || !exclude_empty {
        generate_parameters_section(&mut asciidoc, document, use_emoji, exclude_empty);
    }

    // Variables section
    if !document.variables.is_empty() || !exclude_empty {
        generate_variables_section(&mut asciidoc, document, use_emoji, exclude_empty);
    }

    // Resources section
    if !document.resources.is_empty() || !exclude_empty {
        generate_resources_section(&mut asciidoc, document, use_emoji, exclude_empty);
    }

    // Modules section
    if !document.modules.is_empty() || !exclude_empty {
        generate_modules_section(&mut asciidoc, document, exclude_empty);
    }

    // Outputs section
    if !document.outputs.is_empty() || !exclude_empty {
        generate_outputs_section(&mut asciidoc, document, use_emoji, exclude_empty);
    }

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

/// Generate the Types section of the AsciiDoc
fn generate_types_section(
    asciidoc: &mut String,
    document: &BicepDocument,
    use_emoji: bool,
    exclude_empty: bool,
) {
    asciidoc.push_str("== Types\n\n");

    if document.types.is_empty() {
        if !exclude_empty {
            asciidoc.push_str("_No custom types defined_\n\n");
        }
        return;
    }

    for (name, custom_type) in &document.types {
        asciidoc.push_str(&format!("=== `{}`\n\n", escape_asciidoc(name)));

        if let Some(description) = &custom_type.description {
            asciidoc.push_str(&format!("{}\n\n", escape_asciidoc(description)));
        }

        // Basic information table with properties label
        asciidoc.push_str(".Properties\n");
        let items = vec![
            ("Type", format!("m| {}", custom_type.definition)),
            (
                "Exported",
                format_yes_no(custom_type.is_exported, use_emoji),
            ),
            (
                "Nullable",
                format_yes_no(false, use_emoji), // Types themselves are not nullable
            ),
            ("Secure", format_yes_no(custom_type.is_secure, use_emoji)),
        ];

        generate_key_value_display(asciidoc, &items, "h,1");

        // Check if this is an object type with properties and add object properties section
        if let BicepType::Object(Some(properties)) = &custom_type.definition {
            if !properties.is_empty() {
                asciidoc.push_str("\n*Object Definition*\n\n");

                for (prop_name, prop_param) in properties {
                    asciidoc.push_str(&format!("==== `{}`\n\n", escape_asciidoc(prop_name)));

                    if let Some(description) = &prop_param.description {
                        asciidoc.push_str(&format!("{}\n\n", escape_asciidoc(description)));
                    }

                    asciidoc.push_str(".Properties\n");
                    let prop_items = vec![
                        ("Type", format!("m| {}", prop_param.parameter_type)),
                        (
                            "Nullable",
                            if prop_param.is_nullable {
                                "✅ Yes".to_string()
                            } else {
                                "❌ No".to_string()
                            },
                        ),
                        (
                            "Secure",
                            if prop_param.is_secure {
                                "✅ Yes".to_string()
                            } else {
                                "❌ No".to_string()
                            },
                        ),
                    ];

                    generate_key_value_display(asciidoc, &prop_items, "h,1");

                    // Add constraints section if there are any constraints
                    let mut constraints = Vec::new();
                    if let Some(min_value) = prop_param.min_value {
                        constraints.push(("Minimum Value", min_value.to_string()));
                    }
                    if let Some(max_value) = prop_param.max_value {
                        constraints.push(("Maximum Value", max_value.to_string()));
                    }
                    if let Some(min_length) = prop_param.min_length {
                        constraints.push(("Minimum Length", min_length.to_string()));
                    }
                    if let Some(max_length) = prop_param.max_length {
                        constraints.push(("Maximum Length", max_length.to_string()));
                    }
                    if let Some(allowed_values) = &prop_param.allowed_values {
                        if !allowed_values.is_empty() {
                            let values = allowed_values
                                .iter()
                                .map(|v| v.to_string())
                                .collect::<Vec<_>>()
                                .join(", ");
                            constraints.push(("Allowed Values", values));
                        }
                    }

                    if !constraints.is_empty() {
                        asciidoc.push_str("\n.Constraints\n");
                        generate_key_value_display(asciidoc, &constraints, "h,>m");
                    }

                    // Handle nested object properties recursively
                    if let BicepType::Object(Some(nested_props)) = &prop_param.parameter_type {
                        if !nested_props.is_empty() {
                            generate_nested_object_properties(asciidoc, nested_props, 5, use_emoji);
                        }
                    }

                    if let Some(default_value) = &prop_param.default_value {
                        asciidoc.push_str("\n.Default Value\n");
                        asciidoc.push_str("[source]\n");
                        asciidoc.push_str("----\n");
                        asciidoc.push_str(&default_value.to_string());
                        asciidoc.push_str("\n----\n");
                    }

                    if !prop_param.metadata.is_empty() {
                        asciidoc.push_str("\n.Metadata\n");
                        asciidoc.push_str("[%autowidth,cols=\"h,1\",frame=none]\n");
                        generate_metadata_display_asciidoc(asciidoc, &prop_param.metadata);
                    }

                    asciidoc.push('\n');
                }
            }
        }

        asciidoc.push('\n');
    }
}

/// Generate the Functions section of the AsciiDoc
fn generate_functions_section(
    asciidoc: &mut String,
    document: &BicepDocument,
    use_emoji: bool,
    exclude_empty: bool,
) {
    asciidoc.push_str("== Functions\n\n");

    if document.functions.is_empty() {
        if !exclude_empty {
            asciidoc.push_str("_No user-defined functions_\n\n");
        }
        return;
    }

    for (name, function) in &document.functions {
        asciidoc.push_str(&format!("=== `{}`\n\n", escape_asciidoc(name)));

        if let Some(description) = &function.description {
            asciidoc.push_str(&format!("{}\n\n", escape_asciidoc(description)));
        }

        // Basic information table
        asciidoc.push_str(".Properties\n");
        let items = vec![
            ("Return Type", format!("m| {}", function.return_type)),
            ("Exported", format_yes_no(function.is_exported, use_emoji)),
        ];
        generate_key_value_display(asciidoc, &items, "h,1");

        // Parameters
        if !function.arguments.is_empty() {
            asciidoc.push_str("\n.Parameters\n");
            generate_function_arguments_display(asciidoc, &function.arguments, use_emoji);
        }

        // Function definition
        asciidoc.push_str("\n.Definition\n");
        asciidoc.push_str("[source]\n");
        asciidoc.push_str("----\n");
        asciidoc.push_str(&escape_asciidoc(&function.expression));
        asciidoc.push_str("\n----\n");

        asciidoc.push('\n');
    }
}

/// Generate the Parameters section of the AsciiDoc
fn generate_parameters_section(
    asciidoc: &mut String,
    document: &BicepDocument,
    use_emoji: bool,
    exclude_empty: bool,
) {
    asciidoc.push_str("== Parameters\n\n");

    if document.parameters.is_empty() {
        if !exclude_empty {
            asciidoc.push_str("_No parameters defined_\n\n");
        }
        return;
    }

    for (name, parameter) in &document.parameters {
        asciidoc.push_str(&format!("=== `{}`\n\n", escape_asciidoc(name)));

        if let Some(description) = &parameter.description {
            asciidoc.push_str(&format!("{}\n\n", escape_asciidoc(description)));
        }

        // Handle metadata at the top if it contains description
        if !parameter.metadata.is_empty() {
            // Check if metadata has description that should be shown as the main description
            if let Some(metadata_desc) = parameter.metadata.get("description") {
                if parameter.description.is_none() {
                    asciidoc.push_str(&format!(
                        "{}\n\n",
                        escape_asciidoc(&metadata_desc.to_string())
                    ));
                }
            }

            // Show other metadata
            let mut other_metadata = parameter.metadata.clone();
            other_metadata.shift_remove("description");
            if !other_metadata.is_empty() {
                asciidoc.push_str(".Metadata\n");
                asciidoc.push_str("[%autowidth,cols=\"h,1\",frame=none]\n");
                generate_metadata_display_asciidoc(asciidoc, &other_metadata);
                asciidoc.push('\n');
            }
        }

        // Basic information table
        asciidoc.push_str(".Properties\n");
        let items = vec![
            ("Type", format!("m| {}", parameter.parameter_type)),
            ("Nullable", format_yes_no(parameter.is_nullable, use_emoji)),
            ("Secure", format_yes_no(parameter.is_secure, use_emoji)),
            ("Sealed", format_yes_no(parameter.is_sealed, use_emoji)),
        ];

        generate_key_value_display(asciidoc, &items, "h,1");

        // Add constraints section if there are any constraints
        let mut constraints = Vec::new();
        if let Some(min_value) = parameter.min_value {
            constraints.push(("Minimum Value", min_value.to_string()));
        }
        if let Some(max_value) = parameter.max_value {
            constraints.push(("Maximum Value", max_value.to_string()));
        }
        if let Some(min_length) = parameter.min_length {
            constraints.push(("Minimum Length", min_length.to_string()));
        }
        if let Some(max_length) = parameter.max_length {
            constraints.push(("Maximum Length", max_length.to_string()));
        }
        if let Some(allowed_values) = &parameter.allowed_values {
            if !allowed_values.is_empty() {
                let values = allowed_values
                    .iter()
                    .map(|v| format!("`{}`", v.to_string()))
                    .collect::<Vec<_>>()
                    .join(" +\n   ");
                constraints.push(("Allowed Values", format!("<| {}", values)));
            }
        }

        if !constraints.is_empty() {
            asciidoc.push_str("\n.Constraints\n");
            generate_key_value_display(asciidoc, &constraints, "h,>m");
        }

        // Default value
        if let Some(default_value) = &parameter.default_value {
            asciidoc.push_str("\n.Default Value\n");
            asciidoc.push_str("[source]\n");
            asciidoc.push_str("----\n");
            asciidoc.push_str(&default_value.to_string());
            asciidoc.push_str("\n----\n");
        }

        // Object definition for object types
        if let BicepType::Object(Some(properties)) = &parameter.parameter_type {
            if !properties.is_empty() {
                asciidoc.push_str("\n*Object Definition*\n\n");

                for (prop_name, prop_param) in properties {
                    asciidoc.push_str(&format!("==== `{}`\n\n", escape_asciidoc(prop_name)));

                    if let Some(description) = &prop_param.description {
                        asciidoc.push_str(&format!("{}\n\n", escape_asciidoc(description)));
                    }

                    asciidoc.push_str(".Properties\n");
                    let prop_items = vec![
                        ("Type", format!("m| {}", prop_param.parameter_type)),
                        ("Nullable", format_yes_no(prop_param.is_nullable, use_emoji)),
                        ("Secure", format_yes_no(prop_param.is_secure, use_emoji)),
                    ];

                    generate_key_value_display(asciidoc, &prop_items, "h,1");

                    // Add constraints for properties
                    let mut prop_constraints = Vec::new();
                    if let Some(min_value) = prop_param.min_value {
                        prop_constraints.push(("Minimum Value", min_value.to_string()));
                    }
                    if let Some(max_value) = prop_param.max_value {
                        prop_constraints.push(("Maximum Value", max_value.to_string()));
                    }
                    if let Some(min_length) = prop_param.min_length {
                        prop_constraints.push(("Minimum Length", min_length.to_string()));
                    }
                    if let Some(max_length) = prop_param.max_length {
                        prop_constraints.push(("Maximum Length", max_length.to_string()));
                    }

                    if !prop_constraints.is_empty() {
                        asciidoc.push_str("\n.Constraints\n");
                        generate_key_value_display(asciidoc, &prop_constraints, "h,>m");
                    }

                    // Recursively handle nested object properties
                    if let BicepType::Object(Some(nested_properties)) = &prop_param.parameter_type {
                        if !nested_properties.is_empty() {
                            asciidoc.push_str("\n*Object Definition*\n\n");
                            generate_nested_object_properties(
                                asciidoc,
                                nested_properties,
                                5,
                                use_emoji,
                            );
                        }
                    }

                    asciidoc.push('\n');
                }
            }
        }

        asciidoc.push('\n');
    }
}

/// Generate nested object properties recursively for AsciiDoc
///
/// # Arguments
///
/// * `asciidoc` - The string buffer to append AsciiDoc content to
/// * `properties` - The object properties to document
/// * `header_level` - The header level to use (4 for ==== level, 5 for ===== level, etc.)
fn generate_nested_object_properties(
    asciidoc: &mut String,
    properties: &indexmap::IndexMap<String, crate::parsing::BicepParameter>,
    header_level: usize,
    use_emoji: bool,
) {
    let header_prefix = "=".repeat(header_level);

    for (prop_name, prop_param) in properties {
        asciidoc.push_str(&format!(
            "{} `{}`\n\n",
            header_prefix,
            escape_asciidoc(prop_name)
        ));

        if let Some(description) = &prop_param.description {
            asciidoc.push_str(&format!("{}\n\n", escape_asciidoc(description)));
        }

        asciidoc.push_str(".Properties\n");
        let prop_items = vec![
            ("Type", format!("m| {}", prop_param.parameter_type)),
            ("Nullable", format_yes_no(prop_param.is_nullable, use_emoji)),
            ("Secure", format_yes_no(prop_param.is_secure, use_emoji)),
        ];

        generate_key_value_display(asciidoc, &prop_items, "h,1");

        // Add constraints for properties
        let mut prop_constraints = Vec::new();
        if let Some(min_value) = prop_param.min_value {
            prop_constraints.push(("Minimum Value", min_value.to_string()));
        }
        if let Some(max_value) = prop_param.max_value {
            prop_constraints.push(("Maximum Value", max_value.to_string()));
        }
        if let Some(min_length) = prop_param.min_length {
            prop_constraints.push(("Minimum Length", min_length.to_string()));
        }
        if let Some(max_length) = prop_param.max_length {
            prop_constraints.push(("Maximum Length", max_length.to_string()));
        }

        if !prop_constraints.is_empty() {
            asciidoc.push_str("\n.Constraints\n");
            generate_key_value_display(asciidoc, &prop_constraints, "h,>m");
        }

        // Recursively handle nested object properties (limit depth to avoid infinite recursion)
        if header_level < 7 {
            if let BicepType::Object(Some(nested_properties)) = &prop_param.parameter_type {
                if !nested_properties.is_empty() {
                    asciidoc.push_str("\n*Object Definition*\n\n");
                    generate_nested_object_properties(
                        asciidoc,
                        nested_properties,
                        header_level + 1,
                        use_emoji,
                    );
                }
            }
        }

        asciidoc.push('\n');
    }
}

/// Generate the Variables section of the AsciiDoc
fn generate_variables_section(
    asciidoc: &mut String,
    document: &BicepDocument,
    use_emoji: bool,
    exclude_empty: bool,
) {
    asciidoc.push_str("== Variables\n\n");

    if document.variables.is_empty() {
        if !exclude_empty {
            asciidoc.push_str("_No variables defined_\n\n");
        }
        return;
    }

    for (name, variable) in &document.variables {
        asciidoc.push_str(&format!("=== `{}`\n\n", escape_asciidoc(name)));

        if let Some(description) = &variable.description {
            asciidoc.push_str(&format!("{}\n\n", escape_asciidoc(description)));
        }

        // Basic information table
        asciidoc.push_str(".Properties\n");
        let items = vec![("Exported", format_yes_no(variable.is_exported, use_emoji))];
        generate_key_value_display(asciidoc, &items, "h,1");

        // Value section
        asciidoc.push_str("\n.Value\n");
        asciidoc.push_str("[source]\n");
        asciidoc.push_str("----\n");
        asciidoc.push_str(&variable.value.to_string());
        asciidoc.push_str("\n----\n");

        asciidoc.push('\n');
    }
}

/// Generate the Resources section of the AsciiDoc
fn generate_resources_section(
    asciidoc: &mut String,
    document: &BicepDocument,
    use_emoji: bool,
    exclude_empty: bool,
) {
    asciidoc.push_str("== Resources\n\n");

    if document.resources.is_empty() {
        if !exclude_empty {
            asciidoc.push_str("_No resources defined_\n\n");
        }
        return;
    }

    for (name, resource) in &document.resources {
        asciidoc.push_str(&format!("=== `{}`\n\n", escape_asciidoc(name)));

        if let Some(description) = &resource.description {
            asciidoc.push_str(&format!("{}\n\n", escape_asciidoc(description)));
        }

        // Basic information table
        asciidoc.push_str(".Properties\n");
        let mut items = vec![
            ("Name", escape_asciidoc(&resource.name)),
            ("Type", resource.resource_type.clone()),
            ("API Version", resource.api_version.clone()),
        ];

        if let Some(scope) = &resource.scope {
            let scope_str = scope.to_string();
            items.push(("Scope", scope_str));
        }

        if resource.existing {
            items.push(("Existing", format!("d| {}", format_yes_no(true, use_emoji))));
        }

        if let Some(parent) = &resource.parent {
            items.push(("Parent", parent.to_string()));
        }

        if let Some(depends_on) = &resource.depends_on {
            if !depends_on.is_empty() {
                let deps = depends_on.join(" +\n");
                items.push(("Depends On", deps));
            }
        }

        if let Some(batch_size) = resource.batch_size {
            items.push(("Batch Size", batch_size.to_string()));
        }

        generate_key_value_display(asciidoc, &items, "h,m");

        // Condition section
        if let Some(condition) = &resource.condition {
            asciidoc.push_str("\n.Condition\n");
            asciidoc.push_str("[source]\n");
            asciidoc.push_str("----\n");
            asciidoc.push_str(condition);
            asciidoc.push_str("\n----\n");
        }

        // Loop section
        if let Some(loop_statement) = &resource.loop_statement {
            asciidoc.push_str("\n.Loop\n");
            asciidoc.push_str("[source]\n");
            asciidoc.push_str("----\n");
            asciidoc.push_str(loop_statement);
            asciidoc.push_str("\n----\n");
        }

        asciidoc.push('\n');
    }
}

/// Generate the Modules section of the AsciiDoc
fn generate_modules_section(asciidoc: &mut String, document: &BicepDocument, exclude_empty: bool) {
    asciidoc.push_str("== Modules\n\n");

    if document.modules.is_empty() {
        if !exclude_empty {
            asciidoc.push_str("_No modules defined_\n\n");
        }
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
        generate_key_value_display(asciidoc, &items, "h,1");

        asciidoc.push('\n');
    }
}

/// Generate the Outputs section of the AsciiDoc
fn generate_outputs_section(
    asciidoc: &mut String,
    document: &BicepDocument,
    use_emoji: bool,
    exclude_empty: bool,
) {
    asciidoc.push_str("== Outputs\n\n");

    if document.outputs.is_empty() {
        if !exclude_empty {
            asciidoc.push_str("_No outputs defined_\n\n");
        }
        return;
    }

    for (name, output) in &document.outputs {
        asciidoc.push_str(&format!("=== `{}`\n\n", escape_asciidoc(name)));

        if let Some(description) = &output.description {
            asciidoc.push_str(&format!("{}\n\n", escape_asciidoc(description)));
        }

        // Basic information table
        asciidoc.push_str(".Properties\n");
        let mut items = vec![
            ("Type", format!("m| {}", output.output_type)),
            ("Secure", format_yes_no(output.secure, use_emoji)),
        ];

        if output.sealed {
            items.push(("Sealed", format_yes_no(true, use_emoji)));
        }

        if let Some(discriminator) = &output.discriminator {
            items.push(("Discriminator", discriminator.clone()));
        }

        generate_key_value_display(asciidoc, &items, "h,1");

        let mut prop_constraints = Vec::new();
        if let Some(min_value) = output.min_value {
            prop_constraints.push(("Minimum Value", min_value.to_string()));
        }
        if let Some(max_value) = output.max_value {
            prop_constraints.push(("Maximum Value", max_value.to_string()));
        }
        if let Some(min_length) = output.min_length {
            prop_constraints.push(("Minimum Length", min_length.to_string()));
        }
        if let Some(max_length) = output.max_length {
            prop_constraints.push(("Maximum Length", max_length.to_string()));
        }

        if !prop_constraints.is_empty() {
            asciidoc.push_str("\n.Constraints\n");
            generate_key_value_display(asciidoc, &prop_constraints, "h,>m");
        }

        // Value section
        asciidoc.push_str("\n.Value\n");
        asciidoc.push_str("[source]\n");
        asciidoc.push_str("----\n");
        asciidoc.push_str(&output.value.to_string());
        asciidoc.push_str("\n----\n");

        // Additional metadata if present
        if let Some(metadata) = &output.metadata {
            if !metadata.is_empty() {
                asciidoc.push_str("\n.Metadata\n");
                asciidoc.push_str("[%autowidth,cols=\"h,1\",frame=none]\n");
                generate_metadata_display_asciidoc(asciidoc, metadata);
            }
        }

        asciidoc.push('\n');
    }
}

/// Generate key-value property display
fn generate_key_value_display(asciidoc: &mut String, items: &[(&str, String)], cols: &str) {
    asciidoc.push_str(&format!("[%autowidth,cols=\"{}\",frame=none]\n", cols));
    asciidoc.push_str("|===\n");
    for (key, value) in items {
        match value.split_once("|") {
            Some((attr, split_value)) if !attr.ends_with('\\') => {
                // We have an attribute.
                // If statement catches escaped pipes in the value
                asciidoc.push_str(&format!(
                    "| {}\n{}| {}\n\n",
                    escape_asciidoc(key),
                    attr,
                    escape_asciidoc(split_value.trim())
                ));
            },
            _ => {
                // Otherwise, just display the key and value
                asciidoc.push_str(&format!(
                    "| {}\n| {}\n\n",
                    escape_asciidoc(key),
                    escape_asciidoc(value)
                ));
            },
        }
    }
    asciidoc.push_str("|===\n");
}

/// Generate display for function arguments in table format
///
/// # Arguments
///
/// * `asciidoc` - The string buffer to append AsciiDoc content to
/// * `arguments` - The function arguments to display
fn generate_function_arguments_display(
    asciidoc: &mut String,
    arguments: &[BicepFunctionArgument],
    use_emoji: bool,
) {
    asciidoc.push_str("[%autowidth,cols=\"h,m,1\",frame=none]\n");
    asciidoc.push_str("|===\n");
    asciidoc.push_str("| Name\n| Type\n| Required\n\n");
    for arg in arguments {
        asciidoc.push_str(&format!(
            "| {}\n| {}\n| {}\n\n",
            escape_asciidoc(&arg.name),
            escape_asciidoc(&arg.argument_type.to_string()),
            format_yes_no(!arg.is_nullable, use_emoji)
        ));
    }
    asciidoc.push_str("|===\n");
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

        let asciidoc = result.unwrap();
        assert!(asciidoc.contains("= Test Template"));
        assert!(asciidoc.contains("A test template for unit testing"));
        assert!(asciidoc.contains("resourceGroup"));

        // When exclude_empty is false, empty sections should be present
        assert!(asciidoc.contains("== Parameters"));
        assert!(asciidoc.contains("_No parameters defined_"));
        assert!(asciidoc.contains("== Resources"));
        assert!(asciidoc.contains("_No resources defined_"));
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

        let asciidoc = result.unwrap();
        assert!(asciidoc.contains("== Parameters"));
        assert!(asciidoc.contains("=== `testParam`"));
        assert!(asciidoc.contains("Test parameter"));
        assert!(asciidoc.contains("default"));
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
        assert!(result.contains("= Test Template"));
        assert!(result.contains("== Parameters"));
        assert!(result.contains("=== `testParam`"));

        // Should NOT contain empty sections
        assert!(!result.contains("== Resources"));
        assert!(!result.contains("_No resources defined_"));
        assert!(!result.contains("== Variables"));
        assert!(!result.contains("_No variables defined_"));
        assert!(!result.contains("== Modules"));
        assert!(!result.contains("_No modules defined_"));
        assert!(!result.contains("== Outputs"));
        assert!(!result.contains("_No outputs defined_"));
    }

    #[test]
    fn test_escape_asciidoc() {
        let text = "test | with * special _ characters [and] `code` #heading";
        let escaped = escape_asciidoc(text);
        assert_eq!(
            escaped,
            "test \\| with \\* special \\_ characters \\[and\\] \\`code\\` \\#heading"
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
    }

    #[test]
    fn test_format_bicep_value_object() {
        use indexmap::IndexMap;
        let mut obj = IndexMap::new();
        obj.insert("key1".to_string(), BicepValue::String("value1".to_string()));
        obj.insert("key2".to_string(), BicepValue::Int(42));

        let result = BicepValue::Object(obj).to_string();
        assert!(result.contains("key1: value1"));
        assert!(result.contains("key2: 42"));
    }

    #[test]
    fn test_format_bicep_type_union_formats() {
        let union_type = BicepType::Union(vec!["A".to_string(), "B".to_string()]);

        // Test format (now uses unified format same as Markdown)
        assert_eq!(
            crate::exports::utils::formatting::format_bicep_type_asciidoc(&union_type),
            "A \\| B"
        );
    }

    #[test]
    fn test_format_bicep_value_with_multiline_string() {
        // Test multiline string
        let multiline = BicepValue::String("line1\nline2\nline3".to_string());
        let result = multiline.to_string();
        assert_eq!(result, "line1\nline2\nline3");
    }
}
