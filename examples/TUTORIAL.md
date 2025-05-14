# Analyzing Bicep ASTs: A Tutorial

This tutorial will guide you through using the Bicep AST Export tool to analyze and understand Bicep code structure.

## What is an AST?

An Abstract Syntax Tree (AST) is a tree representation of the abstract syntactic structure of source code. Each node in the tree represents a construct in the source code. The AST export tool parses Bicep files and generates a structured representation of the code that can be analyzed programmatically.

## Getting Started

First, let's export a simple Bicep file's AST:

```bash
cargo run --bin export_ast -- examples/single-file/example.bicep
```

This will generate a file named `example_tree.yaml` containing the full AST.

## Understanding the Output

The YAML output contains a hierarchical representation of your Bicep code. Each node has:

- `kind`: The type of node (e.g., "resource_declaration", "parameter_declaration")
- `field_name`: The field name in the parent node (if applicable)
- `named`: Whether this is a named node in the grammar
- `text`: The actual text content of this node
- `children`: Child nodes

## Common Analysis Tasks

### Finding Resource Declarations

To extract all resource declarations:

```bash
cargo run --bin export_ast -- --type-filter resource_declaration examples/single-file/example.bicep
```

### Analyzing Parameters and Outputs

For parameters only:

```bash
cargo run --bin export_ast -- --type-filter parameter_declaration examples/single-file/example.bicep
```

For outputs only:

```bash
cargo run --bin export_ast -- --type-filter output_declaration examples/single-file/example.bicep
```

### Examining Decorators

To focus on decorators:

```bash
cargo run --bin export_ast -- --type-filter decorator examples/single-file/example.bicep
```

### Getting Statistics

For a quick overview of the AST structure:

```bash
cargo run --bin export_ast -- --stats examples/single-file/example.bicep
```

## Advanced Analysis

### Working with JSON Output

Export to JSON format for programmatic analysis:

```bash
cargo run --bin export_ast -- --format json examples/single-file/example.bicep
```

You can then use jq or other JSON processing tools:

```bash
jq '.children[] | select(.kind == "resource_declaration") | .children[] | select(.kind == "identifier") | .text' example_tree.json
```

### Using the Simplified Tree Format

For a more concise representation:

```bash
cargo run --bin export_ast -- --format simpletree examples/single-file/example.bicep
```

### Visual Tree Structure

To visualize the tree structure:

```bash
cargo run --bin export_ast -- --structure --depth-limit 4 examples/single-file/example.bicep
```

### Analyzing Specific Lines

To focus on a particular line:

```bash
cargo run --bin export_ast -- --line 20 examples/single-file/example.bicep
```

## Practical Examples

### Finding All Resource API Versions

```bash
cargo run --bin export_ast -- --field-filter resource_type examples/single-file/example.bicep
```

### Locating Secure Parameters

Extract parameters with @secure() decorators:

1. First, export the AST
2. Analyze parameters with decorators
3. Look for secure() in the decorator expressions

### Finding Resource Dependencies

Look for dependsOn arrays in resources:

```bash
cargo run --bin export_ast -- --path "dependsOn" examples/single-file/example.bicep
```

## Best Practices

1. Start with the `--stats` option to get an overview
2. Use `--structure` to visualize the tree structure
3. Use `--type-filter` to focus on specific node types
4. Use `--format simpletree` for cleaner output
5. Combine with `grep` or `jq` for complex analysis

## Common Bicep AST Node Types

- `infrastructure`: Root node of the entire AST
- `metadata_declaration`: Metadata statements
- `parameter_declaration`: Parameter declarations
- `variable_declaration`: Variable declarations
- `resource_declaration`: Resource declarations
- `output_declaration`: Output declarations
- `type_declaration`: Type declarations
- `function_declaration`: Function declarations
- `decorator`: Decorator expressions (e.g., @secure())
- `object`: Object literal expressions
- `array`: Array literal expressions

See `--help-node-types` for a complete list.

## Common Field Names

- `name`: Name of declarations or properties
- `value`: Value of properties or variables
- `type`: Type specifier in parameters/variables
- `resource_type`: Type string in resource declarations
- `api_version`: API version in resource declarations

See `--help-field-names` for a complete list.
