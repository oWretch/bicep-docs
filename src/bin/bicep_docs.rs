use bicep_docs::{
    export_bicep_document_to_asciidoc_with_format, export_bicep_document_to_json,
    export_bicep_document_to_markdown_with_format, export_bicep_document_to_yaml, AsciiDocFormat,
    MarkdownFormat,
};
use clap::{self, Args, Parser, Subcommand};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, debug_span, error, trace, Level};
use tracing_subscriber::{
    filter::EnvFilter,
    fmt::{self, format::FmtSpan},
    prelude::*,
};

/// Bicep Documentation Generator
///
/// Parse Azure Bicep files and export documentation in a range of formats
#[derive(Parser)]
#[command(name = "bicep-docs")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Set the verbosity level of output (v: debug, vv: trace)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Don't show any logging output
    #[arg(short, long)]
    quiet: bool,

    /// Output logs as JSON
    #[arg(long)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

/// Available commands
#[derive(Subcommand)]
enum Commands {
    /// Document Bicep file in Markdown format
    #[clap(alias = "md")]
    Markdown {
        /// Format for displaying properties (table or list)
        #[arg(short, long, default_value = "table")]
        format: MarkdownFormat,

        #[command(flatten)]
        common: CommonExportOptions,
    },
    /// Document Bicep file in AsciiDoc format
    #[clap(alias = "adoc")]
    Asciidoc {
        /// Format for displaying properties (table or list)
        #[arg(short, long, default_value = "table")]
        format: AsciiDocFormat,

        #[command(flatten)]
        common: CommonExportOptions,
    },
    /// Document Bicep file in YAML format
    Yaml {
        #[command(flatten)]
        common: CommonExportOptions,
    },
    /// Document Bicep file in JSON format
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
    debug!("YAML exported to: {}", output_path.display());

    Ok(())
}

/// Handle the JSON export command
fn handle_json_export(common: CommonExportOptions, pretty: bool) -> Result<(), Box<dyn Error>> {
    debug!(
        "Beginning JSON export for file: {} (pretty: {})",
        common.input.display(),
        pretty
    );

    // Read the Bicep file
    let source_code = fs::read_to_string(&common.input)?;
    debug!(
        "Successfully read Bicep file: {} ({} bytes)",
        common.input.display(),
        source_code.len()
    );

    // Parse the Bicep file
    let document = bicep_docs::parse_bicep_document(&source_code)?;
    debug!("Successfully parsed Bicep document");

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
    debug!("JSON exported to: {}", output_path.display());
    if pretty {
        debug!("Output is formatted with indentation");
    }

    Ok(())
}

/// Handle the Markdown export command
fn handle_markdown_export(
    common: CommonExportOptions,
    format: MarkdownFormat,
) -> Result<(), Box<dyn Error>> {
    debug!(
        "Beginning Markdown export for file: {} (format: {})",
        common.input.display(),
        if matches!(format, MarkdownFormat::Table) {
            "table"
        } else {
            "list"
        }
    );

    // Read the Bicep file
    let source_code = fs::read_to_string(&common.input)?;
    debug!(
        "Successfully read Bicep file: {} ({} bytes)",
        common.input.display(),
        source_code.len()
    );

    // Parse the Bicep file
    let document = bicep_docs::parse_bicep_document(&source_code)?;
    debug!("Successfully parsed Bicep document");

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
    debug!(
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

/// Handle the AsciiDoc export command
fn handle_asciidoc_export(
    common: CommonExportOptions,
    format: AsciiDocFormat,
) -> Result<(), Box<dyn Error>> {
    debug!(
        "Beginning AsciiDoc export for file: {} (format: {})",
        common.input.display(),
        if matches!(format, AsciiDocFormat::Table) {
            "table"
        } else {
            "list"
        }
    );

    // Read the Bicep file
    let source_code = fs::read_to_string(&common.input)?;
    debug!(
        "Successfully read Bicep file: {} ({} bytes)",
        common.input.display(),
        source_code.len()
    );

    // Parse the Bicep file
    let document = bicep_docs::parse_bicep_document(&source_code)?;
    debug!("Successfully parsed Bicep document");

    // Determine output path if not provided
    let output_path = if let Some(path) = common.output {
        path
    } else {
        let file_stem = common
            .input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");

        Path::new(file_stem).with_extension("adoc")
    };

    // Export the document with the specified format
    export_bicep_document_to_asciidoc_with_format(&document, &output_path, format)?;
    debug!(
        "AsciiDoc exported to: {} (format: {})",
        output_path.display(),
        if matches!(format, AsciiDocFormat::Table) {
            "table"
        } else {
            "list"
        }
    );

    Ok(())
}

/// Configure the tracing subscriber based on command line options
fn setup_tracing(verbose: u8, quiet: bool, json: bool) {
    // Set default filter level based on verbosity
    let filter_level = match (verbose, quiet) {
        (_, true) => Level::ERROR, // When quiet is enabled, only show errors
        (0, _) => Level::INFO,     // Default: show info and above
        (1, _) => Level::DEBUG,    // With -v: show debug and above
        (_, _) => Level::TRACE,    // With -vv or more: show everything
    };

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(""))
        .add_directive(filter_level.into());

    // Configure formatting based on user preferences
    if json {
        // Use JSON formatter
        tracing_subscriber::registry()
            .with(filter)
            .with(
                fmt::Layer::default()
                    .json()
                    .with_target(true)
                    .with_span_events(FmtSpan::CLOSE),
            )
            .init();
    } else {
        // Use human-readable formatter with color support
        tracing_subscriber::registry()
            .with(filter)
            .with(
                fmt::Layer::default()
                    .with_target(true)
                    .with_span_events(FmtSpan::CLOSE)
                    .with_timer(fmt::time::time()),
            )
            .init();
    }
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // Setup tracing with the appropriate verbosity
    setup_tracing(cli.verbose, cli.quiet, cli.json);

    trace!("Starting Bicep-Docs with verbosity level: {}", cli.verbose);
    debug!("Parsed command line arguments");

    // Create a top-level span for the command execution
    let command_name = match &cli.command {
        Commands::Yaml { .. } => "yaml",
        Commands::Json { .. } => "json",
        Commands::Markdown { .. } => "markdown",
        Commands::Asciidoc { .. } => "asciidoc",
    };

    let span = debug_span!("bicep_docs_command", command = command_name);
    let _guard = span.enter();

    let result = match cli.command {
        Commands::Yaml { common } => handle_yaml_export(common),
        Commands::Json { common, pretty } => handle_json_export(common, pretty),
        Commands::Markdown { common, format } => handle_markdown_export(common, format),
        Commands::Asciidoc { common, format } => handle_asciidoc_export(common, format),
    };

    if let Err(ref e) = result {
        error!("Command failed: {}", e);
    } else {
        debug!("Command completed successfully");
    }

    result
}
