use bicep_docs::{export_bicep_document_to_json, export_bicep_document_to_yaml};
use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process;

fn print_usage() {
    println!("Bicep Documentation Generator");
    println!("Usage: bicep-docs <command> [options]");
    println!();
    println!("Commands:");
    println!("  export     Export Bicep file to YAML or JSON format");
    println!();
    println!("For export command:");
    println!(
        "  bicep-docs export <bicep_file> --format=<yaml|json> [--output=<output_path>] [--pretty]"
    );
    println!();
    println!("Options:");
    println!("  --format    Output format (yaml or json)");
    println!("  --output    Output file path (default is input filename with new extension)");
    println!("  --pretty    Format JSON with indentation (only for JSON output)");
}

fn handle_export_command(args: &[String]) -> Result<(), Box<dyn Error>> {
    if args.is_empty() {
        println!("Error: Missing Bicep file path");
        print_usage();
        process::exit(1);
    }

    let bicep_file = &args[0];
    let mut format = "yaml";
    let mut output_path = None;
    let mut pretty = false;

    for arg in &args[1..] {
        if arg.starts_with("--format=") {
            format = &arg["--format=".len()..];
            if format != "yaml" && format != "json" {
                println!("Error: Invalid format. Must be 'yaml' or 'json'");
                process::exit(1);
            }
        } else if arg.starts_with("--output=") {
            output_path = Some(&arg["--output=".len()..]);
        } else if arg == "--pretty" {
            pretty = true;
        }
    }

    // Read the Bicep file
    let source_code = fs::read_to_string(bicep_file)?;

    // Parse the Bicep file
    let document = bicep_docs::parse_bicep_document(&source_code)?;

    // Determine output path if not provided
    let output = if let Some(path) = output_path {
        Path::new(path).to_path_buf()
    } else {
        let input_path = Path::new(bicep_file);
        let file_stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");

        match format {
            "yaml" => Path::new(file_stem).with_extension("yaml"),
            "json" => Path::new(file_stem).with_extension("json"),
            _ => Path::new(file_stem).with_extension("yaml"),
        }
    };

    // Export the document
    match format {
        "yaml" => {
            export_bicep_document_to_yaml(&document, &output)?;
            println!("YAML exported to: {}", output.display());
        },
        "json" => {
            export_bicep_document_to_json(&document, &output, pretty)?;
            println!("JSON exported to: {}", output.display());
            if pretty {
                println!("Output is formatted with indentation.");
            }
        },
        _ => unreachable!(),
    }

    Ok(())
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "export" => handle_export_command(&args[2..])?,
        "--help" | "-h" => print_usage(),
        _ => {
            println!("Error: Unknown command '{}'", args[1]);
            print_usage();
            process::exit(1);
        },
    }

    Ok(())
}
