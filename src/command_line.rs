// Copyright (c) ZeroC, Inc. All rights reserved.

use clap::{Parser, ValueEnum};
use serde::Serialize;
use std::path::Path;

// Note: clap uses the doc-comments of fields to populate the '--help' output of slice-xxx.
//       boolean flags automatically default to false, and strings automatically default to empty.

/// This struct is responsible for parsing the command line options common to all slice compilers.
/// The option parsing capabilities are generated on the struct by the `clap` macro.
#[derive(Parser, Debug)]
#[clap(rename_all = "kebab-case")] // Each compiler sets its own `about` message.
pub struct SliceOptions {
    /// List of slice files to compile.
    #[clap(required = true, value_parser = file_is_slice)]
    // TODO: Add validation that the file is a .slice file
    pub sources: Vec<String>,

    /// Files that are needed for referencing, but that no code should be generated for.
    #[clap(short = 'R', long, number_of_values = 1, multiple = true, value_parser = file_is_slice)]
    // TODO: Add validation that the file is a .slice file
    pub references: Vec<String>,

    /// Instructs the compiler to treat warnings as errors.
    #[clap(short, long, value_parser)]
    pub warn_as_error: bool,

    /// Validates input files without generating code for them.
    #[clap(long, value_parser)]
    pub validate: bool,

    /// Output directory for generated code, defaults to the current working directory.
    #[clap(long, value_parser)]
    pub output_dir: Option<String>,

    /// Output format for emitted errors,
    #[clap(arg_enum, case_insensitive = true, default_value_t = DiagnosticFormat::Human, long, value_parser)]
    pub diagnostic_format: DiagnosticFormat,

    /// Disables ANSI escape code for diagnostic output.
    #[clap(long, value_parser)]
    pub disable_color: bool,
}

const SLICE_FILE_EXTENSION: &str = ".slice";

fn file_is_slice(filename: &str) -> Result<String, String> {
    if let Some(extension) = Path::new(filename).extension().and_then(|f| f.to_str()).to_owned() {
        if extension == SLICE_FILE_EXTENSION {
            return Ok(filename.to_owned());
        }
    }
    Err(format!("{} is not a .slice file", filename))
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
