//! Import declaration parsing for Bicep files.
//!
//! This module handles the parsing of import statements in Bicep files.
//! Imports allow templates to reference external modules, namespaces, and symbols,
//! enabling code reuse and modular template design.

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::error::Error;
use tracing::debug;
use tree_sitter::Node;

use super::{get_node_text, BicepParserError, ModuleSource};

// ---------------------------------------------------------------
// Structs, Enums & Types
// ---------------------------------------------------------------

/// Represents an import declaration in a Bicep file.
///
/// Bicep supports two main types of imports:
/// 1. Namespace imports for built-in or extension namespaces (e.g., `import 'az@1.0.0'`)
/// 2. Module imports for external Bicep modules (e.g., `import { storage } from 'modules/storage.bicep'`)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged, rename_all = "camelCase")]
pub enum BicepImport {
    /// Namespace import for built-in or extension namespaces.
    ///
    /// Example: `import 'az@1.0.0'` imports the Azure namespace with version 1.0.0
    Namespace {
        /// The namespace being imported (e.g., 'az')
        namespace: String,

        /// Optional version specified after @ (e.g., '1.0.0')
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<String>,
    },

    /// Module import for external Bicep modules.
    ///
    /// Supports both specific symbol imports and wildcard imports:
    /// - `import { storage, network } from 'modules/shared.bicep'`
    /// - `import * as shared from 'modules/shared.bicep'`
    Module {
        /// Path or source of the import
        source: ModuleSource,

        /// For specific symbol imports, the imported symbols
        #[serde(skip_serializing_if = "Option::is_none")]
        symbols: Option<Vec<BicepImportSymbol>>,

        /// For wildcard imports (import * as alias), the alias
        #[serde(skip_serializing_if = "Option::is_none")]
        wildcard_alias: Option<String>,
    },
}

/// Represents an imported symbol in a module import.
///
/// Symbols can be imported with their original name or aliased to a different name.
/// Example: `{ storage as storageModule, network }` imports two symbols, one with an alias.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[skip_serializing_none]
pub struct BicepImportSymbol {
    /// Original symbol name being imported
    pub name: String,
    /// Optional alias for the imported symbol
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
}

// ---------------------------------------------------------------
// Functions
// ---------------------------------------------------------------

/// Parses a namespace import statement.
///
/// Namespace imports bring built-in or extension namespaces into scope.
/// These typically have the format: `import 'namespace@version'`
///
/// # Arguments
///
/// * `node` - The tree-sitter Node representing the namespace import
/// * `source_code` - The source code text containing the import statement
///
/// # Returns
///
/// A Result containing a BicepImport::Namespace if successful
///
/// # Errors
///
/// Returns an error if the import statement cannot be parsed or is malformed
///
/// # Examples
///
/// ```rust,ignore
/// // Parsing: import 'az@1.0.0'
/// let import = parse_namespace_import(node, source_code)?;
/// // Result: BicepImport::Namespace with namespace "az" and version "1.0.0"
/// ```
///
/// Note: This example is marked as ignored in doctests because it requires a valid tree-sitter node.
pub fn parse_namespace_import(
    node: Node,
    source_code: &str,
) -> Result<BicepImport, Box<dyn Error>> {
    let mut namespace = String::new();
    let mut version: Option<String> = None;

    // Extract the import string (e.g., 'az@1.0.0')
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "string" {
            let import_text = get_node_text(child, source_code);
            let clean_text = import_text.trim_matches('\'').trim_matches('"');

            if clean_text.is_empty() {
                return Err(Box::new(BicepParserError::ParseError(
                    "Namespace import has empty string".to_string(),
                )));
            }

            // Parse namespace and version (separated by @)
            if let Some(at_index) = clean_text.find('@') {
                namespace = clean_text[0..at_index].to_string();
                version = Some(clean_text[at_index + 1..].to_string());

                if namespace.is_empty() {
                    return Err(Box::new(BicepParserError::ParseError(
                        "Namespace import has empty namespace".to_string(),
                    )));
                }

                debug!(
                    "Parsed namespace import: {} version {}",
                    namespace,
                    version.as_ref().unwrap()
                );
            } else {
                namespace = clean_text.to_string();
                debug!("Parsed namespace import: {} (no version)", namespace);
            }

            break;
        }
    }

    if namespace.is_empty() {
        return Err(Box::new(BicepParserError::ParseError(
            "Could not extract namespace from import statement".to_string(),
        )));
    }

    Ok(BicepImport::Namespace { namespace, version })
}

/// Parses a module import statement.
///
/// Module imports allow importing specific symbols or all symbols from external Bicep modules.
/// Supports two formats:
/// - Specific imports: `import { symbol1, symbol2 as alias } from 'path'`
/// - Wildcard imports: `import * as alias from 'path'`
///
/// # Arguments
///
/// * `node` - The tree-sitter Node representing the module import
/// * `source_code` - The source code text containing the import statement
///
/// # Returns
///
/// A Result containing a BicepImport::Module if successful
///
/// # Errors
///
/// Returns an error if the import statement cannot be parsed or is malformed
///
/// # Examples
///
/// ```rust,ignore
/// // Parsing: import { storage, network as net } from './modules/shared.bicep'
/// let import = parse_module_import(node, source_code)?;
/// // Result: BicepImport::Module with symbols and source path
/// ```
///
/// Note: This example is marked as ignored in doctests because it requires a valid tree-sitter node.
pub fn parse_module_import(node: Node, source_code: &str) -> Result<BicepImport, Box<dyn Error>> {
    let mut symbols: Vec<BicepImportSymbol> = Vec::new();
    let mut wildcard_alias: Option<String> = None;

    let mut cursor = node.walk();
    let children = node.children(&mut cursor).collect::<Vec<_>>();

    // Extract the source path from the 'from' clause
    let source_path = extract_import_source_path(&children, source_code)?;

    // Parse import type (symbols or wildcard)
    parse_import_specifiers(&children, source_code, &mut symbols, &mut wildcard_alias)?;

    debug!(
        "Parsed module import from '{}' with {} symbols",
        source_path,
        symbols.len()
    );

    // Parse the source path to get the ModuleSource
    let source = ModuleSource::parse(&source_path).unwrap_or(ModuleSource::LocalPath(source_path));

    Ok(BicepImport::Module {
        source,
        symbols: if symbols.is_empty() {
            None
        } else {
            Some(symbols)
        },
        wildcard_alias,
    })
}

/// Extracts the source path from module import children nodes.
fn extract_import_source_path(
    children: &[Node],
    source_code: &str,
) -> Result<String, Box<dyn Error>> {
    for (i, child) in children.iter().enumerate() {
        if child.kind() == "from" && i + 1 < children.len() && children[i + 1].kind() == "string" {
            let path = get_node_text(children[i + 1], source_code);
            let clean_path = path.trim_matches('\'').trim_matches('"');

            if clean_path.is_empty() {
                return Err(Box::new(BicepParserError::ParseError(
                    "Module import has empty source path".to_string(),
                )));
            }

            return Ok(clean_path.to_string());
        }
    }

    Err(Box::new(BicepParserError::ParseError(
        "Could not find source path in module import".to_string(),
    )))
}

/// Parses import specifiers (symbols or wildcard).
fn parse_import_specifiers(
    children: &[Node],
    source_code: &str,
    symbols: &mut Vec<BicepImportSymbol>,
    wildcard_alias: &mut Option<String>,
) -> Result<(), Box<dyn Error>> {
    for (i, child) in children.iter().enumerate() {
        match child.kind() {
            "*" => {
                // Wildcard import: import * as alias
                if let Some(alias) = extract_wildcard_alias(children, i, source_code) {
                    *wildcard_alias = Some(alias);
                    debug!("Found wildcard import with alias");
                }
                return Ok(());
            },
            "{" => {
                // Symbol import: import { symbol1, symbol2 as alias }
                parse_symbol_list(children, i, source_code, symbols)?;
                return Ok(());
            },
            _ => continue,
        }
    }

    Ok(())
}

/// Extracts alias from wildcard import.
fn extract_wildcard_alias(
    children: &[Node],
    wildcard_index: usize,
    source_code: &str,
) -> Option<String> {
    if wildcard_index + 2 < children.len()
        && children[wildcard_index + 1].kind() == "as"
        && children[wildcard_index + 2].kind() == "identifier"
    {
        Some(get_node_text(children[wildcard_index + 2], source_code))
    } else {
        None
    }
}

/// Parses the symbol list in braces.
fn parse_symbol_list(
    children: &[Node],
    brace_index: usize,
    source_code: &str,
    symbols: &mut Vec<BicepImportSymbol>,
) -> Result<(), Box<dyn Error>> {
    let mut j = brace_index + 1;

    while j < children.len() && children[j].kind() != "}" {
        if children[j].kind() == "identifier" {
            let symbol = parse_import_symbol(children, j, source_code, &mut j)?;
            symbols.push(symbol);
        } else {
            j += 1; // Skip commas and other tokens
        }
    }

    Ok(())
}

/// Parses a single import symbol with optional alias.
fn parse_import_symbol(
    children: &[Node],
    start_index: usize,
    source_code: &str,
    current_index: &mut usize,
) -> Result<BicepImportSymbol, Box<dyn Error>> {
    let symbol_name = get_node_text(children[start_index], source_code);

    // Check for alias: symbol as alias
    if start_index + 2 < children.len()
        && children[start_index + 1].kind() == "as"
        && children[start_index + 2].kind() == "identifier"
    {
        let alias_name = get_node_text(children[start_index + 2], source_code);
        *current_index = start_index + 3;

        debug!(
            "Parsed symbol '{}' with alias '{}'",
            symbol_name, alias_name
        );

        Ok(BicepImportSymbol {
            name: symbol_name,
            alias: Some(alias_name),
        })
    } else {
        *current_index = start_index + 1;

        debug!("Parsed symbol '{}'", symbol_name);

        Ok(BicepImportSymbol {
            name: symbol_name,
            alias: None,
        })
    }
}
