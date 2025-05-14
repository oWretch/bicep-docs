// AST exporter for Bicep files using tree-sitter
// Uses clap for command line argument parsing
use bicep_docs::parse_bicep_file;
use clap::{Parser, ValueEnum};
use serde::Serialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

/// Output format options for the AST
#[derive(ValueEnum, Debug, Clone, Copy)]
enum OutputFormat {
    Yaml,
    Json,
    /// Simplified tree format (experimental)
    SimpleTree,
}

/// Command line arguments for the AST export tool
#[derive(Parser, Debug)]
#[command(
    name = "bicep-ast",
    version,
    about = "Export Bicep Abstract Syntax Tree (AST) to YAML or JSON",
    long_about = "This tool parses a Bicep file and exports its Abstract Syntax Tree (AST) to a YAML or JSON file. \
    It provides various filtering and output options to help analyze and understand Bicep code structure.",
    after_help = "Use --help-examples to see usage examples, --help-node-types to see common node types, or \
    --help-field-names to see common field names in the Bicep AST."
)]
struct CliArgs {
    /// Path to the Bicep file to parse
    #[arg(required_unless_present_any = ["help_examples", "help_node_types", "help_field_names"], help = "Path to the Bicep file to parse")]
    input_file: Option<String>,

    /// Output file path (defaults to <input_name>_tree.yaml or <input_name>_tree.json)
    #[arg(short, long, help = "Output file path")]
    output_file: Option<PathBuf>,

    /// Only show statistics about the AST, don't export to file
    #[arg(short, long, help = "Show statistics only (no file output)")]
    stats: bool,

    /// Filter nodes by type (e.g., 'resource_declaration', 'parameter_declaration')
    #[arg(
        short = 't',
        long,
        help = "Filter nodes by type (e.g., 'resource_declaration')"
    )]
    type_filter: Option<String>,

    /// Filter nodes by path pattern (text content)
    #[arg(short = 'p', long, help = "Filter nodes by text content")]
    path: Option<String>,

    /// Filter nodes by line number
    #[arg(short = 'l', long, help = "Filter nodes by line number")]
    line: Option<usize>,

    /// Exclude node text from the output for cleaner visualization
    #[arg(short = 'c', long, help = "Exclude full node text from the output")]
    compact: bool,

    /// Output format (yaml or json)
    #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::Yaml, help = "Output format (yaml or json)")]
    format: OutputFormat,

    /// Show tree structure visualization of node hierarchy
    #[arg(
        short = 'v',
        long,
        help = "Show tree structure visualization of node hierarchy"
    )]
    structure: bool,

    /// Depth limit for tree structure visualization (0 for unlimited)
    #[arg(
        long,
        default_value_t = 5,
        help = "Depth limit for tree structure (0 for unlimited)"
    )]
    depth_limit: usize,

    /// Include AST path in node output
    #[arg(long, help = "Include AST path in node output")]
    include_path: bool,

    /// Maximum number of nodes to include (0 for unlimited)
    #[arg(
        long,
        default_value_t = 0,
        help = "Maximum number of nodes to include (0 for unlimited)"
    )]
    max_nodes: usize,

    /// Only include nodes matching a specific field name
    #[arg(long, help = "Only include nodes with specified field name")]
    field_filter: Option<String>,

    /// Find nodes by path pattern (e.g. "resource_declaration/object/property")
    #[arg(
        long,
        help = "Find nodes by path pattern (e.g. 'resource_declaration/object/property')"
    )]
    path_pattern: Option<String>,

    /// Show detailed usage examples
    #[arg(
        long,
        help = "Show detailed usage examples",
        conflicts_with = "input_file"
    )]
    help_examples: bool,

    /// Show common node types in Bicep AST
    #[arg(
        long,
        help = "Show common node types in Bicep AST",
        conflicts_with = "input_file"
    )]
    help_node_types: bool,

    /// Show common field names in Bicep AST
    #[arg(
        long,
        help = "Show common field names in Bicep AST",
        conflicts_with = "input_file"
    )]
    help_field_names: bool,
}

#[derive(Serialize, Debug, Clone)]
struct NodeSerialized {
    /// The node type in the tree-sitter grammar
    kind: String,
    /// The field name in the parent grammar rule (e.g., "namespace", "name", "value")
    field_name: Option<String>,
    /// Whether this is a named node in the tree-sitter grammar
    named: bool,
    /// Start position (row, column) in the source file
    #[serde(skip_serializing)]
    start_position: Position,
    /// End position (row, column) in the source file
    #[serde(skip_serializing)]
    end_position: Position,
    /// Start byte offset in the source file
    #[serde(skip_serializing)]
    start_byte: usize,
    /// End byte offset in the source file
    #[serde(skip_serializing)]
    end_byte: usize,
    /// The actual text content of this node
    text: String,
    /// Full path to this node in the AST
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    /// Child nodes
    children: Vec<NodeSerialized>,
}

#[derive(Serialize, Debug, Clone)]
struct Position {
    row: usize,
    column: usize,
}

/// Simplified tree node for easier analysis
#[derive(Serialize, Debug)]
struct SimpleTreeNode {
    kind: String,
    field: Option<String>,
    line: usize,
    text: Option<String>,
    children: Vec<SimpleTreeNode>,
}

/// Convert a NodeSerialized to a SimpleTreeNode
fn to_simple_tree(node: &NodeSerialized, include_text: bool) -> SimpleTreeNode {
    SimpleTreeNode {
        kind: node.kind.clone(),
        field: node.field_name.clone(),
        line: node.start_position.row + 1,
        text: if include_text {
            Some(node.text.chars().take(50).collect::<String>())
        } else {
            None
        },
        children: node
            .children
            .iter()
            .map(|child| to_simple_tree(child, include_text))
            .collect(),
    }
}

/// Count the total number of nodes in the AST
fn count_nodes(node: &NodeSerialized) -> usize {
    // Count this node plus all its children recursively
    1 + node.children.iter().map(count_nodes).sum::<usize>()
}

/// Calculate the maximum depth of the AST
fn max_depth(node: &NodeSerialized) -> usize {
    if node.children.is_empty() {
        1 // Leaf node has depth 1
    } else {
        // Find the child with the maximum depth and add 1 for this node
        1 + node.children.iter().map(max_depth).max().unwrap_or(0)
    }
}

/// Count the frequency of each node type in the AST
fn count_node_types(node: &NodeSerialized) -> HashMap<String, usize> {
    let mut counts = HashMap::new();

    // Count this node's type
    *counts.entry(node.kind.clone()).or_insert(0) += 1;

    // Count children's types
    for child in &node.children {
        let child_counts = count_node_types(child);
        for (type_name, count) in child_counts {
            *counts.entry(type_name).or_insert(0) += count;
        }
    }

    counts
}

/// Count the frequency of each field name in the AST
fn count_field_names(node: &NodeSerialized) -> HashMap<String, usize> {
    let mut counts = HashMap::new();

    // Count this node's field name if it has one
    if let Some(field_name) = &node.field_name {
        *counts.entry(field_name.clone()).or_insert(0) += 1;
    }

    // Count children's field names
    for child in &node.children {
        let child_counts = count_field_names(child);
        for (field_name, count) in child_counts {
            *counts.entry(field_name).or_insert(0) += count;
        }
    }

    counts
}

/// Filter nodes by line number
fn filter_nodes_by_line(node: &NodeSerialized, line: usize) -> Option<NodeSerialized> {
    // Check if this node contains the specified line
    let node_start_line = node.start_position.row;
    let node_end_line = node.end_position.row;

    if line >= node_start_line && line <= node_end_line {
        // This node contains the line of interest

        // First, clone this node
        let mut result = node.clone();

        // Then filter its children recursively
        result.children = node
            .children
            .iter()
            .filter_map(|child| filter_nodes_by_line(child, line))
            .collect();

        Some(result)
    } else {
        // This node doesn't contain the line of interest
        None
    }
}

/// Filter nodes by type and/or path pattern
fn filter_nodes(
    node: &NodeSerialized,
    type_filter: Option<&str>,
    path_filter: Option<&str>,
) -> NodeSerialized {
    // Check if this node matches the type filter
    let type_match = match type_filter {
        Some(filter) => node.kind == filter,
        None => true,
    };

    let path_match = match path_filter {
        Some(filter) => node.text.contains(filter),
        None => true,
    };

    if type_match && path_match {
        // If this node matches, include it with all its children
        node.clone()
    } else {
        // Otherwise, check children and include only those that match
        let filtered_children = node
            .children
            .iter()
            .filter_map(|child| {
                let filtered = filter_nodes(child, type_filter, path_filter);
                if filtered.children.is_empty() && !type_match && !path_match {
                    None
                } else {
                    Some(filtered)
                }
            })
            .collect();

        // Create a new node with filtered children
        NodeSerialized {
            kind: node.kind.clone(),
            field_name: node.field_name.clone(),
            named: node.named,
            start_position: Position {
                row: node.start_position.row,
                column: node.start_position.column,
            },
            end_position: Position {
                row: node.end_position.row,
                column: node.end_position.column,
            },
            start_byte: node.start_byte,
            end_byte: node.end_byte,
            text: node.text.clone(),
            path: node.path.clone(),
            children: filtered_children,
        }
    }
}

/// Generate paths for all nodes in the AST
fn generate_node_paths(node: &mut NodeSerialized, current_path: &str) {
    // Generate path for this node
    let my_path = if current_path.is_empty() {
        node.kind.clone()
    } else {
        if let Some(field_name) = &node.field_name {
            format!("{}:{}.{}", current_path, field_name, node.kind)
        } else {
            format!("{}.{}", current_path, node.kind)
        }
    };

    // Set this node's path
    node.path = Some(my_path.clone());

    // Recursively set paths for children
    for child in &mut node.children {
        generate_node_paths(child, &my_path);
    }
}

/// Visualize the tree structure of the AST
fn visualize_tree_structure(
    node: &NodeSerialized,
    depth: usize,
    depth_limit: usize,
    prefix: &str,
    is_last: bool,
) {
    if depth_limit > 0 && depth >= depth_limit {
        // Depth limit reached
        return;
    }

    // Generate the current line's prefix
    let this_prefix = if is_last {
        format!("{}└── ", prefix)
    } else {
        format!("{}├── ", prefix)
    };

    // Display this node
    let display_name = if let Some(field_name) = &node.field_name {
        format!(
            "{}:{} ({})",
            field_name,
            node.kind,
            node.start_position.row + 1
        )
    } else {
        format!("{} ({})", node.kind, node.start_position.row + 1)
    };

    // Print this node
    println!("{}{}", this_prefix, display_name);

    // Determine the next level's prefix
    let next_prefix = if is_last {
        format!("{}    ", prefix)
    } else {
        format!("{}│   ", prefix)
    };

    // Recursively print children
    let child_count = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        let is_child_last = i == child_count - 1;
        visualize_tree_structure(child, depth + 1, depth_limit, &next_prefix, is_child_last);
    }
}

/// Apply a field name filter to the nodes
fn filter_by_field_name(node: &NodeSerialized, field_filter: &str) -> Option<NodeSerialized> {
    // Check if this node matches the field filter
    let field_match = match &node.field_name {
        Some(field_name) => field_name == field_filter,
        None => false,
    };

    if field_match {
        // If this node matches, include it with all its children
        return Some(node.clone());
    }

    // Otherwise, check children
    let filtered_children: Vec<NodeSerialized> = node
        .children
        .iter()
        .filter_map(|child| filter_by_field_name(child, field_filter))
        .collect();

    if filtered_children.is_empty() {
        None
    } else {
        // Create a new node with filtered children
        Some(NodeSerialized {
            kind: node.kind.clone(),
            field_name: node.field_name.clone(),
            named: node.named,
            start_position: Position {
                row: node.start_position.row,
                column: node.start_position.column,
            },
            end_position: Position {
                row: node.end_position.row,
                column: node.end_position.column,
            },
            start_byte: node.start_byte,
            end_byte: node.end_byte,
            text: node.text.clone(),
            path: node.path.clone(),
            children: filtered_children,
        })
    }
}

/// Limit the number of nodes in the AST
fn limit_nodes(
    node: &NodeSerialized,
    max_nodes: usize,
    current_count: &mut usize,
) -> Option<NodeSerialized> {
    if *current_count >= max_nodes && max_nodes > 0 {
        return None;
    }

    *current_count += 1;

    if max_nodes > 0 && *current_count >= max_nodes {
        // Reached the limit, return this node without children
        return Some(NodeSerialized {
            kind: node.kind.clone(),
            field_name: node.field_name.clone(),
            named: node.named,
            start_position: node.start_position.clone(),
            end_position: node.end_position.clone(),
            start_byte: node.start_byte,
            end_byte: node.end_byte,
            text: node.text.clone(),
            path: node.path.clone(),
            children: vec![], // No children
        });
    }

    // Process children
    let mut limited_children = Vec::new();
    for child in &node.children {
        if let Some(limited_child) = limit_nodes(child, max_nodes, current_count) {
            limited_children.push(limited_child);
        }

        if max_nodes > 0 && *current_count >= max_nodes {
            break;
        }
    }

    // Return the node with limited children
    Some(NodeSerialized {
        kind: node.kind.clone(),
        field_name: node.field_name.clone(),
        named: node.named,
        start_position: node.start_position.clone(),
        end_position: node.end_position.clone(),
        start_byte: node.start_byte,
        end_byte: node.end_byte,
        text: node.text.clone(),
        path: node.path.clone(),
        children: limited_children,
    })
}

/// Count nodes by line number
fn count_nodes_by_line(node: &NodeSerialized, counts: &mut HashMap<usize, usize>) {
    // Count this node for its line
    let line = node.start_position.row;
    *counts.entry(line).or_insert(0) += 1;

    // Recursively count children
    for child in &node.children {
        count_nodes_by_line(child, counts);
    }
}

/// Display usage examples for the command line tool
fn display_examples() {
    println!("\nExamples:");
    println!("  # Export AST to default YAML file");
    println!("  ast_export_clap example.bicep");
    println!();
    println!("  # Export to JSON format");
    println!("  ast_export_clap -f json example.bicep");
    println!();
    println!("  # Export to simplified tree format (more compact)");
    println!("  ast_export_clap -f simpletree example.bicep");
    println!();
    println!("  # Show only statistics");
    println!("  ast_export_clap --stats example.bicep");
    println!();
    println!("  # Filter by node type");
    println!("  ast_export_clap -t resource_declaration example.bicep");
    println!();
    println!("  # Show tree structure visualization");
    println!("  ast_export_clap --structure example.bicep");
    println!();
    println!("  # Filter nodes at a specific line");
    println!("  ast_export_clap -l 10 example.bicep");
    println!();
    println!("  # Show compact output (exclude full node text)");
    println!("  ast_export_clap -c example.bicep");
    println!();
    println!("  # Filter nodes with a specific field name");
    println!("  ast_export_clap --field-filter name example.bicep");
    println!();
    println!("  # Include AST paths in output");
    println!("  ast_export_clap --include-path example.bicep");
    println!();
    println!("  # Limit output to specific number of nodes");
    println!("  ast_export_clap --max-nodes 100 example.bicep");
}

/// Display information about common node types in the Bicep AST
fn display_node_types_help() {
    println!("\nCommon Bicep AST Node Types:");
    println!("---------------------------");
    println!("  infrastructure         - Root node of the entire AST");
    println!("  metadata_declaration   - Metadata statements");
    println!("  parameter_declaration  - Parameter declarations");
    println!("  variable_declaration   - Variable declarations");
    println!("  resource_declaration   - Resource declarations");
    println!("  output_declaration     - Output declarations");
    println!("  type_declaration       - Type declarations");
    println!("  function_declaration   - Function declarations");
    println!("  object                 - Object literal expressions");
    println!("  array                  - Array literal expressions");
    println!("  property               - Object property");
    println!("  string_literal         - String literal value");
    println!("  numeric_literal        - Numeric literal value");
    println!("  boolean_literal        - Boolean literal value");
    println!("  decorator              - Decorator (e.g., @secure())");
    println!("  decorator_expression   - Expression used in a decorator");
    println!();
    println!("  Use --type-filter to filter nodes by these types");
    println!("  For example: ast_export_clap --type-filter resource_declaration example.bicep");
}

/// Display information about common field names in the Bicep AST
fn display_field_names_help() {
    println!("\nCommon Bicep AST Field Names:");
    println!("----------------------------");
    println!("  name           - Name of declarations or properties");
    println!("  value          - Value of properties or variables");
    println!("  type           - Type specifier in parameters/variables");
    println!("  resource_type  - Type string in resource declarations");
    println!("  api_version    - API version in resource declarations");
    println!("  properties     - Properties section in resources");
    println!("  parent         - Parent reference in resources");
    println!("  scope          - Scope reference in resources");
    println!("  condition      - Condition expression in conditional resources");
    println!("  decorator      - For decorator nodes");
    println!();
    println!("  Use --field-filter to filter nodes with specific field names");
    println!("  For example: ast_export_clap --field-filter name example.bicep");
}

/// Generate a very brief tree structure visualization (just top-level nodes)
fn generate_brief_structure(node: &NodeSerialized) {
    println!("\nBrief AST Structure:");
    println!("-----------------");
    println!("Root: {} ({} nodes)", node.kind, count_nodes(node));

    // Show all top-level declarations
    for (i, child) in node.children.iter().enumerate() {
        if i >= 15 {
            println!(
                "└── ... and {} more top-level nodes",
                node.children.len() - 15
            );
            break;
        }

        let prefix = if i == node.children.len() - 1 || i == 14 {
            "└── "
        } else {
            "├── "
        };
        let display_name = if let Some(field_name) = &child.field_name {
            format!(
                "{}:{} (Line {})",
                field_name,
                child.kind,
                child.start_position.row + 1
            )
        } else {
            format!("{} (Line {})", child.kind, child.start_position.row + 1)
        };

        println!("{}{}", prefix, display_name);
    }
    println!();
}

/// Main entry point for the Bicep AST export tool
fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments using clap
    let args = CliArgs::parse();

    // Check if we need to show help information
    if args.help_examples {
        display_examples();
        return Ok(());
    }

    if args.help_node_types {
        display_node_types_help();
        return Ok(());
    }

    if args.help_field_names {
        display_field_names_help();
        return Ok(());
    }

    // Extract arguments into local variables
    let input_file = args.input_file.as_ref().expect("Input file is required");
    let stats_only = args.stats;
    let filter_type = args.type_filter;
    let filter_path = args.path;
    let filter_line = args.line;
    let compact_mode = args.compact;
    let output_format = args.format;

    // Determine output file name
    let output_file = match args.output_file {
        Some(path) => path.to_string_lossy().to_string(),
        None => {
            // Default output name based on input file and format
            let input_path = PathBuf::from(input_file);
            let stem = input_path.file_stem().unwrap_or_default().to_string_lossy();

            match output_format {
                OutputFormat::Yaml => format!("{}_tree.yaml", stem),
                OutputFormat::Json => format!("{}_tree.json", stem),
                OutputFormat::SimpleTree => format!("{}_simple_tree.json", stem),
            }
        },
    };

    // Read and parse the input file
    println!("Reading Bicep file: {}", input_file);
    let source_code = fs::read_to_string(input_file)
        .map_err(|e| format!("Failed to read file {}: {}", input_file, e))?;

    println!("Parsing Bicep file...");
    let tree = parse_bicep_file(&source_code)
        .ok_or_else(|| format!("Failed to parse file {} as valid Bicep", input_file))?;

    // Convert the tree to a serializable format
    println!("Converting to serializable format...");
    let root_node = tree.root_node();
    let mut serialized = serialize_node(&root_node, &source_code, compact_mode);

    // Apply line filter if requested
    if let Some(line) = filter_line {
        if let Some(filtered) = filter_nodes_by_line(&serialized, line) {
            serialized = filtered;
        } else {
            println!("Warning: No nodes found at line {}", line);
        }
    }

    // Apply type and path filters if requested
    if filter_type.is_some() || filter_path.is_some() {
        serialized = filter_nodes(&serialized, filter_type.as_deref(), filter_path.as_deref());
    }

    // Apply field name filter if requested
    if let Some(field_filter) = &args.field_filter {
        if let Some(filtered) = filter_by_field_name(&serialized, field_filter) {
            serialized = filtered;
        } else {
            println!("Warning: No nodes found with field name '{}'", field_filter);
        }
    }

    // Apply node limit if requested
    if args.max_nodes > 0 {
        let mut current_count = 0;
        if let Some(limited) = limit_nodes(&serialized, args.max_nodes, &mut current_count) {
            serialized = limited;
        }
    }

    // Generate paths for nodes if requested
    if args.include_path {
        generate_node_paths(&mut serialized, "");
    }

    // Apply path pattern search if requested
    if let Some(path_pattern) = &args.path_pattern {
        println!(
            "Searching for nodes matching path pattern: '{}'...",
            path_pattern
        );
        let matching_nodes = find_nodes_by_path_pattern(&serialized, path_pattern);

        if matching_nodes.is_empty() {
            println!(
                "Warning: No nodes found matching the path pattern '{}'",
                path_pattern
            );
        } else {
            println!(
                "Found {} nodes matching the path pattern",
                matching_nodes.len()
            );

            // Create a new root node with all matching nodes as children
            serialized = NodeSerialized {
                kind: "search_results".to_string(),
                field_name: None,
                named: true,
                start_position: Position { row: 0, column: 0 },
                end_position: Position { row: 0, column: 0 },
                start_byte: 0,
                end_byte: 0,
                text: format!("Search results for path pattern: {}", path_pattern),
                path: Some("search_results".to_string()),
                children: matching_nodes,
            };
        }
    }

    // Gather statistics about the AST
    let node_count = count_nodes(&serialized);
    let max_depth = max_depth(&serialized);
    let node_types = count_node_types(&serialized);
    let field_name_count = count_field_names(&serialized);

    if stats_only {
        // Show detailed statistics about the AST
        println!("\nAST Statistics:");
        println!("--------------");
        println!("Source file: {}", input_file);
        println!("File size: {} bytes", source_code.len());
        println!("Total nodes: {}", node_count);
        println!(
            "Nodes per KB: {:.1}",
            node_count as f64 * 1000.0 / source_code.len() as f64
        );
        println!("Maximum depth: {}", max_depth);

        println!("\nTop 10 node types:");
        let mut types: Vec<(&String, &usize)> = node_types.iter().collect();
        types.sort_by(|a, b| b.1.cmp(a.1));

        for (i, (node_type, count)) in types.iter().take(10).enumerate() {
            println!(
                "  {}. {} - {} nodes ({}%)",
                i + 1,
                node_type,
                count,
                (**count as f64 / node_count as f64 * 100.0).round() as usize
            );
        }

        if types.len() > 10 {
            println!("  ... and {} more node types", types.len() - 10);
        }

        println!("\nTop 10 field names:");
        let mut fields: Vec<(&String, &usize)> = field_name_count.iter().collect();
        fields.sort_by(|a, b| b.1.cmp(a.1));

        for (i, (field_name, count)) in fields.iter().take(10).enumerate() {
            println!("  {}. {} - {} occurrences", i + 1, field_name, count);
        }

        if fields.len() > 10 {
            println!("  ... and {} more field names", fields.len() - 10);
        }

        // Count nodes by line number
        let mut line_counts = HashMap::new();
        count_nodes_by_line(&serialized, &mut line_counts);

        println!("\nLine distribution:");
        let mut lines: Vec<(&usize, &usize)> = line_counts.iter().collect();
        lines.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count (highest first)

        println!("  Top 5 lines by node count:");
        for (i, (line, count)) in lines.iter().take(5).enumerate() {
            println!("    {}. Line {} - {} nodes", i + 1, **line + 1, count);
        }
    } else {
        // Show a brief structure visualization of the top-level nodes
        generate_brief_structure(&serialized);

        // Write the AST to the output file in the selected format
        println!("Writing to file: {}", output_file);

        let file_content = match output_format {
            OutputFormat::Yaml => {
                println!("Format: YAML");
                serde_yaml::to_string(&serialized)?
            },
            OutputFormat::Json => {
                println!("Format: JSON");
                serde_json::to_string_pretty(&serialized)?
            },
            OutputFormat::SimpleTree => {
                println!("Format: Simplified Tree (JSON)");
                // Convert to simplified tree format
                let simple_tree = to_simple_tree(&serialized, !compact_mode);
                serde_json::to_string_pretty(&simple_tree)?
            },
        };

        // Write to file
        let mut file = File::create(&output_file)?;
        file.write_all(file_content.as_bytes())?;

        // Show summary after successful export
        println!("\nAST export summary:");
        println!("------------------");
        println!("Source file: {}", input_file);
        println!("File size: {} bytes", source_code.len());
        println!("Total nodes: {}", node_count);
        println!("Maximum depth: {}", max_depth);
        println!("Node types: {} unique types", node_types.len());
        println!("Field names: {} unique fields", field_name_count.len());
        println!("Output file: {}", output_file);
        println!(
            "Output format: {}",
            match output_format {
                OutputFormat::Yaml => "YAML",
                OutputFormat::Json => "JSON",
                OutputFormat::SimpleTree => "Simplified Tree (JSON)",
            }
        );
        println!(
            "Filters applied: {}",
            if filter_type.is_some()
                || filter_path.is_some()
                || filter_line.is_some()
                || args.field_filter.is_some()
                || args.max_nodes > 0
            {
                "Yes"
            } else {
                "No"
            }
        );
        println!("AST exported successfully!");
    }

    // Visualize tree structure if requested
    if args.structure {
        println!("\nTree Structure Visualization:");
        println!("----------------------------");
        visualize_tree_structure(&serialized, 0, args.depth_limit, "", true);
    }

    Ok(())
}

/// Create a serialized representation of a tree-sitter node
fn serialize_node(
    node: &tree_sitter::Node,
    source_code: &str,
    compact_mode: bool,
) -> NodeSerialized {
    let mut children = Vec::new();
    let mut cursor = node.walk();

    // Extract field names for children
    let mut child_field_names = Vec::new();
    cursor.goto_first_child();

    // First pass - collect field names for each child
    if cursor.field_name().is_some() {
        child_field_names.push(cursor.field_name().map(String::from));

        while cursor.goto_next_sibling() {
            child_field_names.push(cursor.field_name().map(String::from));
        }
    }

    // Reset cursor position
    cursor.reset(*node);

    // Second pass - create child nodes with field names
    let mut i = 0;
    for child in node.children(&mut cursor) {
        let field = if i < child_field_names.len() {
            child_field_names[i].clone()
        } else {
            None
        };

        // Create child node with its field name
        let mut child_node = serialize_node(&child, source_code, compact_mode);
        child_node.field_name = field;
        children.push(child_node);

        i += 1;
    }

    // Extract node text from source code (if not in compact mode)
    let text = if compact_mode {
        // In compact mode, include very short text or empty string for longer text
        if node.end_byte() - node.start_byte() <= 20
            && node.start_byte() < node.end_byte()
            && node.end_byte() <= source_code.len()
        {
            source_code[node.start_byte()..node.end_byte()].to_string()
        } else if node.is_named() {
            format!("... ({} bytes)", node.end_byte() - node.start_byte())
        } else {
            String::new()
        }
    } else if node.start_byte() < node.end_byte() && node.end_byte() <= source_code.len() {
        source_code[node.start_byte()..node.end_byte()].to_string()
    } else {
        String::new()
    };

    NodeSerialized {
        kind: node.kind().to_string(),
        field_name: None, // Will be set by parent when adding to its children
        named: node.is_named(),
        start_position: Position {
            row: node.start_position().row,
            column: node.start_position().column,
        },
        end_position: Position {
            row: node.end_position().row,
            column: node.end_position().column,
        },
        start_byte: node.start_byte(),
        end_byte: node.end_byte(),
        text,
        path: None, // Will be set by parent when adding to its children
        children,
    }
}

/// Find nodes matching a path pattern
/// Format: parent_type/child_type/grandchild_type
/// Example: "resource_declaration/object/property"
fn find_nodes_by_path_pattern(node: &NodeSerialized, pattern: &str) -> Vec<NodeSerialized> {
    let path_parts: Vec<&str> = pattern.split('/').collect();

    // Check if the current node matches the first part of the pattern
    if path_parts.is_empty() || node.kind != path_parts[0] {
        // Try with children
        let mut result = Vec::new();
        for child in &node.children {
            result.append(&mut find_nodes_by_path_pattern(child, pattern));
        }
        return result;
    }

    // If we're at the last part of the pattern, we found a match
    if path_parts.len() == 1 {
        return vec![node.clone()];
    }

    // If there are more parts in the pattern, search in children
    let sub_pattern = path_parts[1..].join("/");
    let mut result = Vec::new();

    for child in &node.children {
        result.append(&mut find_nodes_by_path_pattern(child, &sub_pattern));
    }

    result
}
