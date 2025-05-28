//! Text processing utilities for Bicep files
//!
//! This module contains utilities for extracting and processing text from
//! tree-sitter nodes, including escape sequence handling and primitive value extraction.

use tree_sitter::Node;

/// Extract text content from a tree-sitter node
///
/// This is a fundamental utility function that extracts the source code text
/// corresponding to a specific node in the syntax tree.
///
/// # Arguments
///
/// * `node` - The tree-sitter Node to extract text from
/// * `source_code` - The complete source code string
///
/// # Returns
///
/// The text content as a String
pub fn get_node_text(node: Node, source_code: &str) -> String {
    let start_byte = node.start_byte();
    let end_byte = node.end_byte();
    
    // Ensure we don't go out of bounds
    if start_byte <= source_code.len() && end_byte <= source_code.len() && start_byte <= end_byte {
        source_code[start_byte..end_byte].to_string()
    } else {
        String::new()
    }
}

/// Process escape sequences in a string
///
/// Handles common escape sequences found in Bicep strings,
/// converting them to their actual character representations.
/// This function handles the complete Bicep string format including
/// multiline strings and Unicode escapes.
///
/// # Arguments
///
/// * `text` - The text containing potential escape sequences
///
/// # Returns
///
/// The processed text with escape sequences converted
pub fn process_escape_sequences(text: &str) -> String {
    // Determine the string format
    let is_multiline = text.starts_with("'''") && text.ends_with("'''");
    let is_single_quote = text.starts_with('\'') && text.ends_with('\'') && !is_multiline;
    let is_double_quote = text.starts_with('"') && text.ends_with('"');

    // Extract the content without quotes
    let content = if is_multiline && text.len() >= 6 {
        &text[3..text.len() - 3]
    } else if (is_single_quote || is_double_quote) && text.len() >= 2 {
        &text[1..text.len() - 1]
    } else {
        text // No quotes to remove
    };

    // Pre-allocate result string with content length as estimate
    let mut result = String::with_capacity(content.len());
    let mut chars = content.char_indices();

    while let Some((_, ch)) = chars.next() {
        // Check for escape sequences
        if ch == '\\' {
            if let Some((_, next_ch)) = chars.next() {
                match next_ch {
                    't' => result.push('\t'),
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    '\'' => result.push('\''),
                    '"' => result.push('"'),
                    '\\' => result.push('\\'),
                    'u' => {
                        // Unicode escape - check if it's in the format \u{...}
                        if let Some((_, '{')) = chars.next() {
                            let mut hex_digits = String::new();
                            let mut found_closing_brace = false;

                            // Collect hex digits until we find '}'
                            for (_, hex_ch) in chars.by_ref() {
                                if hex_ch == '}' {
                                    found_closing_brace = true;
                                    break;
                                } else if hex_ch.is_ascii_hexdigit() {
                                    hex_digits.push(hex_ch);
                                } else {
                                    // Invalid hex digit, break
                                    break;
                                }
                            }

                            if found_closing_brace && !hex_digits.is_empty() {
                                // Convert hex to Unicode character
                                if let Ok(code_point) = u32::from_str_radix(&hex_digits, 16) {
                                    if let Some(unicode_char) = std::char::from_u32(code_point) {
                                        result.push(unicode_char);
                                        continue;
                                    }
                                }
                            }
                        }

                        // If we get here, it's not a valid unicode escape
                        result.push('\\');
                        result.push('u');
                    },
                    _ => {
                        // Unknown escape sequence, treat as literal
                        result.push('\\');
                        result.push(next_ch);
                    },
                }
            } else {
                // Trailing backslash
                result.push('\\');
            }
        } else {
            // Regular character - properly handle UTF-8
            result.push(ch);
        }
    }

    result
}

/// Extract primitive value from text, removing quotes and processing escapes
///
/// This function handles the conversion of quoted string literals to their
/// actual string values, including escape sequence processing.
///
/// # Arguments
///
/// * `text` - The raw text that may contain quotes and escape sequences
///
/// # Returns
///
/// The processed primitive value as a String
pub fn get_primitive_value_from_text(text: &str) -> String {
    let trimmed = text.trim();
    
    // Handle quoted strings (both single and double quotes)
    if (trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2)
        || (trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() >= 2)
    {
        let inner = &trimmed[1..trimmed.len() - 1];
        process_escape_sequences(inner)
    } else {
        // Not a quoted string, return as-is
        trimmed.to_string()
    }
}

/// Extract a primitive Bicep value from a node
///
/// This function extracts primitive values (strings, numbers, booleans) from
/// tree-sitter nodes and returns them as BicepValues.
///
/// # Arguments
///
/// * `node` - The tree-sitter Node representing a primitive value
/// * `source_code` - The source code text
///
/// # Returns
///
/// A Result containing the parsed BicepValue
pub fn get_primitive_value(node: Node, source_code: &str) -> Result<crate::BicepValue, Box<dyn std::error::Error>> {
    let node_text = get_node_text(node, source_code);
    match node.kind() {
        "string" => {
            // Remove the surrounding quotes for string values
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
            Ok(crate::BicepValue::String(process_escape_sequences(&text)))
        }
        "integer" => {
            let value = node_text.parse::<i64>();
            match value {
                Ok(num) => Ok(crate::BicepValue::Int(num)),
                Err(_) => Ok(crate::BicepValue::String(node_text)),
            }
        }
        "boolean" => {
            let value = node_text.parse::<bool>();
            match value {
                Ok(b) => Ok(crate::BicepValue::Bool(b)),
                Err(_) => Ok(crate::BicepValue::String(node_text)),
            }
        }
        _ => Ok(crate::BicepValue::String(node_text)),
    }
}
