// Copyright (c) ZeroC, Inc. All rights reserved.

use clap::ArgAction::Append;
use clap::{Parser, ValueEnum};
use serde::Serialize;
use std::path::Path;

// Note: clap uses the doc-comments of fields to populate the '--help' output of slice-xxx.
//       boolean flags automatically default to false, and strings automatically default to empty.

/// This struct is responsible for parsing the command line options common to all slice compilers.
/// The option parsing capabilities are generated on the struct by the `clap` macro.
#[derive(Debug, Default, Parser)]
#[command(rename_all = "kebab-case")]
pub struct SliceOptions {
    /// List of slice files to compile.
    #[arg(required = true, value_parser = is_valid_source)]
    pub sources: Vec<String>,

    /// Files that are needed for referencing, but that no code should be generated for.
    #[arg(short = 'R', long, num_args = 1, action = Append, value_parser = is_valid_reference)]
    pub references: Vec<String>,

    /// Preprocessor Symbols defined on the command line.
    #[arg(short = 'D', long, num_args = 1, action = Append)]
    pub definitions: Vec<String>,

    /// Instructs the compiler to treat warnings as errors.
    #[arg(short, long)]
    pub warn_as_error: bool,

    // Instructs the compiler to ignore warnings. Specify a list of warnings to ignore, or leave empty to ignore all
    // warnings.
    #[arg(long)]
    pub ignore_warnings: Option<Vec<String>>,

    /// Validates input files without generating code for them.
    #[arg(long)]
    pub dry_run: bool,

    /// Output directory for generated code, defaults to the current working directory.
    #[arg(long)]
    pub output_dir: Option<String>,

    /// Output format for emitted errors,
    #[arg(value_enum, default_value_t = DiagnosticFormat::Human, long, ignore_case = true)]
    pub diagnostic_format: DiagnosticFormat,

    /// Disables ANSI escape code for diagnostic output.
    #[arg(long)]
    pub disable_color: bool,
}

const SLICE_FILE_EXTENSION: &str = "slice";

fn is_valid_source(s: &str) -> Result<String, String> {
    match Path::new(s).extension() {
        Some(extension) if extension == SLICE_FILE_EXTENSION => Ok(s.to_owned()),
        _ => Err("Slice files must end with a .slice extension".to_owned()),
    }
}

fn is_valid_reference(s: &str) -> Result<String, String> {
    if Path::new(s).is_file() {
        // The user supplied a file, need to check if it ends with '.slice'.
        is_valid_source(s)
    } else {
        // The user supplied a directory, no checks needed.
        Ok(s.to_owned())
    }
}

/// This enum is used to specify the format for emitted diagnostics.
///
/// # Variants
/// * Human - Any emitted diagnostics will be printed to the console with an easily readable format.
/// * Json - Any emitted diagnostics will be serialized as JSON objects and printed to the console.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, ValueEnum)]
pub enum DiagnosticFormat {
    Human,
    Json,
}

impl Default for DiagnosticFormat {
    fn default() -> Self {
        DiagnosticFormat::Human
    }
}
