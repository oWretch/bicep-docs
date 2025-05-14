# Bicep Docs

A documentation generator for Azure Bicep files, similar to terraform-docs.

## Overview

This library uses the tree-sitter grammar for Azure Bicep files to parse and generate a YAML representation of the Abstract Syntax Tree (AST). This can be used to analyze Bicep files programmatically, extract documentation, or perform other operations based on the structure of Bicep files.

## Installation

The package is available on crates.io:

```bash
cargo add bicep-docs
```

## Usage

### Command Line

#### AST Export Tool

The enhanced AST export tool (`ast_export_clap`) allows you to export the Abstract Syntax Tree of a Bicep file in various formats with many options for filtering and analyzing the structure:

```bash
# Export AST to YAML (default)
cargo run --bin ast_export_clap -- path/to/your/bicep_file.bicep

# Show only statistics about the AST
cargo run --bin ast_export_clap -- --stats path/to/your/bicep_file.bicep

# Export AST to JSON
cargo run --bin ast_export_clap -- --format json path/to/your/bicep_file.bicep

# Export a simplified tree format (more concise)
cargo run --bin ast_export_clap -- --format simpletree path/to/your/bicep_file.bicep

# Filter by node type (e.g., only resource declarations)
cargo run --bin ast_export_clap -- --type-filter resource_declaration path/to/your/bicep_file.bicep

# Show tree structure visualization
cargo run --bin ast_export_clap -- --structure path/to/your/bicep_file.bicep

# Filter by line number
cargo run --bin ast_export_clap -- --line 10 path/to/your/bicep_file.bicep

# Get help and examples
cargo run --bin ast_export_clap -- --help-examples
cargo run --bin ast_export_clap -- --help-node-types
cargo run --bin ast_export_clap -- --help-field-names
```

See the examples directory for more details and sample outputs.

#### Convert a Bicep file to YAML (simplified):

```bash
cargo run --bin bicep_to_yaml path/to/your/file.bicep
```

This will create a file named `file_tree.yaml` in the same directory as your input file.

#### Export full AST to YAML:

```bash
cargo run --bin ast_export path/to/your/file.bicep [output_file.yaml]
```

This will create a detailed YAML file containing the complete AST structure of your Bicep file with field names, positions, and text content.

### Library Usage

```rust
use bicep_docs::bicep_to_yaml;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read bicep file content
    let bicep_content = fs::read_to_string("path/to/your/file.bicep")?;
    
    // Convert to YAML
    let yaml_content = bicep_to_yaml(&bicep_content)?;
    
    // Write or process the YAML content
    fs::write("output.yaml", yaml_content)?;
    
    Ok(())
}
```

## YAML Structure

The generated YAML represents the AST structure with the following properties for each node:

```yaml
kind: <node type>               # The node type in the tree-sitter grammar
field_name: <field name or null> # The field name in the parent grammar rule
named: <true/false>             # Whether this is a named node in the grammar
start_position:                 # Start position in the source file
  row: <row number>
  column: <column number>
end_position:                   # End position in the source file
  row: <row number>
  column: <column number>
start_byte: <byte offset>       # Start byte offset in the source file
end_byte: <byte offset>         # End byte offset in the source file
text: <node text>               # The actual text content of this node
children: [<child nodes>]       # Child nodes (recursive structure)
```

- `kind`: The type of tree-sitter node
- `type_`: Additional type information (if available)
- `named`: Whether this is a named node
- `start_position`: Starting position (row, column)
- `end_position`: Ending position (row, column)
- `start_byte`: Start position in bytes
- `end_byte`: End position in bytes
- `text`: The source text for this node
- `children`: Child nodes (recursive structure)
