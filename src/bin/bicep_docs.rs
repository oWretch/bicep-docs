use bicep_docs::{
    export_bicep_document_to_json, export_bicep_document_to_markdown_with_format,
    export_bicep_document_to_yaml, MarkdownFormat,
};
use clap::{Args, Parser, Subcommand};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

/// Bicep Documentation Generator
///
/// Parse Azure Bicep files and export documentation in a range of formats
#[derive(Parser)]
#[command(name = "bicep-docs")]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Available commands
#[derive(Subcommand)]
enum Commands {
    /// Export Bicep file to Markdown format
    #[clap(alias = "md")]
    Markdown {
        /// Format for displaying properties (table or list)
        #[arg(short, long, default_value = "table")]
        format: MarkdownFormat,

        #[command(flatten)]
        common: CommonExportOptions,
    },
    /// Export Bicep file to YAML format
    Yaml {
        #[command(flatten)]
        common: CommonExportOptions,
    },
    /// Export Bicep file to JSON format
    Json {
        #[command(flatten)]
        common: CommonExportOptions,

        /// Format JSON with indentation for readability
        #[arg(short, long, default_value_t = false)]
        pretty: bool,
    },
}

/// Common options shared between export formats
#[derive(Args)]
struct CommonExportOptions {
    /// Path to the Bicep file to parse
    #[arg(value_name = "BICEP_FILE")]
    input: PathBuf,

    /// Output file path (defaults to input filename with appropriate extension)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

/// Handle the YAML export command
fn handle_yaml_export(common: CommonExportOptions) -> Result<(), Box<dyn Error>> {
    // Read the Bicep file
    let source_code = fs::read_to_string(&common.input)?;

    // Parse the Bicep file
    let document = bicep_docs::parse_bicep_document(&source_code)?;

    // Determine output path if not provided
    let output_path = if let Some(path) = common.output {
        path
    } else {
        let file_stem = common
            .input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");

        Path::new(file_stem).with_extension("yaml")
    };

    // Export the document
    export_bicep_document_to_yaml(&document, &output_path)?;
    println!("YAML exported to: {}", output_path.display());

    Ok(())
}

/// Handle the JSON export command
fn handle_json_export(common: CommonExportOptions, pretty: bool) -> Result<(), Box<dyn Error>> {
    // Read the Bicep file
    let source_code = fs::read_to_string(&common.input)?;

    // Parse the Bicep file
    let document = bicep_docs::parse_bicep_document(&source_code)?;

    // Determine output path if not provided
    let output_path = if let Some(path) = common.output {
        path
    } else {
        let file_stem = common
            .input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");

        Path::new(file_stem).with_extension("json")
    };

    // Export the document
    export_bicep_document_to_json(&document, &output_path, pretty)?;
    println!("JSON exported to: {}", output_path.display());
    if pretty {
        println!("Output is formatted with indentation.");
    }

    Ok(())
}

/// Handle the Markdown export command
fn handle_markdown_export(
    common: CommonExportOptions,
    format: MarkdownFormat,
) -> Result<(), Box<dyn Error>> {
    // Read the Bicep file
    let source_code = fs::read_to_string(&common.input)?;

    // Parse the Bicep file
    let document = bicep_docs::parse_bicep_document(&source_code)?;

    // Determine output path if not provided
    let output_path = if let Some(path) = common.output {
        path
    } else {
        let file_stem = common
            .input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");

        Path::new(file_stem).with_extension("md")
    };

    // Export the document with the specified format
    export_bicep_document_to_markdown_with_format(&document, &output_path, format)?;
    println!(
        "Markdown exported to: {} (format: {})",
        output_path.display(),
        if matches!(format, MarkdownFormat::Table) {
            "table"
        } else {
            "list"
        }
    );

    Ok(())
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Yaml { common } => handle_yaml_export(common)?,
        Commands::Json { common, pretty } => handle_json_export(common, pretty)?,
        Commands::Markdown { common, format } => handle_markdown_export(common, format)?,
    }

    Ok(())
}
