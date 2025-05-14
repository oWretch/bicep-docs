# GitHub Copilot Instructions for Bicep-Docs

This document outlines the coding standards, best practices, and architectural patterns used in the Bicep-Docs project. Follow these guidelines when making suggestions or generating code.

## Project Overview

Bicep-Docs is a Rust-based parser and documentation generator for Azure Bicep files. It uses tree-sitter for parsing and provides structured output in YAML format.

## Coding Standards

### Rust Idioms

1. **Error Handling**
   - Use `Result<T, E>` for fallible operations
   - Prefer `Box<dyn Error>` for error types in parsing functions
   - Use `?` operator for error propagation
   - Avoid `unwrap()` in library code; use proper error handling

2. **Memory Management**
   - Prefer borrowing (`&str`, `&[T]`) over owned types when possible
   - Use `String::new()` instead of `"".to_string()` for empty strings
   - Use `Vec::with_capacity()` when the size is known
   - Prefer `&str` over `String` for function parameters when ownership isn't needed

3. **Pattern Matching**
   - Use exhaustive pattern matching with `match`
   - Prefer `if let` for single pattern matches
   - Use `matches!` macro for boolean pattern checks

4. **Collections**
   - Use `IndexMap` for ordered key-value mappings (maintains insertion order)
   - Use `Vec` for ordered collections
   - Use `HashMap` for unordered key-value mappings when order doesn't matter

### Documentation Standards

1. **Function Documentation**
   ```rust
   /// Brief description of what the function does
   ///
   /// More detailed explanation if needed, including:
   /// - Algorithm description
   /// - Performance characteristics
   /// - Error conditions
   ///
   /// # Arguments
   ///
   /// * `param1` - Description of parameter
   /// * `param2` - Description of parameter
   ///
   /// # Returns
   ///
   /// Description of return value and its variants
   ///
   /// # Errors
   ///
   /// Description of when this function returns an error
   ///
   /// # Examples
   ///
   /// ```rust
   /// // Example usage
   /// ```
   ```

2. **Struct Documentation**
   ```rust
   /// Brief description of the struct's purpose
   ///
   /// Detailed explanation of what this struct represents,
   /// its use cases, and any important invariants.
   #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
   pub struct MyStruct {
       /// Description of this field
       pub field1: Type1,
       /// Description of this field
       pub field2: Type2,
   }
   ```

3. **Module Documentation**
   - Each module should have a module-level doc comment
   - Explain the module's purpose and main types/functions

### Security Best Practices

1. **Input Validation**
   - Validate all external inputs (file paths, node indices)
   - Use bounds checking for array/slice access
   - Sanitize strings before processing

2. **Memory Safety**
   - Avoid unsafe code unless absolutely necessary
   - Use safe indexing methods (`get()` instead of direct indexing)
   - Handle UTF-8 validation properly

3. **Resource Management**
   - Limit recursion depth in tree traversal
   - Implement timeouts for long-running operations
   - Use streaming for large files when possible

### Performance Guidelines

1. **String Operations**
   - Use `&str` for read-only string operations
   - Pre-allocate `String` capacity when size is known
   - Use `format!` sparingly; prefer string concatenation for simple cases

2. **Collection Operations**
   - Use `Vec::with_capacity()` when size is predictable
   - Prefer iteration over indexing
   - Use `collect()` judiciously; consider fold operations

3. **Tree Traversal**
   - Implement iterative algorithms when possible to avoid stack overflow
   - Cache frequently accessed node properties
   - Use early returns to avoid unnecessary work

## Architecture Patterns

### Parsing Module Structure

```
src/parsing/
├── mod.rs           # Main types and utilities
├── parameters.rs    # Parameter parsing
├── resources.rs     # Resource parsing
├── types.rs         # Type definitions parsing
├── variables.rs     # Variable parsing
├── functions.rs     # Function parsing
├── modules.rs       # Module parsing
├── outputs.rs       # Output parsing
└── imports.rs       # Import parsing
```

### Common Patterns

1. **Parser Functions**
   ```rust
   pub fn parse_declaration(
       node: Node,
       source_code: &str,
       decorators: Vec<BicepDecorator>,
   ) -> Result<(String, StructType), Box<dyn Error>> {
       // Implementation
   }
   ```

2. **Error Handling**
   ```rust
   match parse_operation() {
       Ok(result) => process_result(result),
       Err(e) => {
           warn!("Operation failed: {}", e);
           return default_value(); // or propagate error
       }
   }
   ```

3. **Decorator Processing**
   ```rust
   for decorator in &decorators {
       match decorator.name.as_str() {
           "description" | "sys.description" => {
               // Handle description
           }
           "metadata" | "sys.metadata" => {
               // Handle metadata
           }
           _ => {} // Ignore unknown decorators
       }
   }
   ```

## Testing Standards

1. **Unit Tests**
   - Test each parsing function independently
   - Include edge cases and error conditions
   - Use descriptive test names

2. **Integration Tests**
   - Test complete file parsing
   - Validate output format
   - Test with real Bicep files

3. **Property Testing**
   - Use property-based testing for complex parsing logic
   - Verify round-trip serialization/deserialization

## Code Review Guidelines

1. **Before Submitting**
   - Remove debug print statements
   - Add comprehensive documentation
   - Run `cargo clippy` and fix warnings
   - Format code with `cargo fmt`
   - Ensure all tests pass

2. **Review Checklist**
   - [ ] Proper error handling
   - [ ] Documentation present
   - [ ] Performance considerations addressed
   - [ ] Security implications considered
   - [ ] Tests included
   - [ ] No debugging code left

## Specific Project Patterns

### Serde Attributes

1. **Consistent Serialization**
   ```rust
   #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
   #[serde(rename_all = "camelCase")]
   #[skip_serializing_none]
   pub struct BicepType {
       #[serde(skip_serializing_if = "Option::is_none")]
       pub description: Option<String>,
       #[serde(skip_serializing_if = "IndexMap::is_empty")]
       pub metadata: IndexMap<String, BicepValue>,
   }
   ```

2. **Custom Serialization**
   - Implement custom serializers for complex types
   - Handle special cases (union types, references)
   - Maintain backward compatibility

### Tree-Sitter Integration

1. **Node Traversal**
   ```rust
   let mut cursor = node.walk();
   let children = node.children(&mut cursor).collect::<Vec<_>>();
   for child in children {
       match child.kind() {
           "expected_type" => process_child(child),
           _ => continue, // Skip unknown nodes
       }
   }
   ```

2. **Text Extraction**
   - Use `get_node_text()` utility function
   - Handle UTF-8 validation
   - Process escape sequences properly

### Logging

1. **Use Structured Logging**
   ```rust
   use tracing::{debug, info, warn, error};
   
   // Good
   debug!("Parsing parameter: {}", param_name);
   warn!("Unknown decorator: {}", decorator_name);
   
   // Avoid
   println!("DEBUG: ...");
   ```

2. **Log Levels**
   - `error!`: Unrecoverable errors
   - `warn!`: Unexpected but recoverable situations
   - `info!`: Important events
   - `debug!`: Detailed diagnostic information

Remember: The goal is maintainable, performant, and secure code that follows Rust best practices while being accessible to contributors of all levels.
