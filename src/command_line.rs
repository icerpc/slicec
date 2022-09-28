// Copyright (c) ZeroC, Inc. All rights reserved.

use clap::{Parser, ValueEnum};
use serde::Serialize;
use std::path::Path;

// Note: clap uses the doc-comments of fields to populate the '--help' output of slice-xxx.
//       boolean flags automatically default to false, and strings automatically default to empty.

/// This struct is responsible for parsing the command line options common to all slice compilers.
/// The option parsing capabilities are generated on the struct by the `clap` macro.
#[derive(Debug, Parser)]
#[clap(rename_all = "kebab-case")] // Each compiler sets its own `about` message.
pub struct SliceOptions {
    /// List of slice files to compile.
    #[clap(required = true, value_parser = is_valid_source)]
    pub sources: Vec<String>,

    /// Files that are needed for referencing, but that no code should be generated for.
    #[clap(short = 'R', long, number_of_values = 1, multiple = true, value_parser = is_valid_reference)]
    pub references: Vec<String>,

    /// Instructs the compiler to treat warnings as errors.
    #[clap(short, long, action)]
    pub warn_as_error: bool,

    /// Validates input files without generating code for them.
    #[clap(long, action)]
    pub validate: bool,

    /// Output directory for generated code, defaults to the current working directory.
    #[clap(long)]
    pub output_dir: Option<String>,

    /// Output format for emitted errors,
    #[clap(arg_enum, case_insensitive = true, default_value_t = DiagnosticFormat::Human, long)]
    pub diagnostic_format: DiagnosticFormat,

    /// Disables ANSI escape code for diagnostic output.
    #[clap(long, action)]
    pub disable_color: bool,
}

const SLICE_FILE_EXTENSION: &str = "slice";

fn is_valid_source(s: &str) -> Result<String, String> {
    match Path::new(s).extension() {
        Some(extension) if extension == SLICE_FILE_EXTENSION => Ok(s.to_owned()),
        _ => Err("slice files must end with a .slice extension".to_owned()),
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
