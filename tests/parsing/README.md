# Bicep Test Files

This directory contains a collection of Bicep files used for testing the Bicep parser. Each file focuses on specific Bicep language constructs to ensure that the parser can correctly extract and represent them.

## Files and Their Purpose

- **parameters.bicep**: Tests parameter declarations including required/optional parameters, secure parameters, and parameters with decorators.
- **variables.bicep**: Tests variable declarations including simple variables, variables with descriptions, and exported variables.
- **resources.bicep**: Tests resource declarations including various resource types, existing resources, conditional resources, and parent-child relationships.
- **outputs.bicep**: Tests output declarations including simple outputs, secure outputs, and outputs with constraints.
- **metadata.bicep**: Tests metadata declarations with various value types (string, number, boolean).
- **decorators.bicep**: Tests various decorators like @description, @minLength, @maxLength, @secure, etc.
- **functions.bicep**: Tests function declarations including simple functions, functions with parameters, and exported functions.
- **types.bicep**: Tests type definitions including simple types and complex object types with properties.
- **modules.bicep**: Tests module usage including local modules, registry modules, and conditional modules.
- **imports.bicep**: Tests import statements including namespace imports, module imports, wildcard imports, and explicit symbol imports.
- **exports.bicep**: Tests export capabilities for types, variables, and functions.

## Testing Strategy

Each file is designed to test a specific aspect of the Bicep language. The corresponding test functions in `test_parsing.rs` validate that the parser correctly extracts and represents these constructs.

The tests are designed to be resilient to changes in the actual content of the Bicep files, focusing on validating the structure and properties of the parsed elements rather than their specific values.
