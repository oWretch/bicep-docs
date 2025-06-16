use std::{
    error::Error,
    fs::{self, File},
    path::{Path, PathBuf},
    process,
};

use bicep_docs::{
    export_bicep_document_to_asciidoc, export_bicep_document_to_asciidoc_string,
    export_bicep_document_to_json, export_bicep_document_to_json_string,
    export_bicep_document_to_markdown, export_bicep_document_to_markdown_string,
    export_bicep_document_to_yaml, export_bicep_document_to_yaml_string,
};
use clap::{self, Args, Parser, Subcommand, ValueEnum};
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
#[command(
    name = "bicep-docs",
    version,
    about,
    long_about = None,
    subcommand_help_heading = "Formats",
    subcommand_value_name = "FORMAT")]
struct Cli {
    /// Set the verbosity level of output (v: debug, vv: trace)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Don't show any logging output
    #[arg(short, long)]
    quiet: bool,

    /// Set the format for logging output
    #[arg(long, value_enum, default_value_t = LogFormat::Text)]
    log_format: LogFormat,

    /// Path to a file to write logs to (instead of stdout/stderr)
    #[arg(long)]
    log_file: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

/// Available log formats
#[derive(Clone, Debug, ValueEnum, Default)]
enum LogFormat {
    #[default]
    Text,
    Json,
}

/// Available commands
#[derive(Subcommand)]
enum Commands {
    /// Document Bicep file in Markdown format
    #[clap(alias = "md")]
    Markdown {
        #[command(flatten)]
        common: CommonExportOptions,
    },
    /// Document Bicep file in AsciiDoc format
    #[clap(alias = "adoc")]
    Asciidoc {
        #[command(flatten)]
        common: CommonExportOptions,
    },
    /// Document Bicep file in YAML format
    #[clap(alias = "yml")]
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
    #[arg(value_name = "BICEP FILE", required = true)]
    #[arg(value_hint = clap::ValueHint::FilePath)]
    input: PathBuf,

    /// Output file path. Defaults to input filename with appropriate extension.
    #[arg(short, long)]
    #[arg(value_hint = clap::ValueHint::FilePath)]
    output: Option<PathBuf>,

    /// Enable emoji usage in documentation output
    #[arg(long, default_value_t = false)]
    emoji: bool,

    /// Skip exporting empty sections in the documentation
    #[arg(long, default_value_t = false)]
    exclude_empty: bool,

    /// Check if generated documentation matches existing file and exit with appropriate code
    #[arg(long, default_value_t = false)]
    check: bool,
}

/// Compare generated content with existing file and exit with appropriate code
fn check_file_diff(generated_content: &str, output_path: &Path) -> Result<(), Box<dyn Error>> {
    let existing_content = match fs::read_to_string(output_path) {
        Ok(content) => content,
        Err(_) => {
            // File doesn't exist, so they're different
            println!("--- {}", output_path.display());
            println!("+++ {}", output_path.display());
            println!("@@ -0,0 +1,{} @@", generated_content.lines().count());

            for line in generated_content.lines() {
                println!("+{line}");
            }

            process::exit(1);
        },
    };

    if generated_content == existing_content {
        // Files match
        process::exit(0);
    } else {
        // Files differ, output unified diff
        let generated_lines: Vec<&str> = generated_content.lines().collect();
        let existing_lines: Vec<&str> = existing_content.lines().collect();

        println!("--- {}", output_path.display());
        println!("+++ {}", output_path.display());

        // Simple unified diff implementation
        let mut i = 0;
        let mut j = 0;
        let mut context_lines = Vec::new();
        let mut changes = Vec::new();

        while i < existing_lines.len() || j < generated_lines.len() {
            if i < existing_lines.len()
                && j < generated_lines.len()
                && existing_lines[i] == generated_lines[j]
            {
                // Lines match
                context_lines.push(format!(" {}", existing_lines[i]));
                i += 1;
                j += 1;

                // If we have changes to flush, do it now
                if !changes.is_empty() {
                    print_diff_hunk(
                        &context_lines,
                        &changes,
                        i - context_lines.len(),
                        j - context_lines.len(),
                    );
                    context_lines.clear();
                    changes.clear();
                }
            } else {
                // Lines differ
                if i < existing_lines.len() {
                    changes.push(format!("-{}", existing_lines[i]));
                    i += 1;
                }
                if j < generated_lines.len() {
                    changes.push(format!("+{}", generated_lines[j]));
                    j += 1;
                }
            }
        }

        // Flush any remaining changes
        if !changes.is_empty() {
            print_diff_hunk(
                &context_lines,
                &changes,
                i - context_lines.len(),
                j - context_lines.len(),
            );
        }

        process::exit(1);
    }
}

/// Print a unified diff hunk
fn print_diff_hunk(
    context_lines: &[String],
    changes: &[String],
    old_start: usize,
    new_start: usize,
) {
    let old_count = changes.iter().filter(|line| line.starts_with('-')).count();
    let new_count = changes.iter().filter(|line| line.starts_with('+')).count();

    println!(
        "@@ -{},{} +{},{} @@",
        old_start + 1,
        old_count,
        new_start + 1,
        new_count
    );

    for line in context_lines {
        println!("{line}");
    }

    for line in changes {
        println!("{line}");
    }
}

/// Generic export handler to reduce duplication
fn handle_export<F, G>(
    common: CommonExportOptions,
    extension: &str,
    export_fn: F,
    export_to_string_fn: G,
) -> Result<(), Box<dyn Error>>
where
    F: Fn(&bicep_docs::parsing::BicepDocument, &Path, bool, bool) -> Result<(), Box<dyn Error>>,
    G: Fn(&bicep_docs::parsing::BicepDocument, bool, bool) -> Result<String, Box<dyn Error>>,
{
    debug!(
        "Beginning {} export for file: {}",
        extension.to_uppercase(),
        common.input.display()
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

    // Determine output path
    let output_path = common
        .output
        .clone()
        .unwrap_or_else(|| common.input.with_extension(extension));
    debug!("Using output path: {}", output_path.display());

    if common.check {
        // Check mode: compare generated content with existing file
        let generated_content = export_to_string_fn(&document, common.emoji, common.exclude_empty)?;
        check_file_diff(&generated_content, &output_path)?;
    } else {
        // Normal mode: export the document
        export_fn(&document, &output_path, common.emoji, common.exclude_empty)?;
        debug!(
            "{} exported to: {}",
            extension.to_uppercase(),
            output_path.display()
        );
    }

    Ok(())
}

/// Handle the YAML export command
fn handle_yaml_export(common: CommonExportOptions) -> Result<(), Box<dyn Error>> {
    if common.check {
        // YAML export doesn't use emoji parameter, so handle separately
        debug!("Beginning YAML check for file: {}", common.input.display());

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

        // Determine output path
        let output_path = common
            .output
            .clone()
            .unwrap_or_else(|| common.input.with_extension("yaml"));
        debug!("Using output path: {}", output_path.display());

        // Generate content and check diff
        let generated_content =
            export_bicep_document_to_yaml_string(&document, common.exclude_empty)?;
        check_file_diff(&generated_content, &output_path)?;

        Ok(())
    } else {
        handle_export(
            common,
            "yaml",
            |doc, path, _emoji, exclude_empty| {
                export_bicep_document_to_yaml(doc, path, exclude_empty)
            },
            |doc, _emoji, exclude_empty| export_bicep_document_to_yaml_string(doc, exclude_empty),
        )
    }
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

    // Determine output path
    let output_path = common.output.clone().unwrap_or_else(|| {
        let file_stem = common
            .input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        Path::new(file_stem).with_extension("json")
    });

    if common.check {
        // Check mode: compare generated content with existing file
        let generated_content =
            export_bicep_document_to_json_string(&document, pretty, common.exclude_empty)?;
        check_file_diff(&generated_content, &output_path)?;
    } else {
        // Normal mode: export the document
        export_bicep_document_to_json(&document, &output_path, pretty, common.exclude_empty)?;
        debug!("JSON exported to: {}", output_path.display());
        if pretty {
            debug!("Output is formatted with indentation");
        }
    }

    Ok(())
}

/// Handle the Markdown export command
fn handle_markdown_export(common: CommonExportOptions) -> Result<(), Box<dyn Error>> {
    handle_export(
        common,
        "md",
        |doc, path, emoji, exclude_empty| {
            export_bicep_document_to_markdown(doc, path, emoji, exclude_empty)
        },
        |doc, emoji, exclude_empty| {
            export_bicep_document_to_markdown_string(doc, emoji, exclude_empty)
        },
    )
}

/// Handle the AsciiDoc export command
fn handle_asciidoc_export(common: CommonExportOptions) -> Result<(), Box<dyn Error>> {
    handle_export(
        common,
        "adoc",
        |doc, path, emoji, exclude_empty| {
            export_bicep_document_to_asciidoc(doc, path, emoji, exclude_empty)
        },
        |doc, emoji, exclude_empty| {
            export_bicep_document_to_asciidoc_string(doc, emoji, exclude_empty)
        },
    )
}

/// Configure the tracing subscriber based on command line options
fn setup_tracing(verbose: u8, quiet: bool, log_format: LogFormat, log_file: Option<PathBuf>) {
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

    let subscriber_builder = tracing_subscriber::registry().with(filter);

    // Configure formatting and output based on user preferences
    match log_file {
        Some(path) => {
            let file = File::create(path).expect("Unable to create log file");
            match log_format {
                LogFormat::Json => {
                    subscriber_builder
                        .with(
                            fmt::Layer::default()
                                .json()
                                .with_writer(file)
                                .with_target(true)
                                .with_span_events(FmtSpan::CLOSE),
                        )
                        .init();
                },
                LogFormat::Text => {
                    subscriber_builder
                        .with(
                            fmt::Layer::default()
                                .with_writer(file)
                                .with_target(true)
                                .with_span_events(FmtSpan::CLOSE)
                                .with_timer(fmt::time::time()),
                        )
                        .init();
                },
            }
        },
        None => {
            match log_format {
                LogFormat::Json => {
                    subscriber_builder
                        .with(
                            fmt::Layer::default()
                                .json()
                                .with_writer(std::io::stdout)
                                .with_target(true)
                                .with_span_events(FmtSpan::CLOSE),
                        )
                        .init();
                },
                LogFormat::Text => {
                    subscriber_builder
                        .with(
                            fmt::Layer::default()
                                .with_writer(std::io::stderr) // Or stdout, depending on preference for text logs
                                .with_target(true)
                                .with_span_events(FmtSpan::CLOSE)
                                .with_timer(fmt::time::time()),
                        )
                        .init();
                },
            }
        },
    }
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // Setup tracing with the appropriate verbosity and format
    setup_tracing(cli.verbose, cli.quiet, cli.log_format, cli.log_file);

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
        Commands::Markdown { common } => handle_markdown_export(common),
        Commands::Asciidoc { common } => handle_asciidoc_export(common),
    };

    if let Err(ref e) = result {
        error!("Command failed: {}", e);
    } else {
        debug!("Command completed successfully");
    }

    result
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[test]
    fn test_exclude_empty_flag_parsing() {
        // Test without exclude_empty flag
        let args = vec!["bicep-docs", "markdown", "input.bicep"];
        let cli = Cli::parse_from(args);

        if let Commands::Markdown { common } = cli.command {
            assert!(!common.exclude_empty);
            assert!(!common.check);
        } else {
            panic!("Expected Markdown command");
        }

        // Test with exclude_empty flag
        let args = vec!["bicep-docs", "markdown", "--exclude-empty", "input.bicep"];
        let cli = Cli::parse_from(args);

        if let Commands::Markdown { common } = cli.command {
            assert!(common.exclude_empty);
            assert!(!common.check);
        } else {
            panic!("Expected Markdown command");
        }
    }

    #[test]
    fn test_check_flag_parsing() {
        // Test with check flag
        let args = vec!["bicep-docs", "markdown", "--check", "input.bicep"];
        let cli = Cli::parse_from(args);

        if let Commands::Markdown { common } = cli.command {
            assert!(common.check);
            assert!(!common.exclude_empty);
        } else {
            panic!("Expected Markdown command");
        }

        // Test with both check and exclude_empty flags
        let args = vec![
            "bicep-docs",
            "markdown",
            "--check",
            "--exclude-empty",
            "input.bicep",
        ];
        let cli = Cli::parse_from(args);

        if let Commands::Markdown { common } = cli.command {
            assert!(common.check);
            assert!(common.exclude_empty);
        } else {
            panic!("Expected Markdown command");
        }
    }
}
