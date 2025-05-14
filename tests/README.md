# Bicep-Docs Test Suite

This directory contains the comprehensive test suite for the Bicep-Docs parser library. The tests cover all aspects of Bicep file parsing, including:

- Parameters
- Variables
- Resources
- Outputs
- Metadata
- Decorators
- Functions
- Types
- Modules
- Imports
- Exports

## Test Files

Each `.bicep` file in this directory tests a specific feature or component of the Bicep language:

- `parameters.bicep` - Tests parameter declarations with various features (required, optional, secure, defaults, etc.)
- `variables.bicep` - Tests variable declarations with different types and decorators
- `resources.bicep` - Tests resource declarations with different properties (existing, nested, loops, etc.)
- `outputs.bicep` - Tests output declarations with different constraints and types
- `metadata.bicep` - Tests metadata declarations with different value types
- `decorators.bicep` - Tests various decorators and their applications
- `functions.bicep` - Tests function declarations with parameters and return types
- `types.bicep` - Tests custom type declarations including object and union types
- `modules.bicep` - Tests module usage and declarations
- `imports.bicep` - Tests import statements and configurations
- `exports.bicep` - Tests export functionality for various components

## Running Tests

To run the complete test suite use Cargo:

```bash
# Run all tests
cargo test

# Run only parser tests
cargo test --test test_parser

# Run a specific test
cargo test parsing::parameters
```

## Test Structure

The main test file `test_parsing.rs` contains the comprehensive test suite that validates the parsing functionality of the Bicep-Docs library against all the test files. Each test focuses on a specific aspect of Bicep language parsing.

## Test Plan

The file `TEST_PLAN.md` contains the detailed testing strategy, which includes:

1. Analysis of current test failures
2. Phased approach to address gaps between expected and actual behavior
3. Integration testing strategy
4. Success metrics

The test plan outlines how to systematically improve test coverage and parser reliability.

## Adding New Tests

To add a new test:

1. Create a new Bicep file in the `parsing` directory with test cases for the feature you're testing
2. Add a corresponding test function in `test_parsing.rs`
3. Make sure your test properly validates the parsing functionality

## Validation Approach

The tests validate that:

1. All test files can be parsed without errors
2. The parsed document contains the expected elements
3. Element properties and decorators are correctly captured
4. The structure of complex elements (nested resources, object types, etc.) is preserved

## Troubleshooting Test Failures

If tests are failing:

1. Use `./test_single_file.sh <filename>` to examine the actual parsed structure
2. Compare with the expected behavior in the test assertions
3. Check if the issue is with the test expectations or the parser implementation
4. Refer to the test plan for guidance on addressing specific categories of failures
