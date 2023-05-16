// Copyright (c) ZeroC, Inc.

use clap::ArgAction::Append;
use clap::{Parser, ValueEnum};
use serde::Serialize;

use crate::compilation_state::CompilationStatePatcher;

// Note: clap uses the doc-comments of fields to populate the '--help' output of slice-xxx.
//       boolean flags automatically default to false, and strings automatically default to empty.

/// This struct is responsible for parsing the command line options common to all slice compilers.
/// The option parsing capabilities are generated on the struct by the `clap` macro.
#[derive(Default, Parser)]
#[command(rename_all = "kebab-case")]
pub struct SliceOptions {
    /// List of Slice files to compile.
    #[arg(required = true)]
    pub sources: Vec<String>,

    /// Add a directory or Slice file to the list of references.
    #[arg(short = 'R', value_name="REFERENCE", num_args = 1, action = Append)]
    pub references: Vec<String>,

    /// Define a preprocessor definition.
    #[arg(short = 'D', value_name="DEFINITION", num_args = 1, action = Append)]
    pub definitions: Vec<String>,

    /// Instruct the compiler to treat warnings as errors.
    #[arg(short = 'W', long)]
    pub warn_as_error: bool,

    /// Instruct the compiler to allow (not emit) the specified warning.
    #[arg(short = 'A', long = "allow", value_name="WARNING", num_args = 1, action = Append)]
    pub allowed_warnings: Vec<String>,

    /// Validate input files without generating code for them.
    #[arg(long)]
    pub dry_run: bool,

    /// Set the output directory for the generated code. Defaults to the current working directory.
    #[arg(short = 'O', long)]
    pub output_dir: Option<String>,

    /// Set the output format for emitted errors.
    #[arg(value_enum, default_value_t = DiagnosticFormat::Human, long, ignore_case = true)]
    pub diagnostic_format: DiagnosticFormat,

    /// Disable ANSI color codes in diagnostic output.
    #[arg(long)]
    pub disable_color: bool,

    #[arg(skip)]
    pub state_patcher: Option<CompilationStatePatcher>,
}

/// This enum is used to specify the format for emitted diagnostics.
///
/// # Variants
/// * Human - Any emitted diagnostics will be printed to the console with an easily readable format.
/// * Json - Any emitted diagnostics will be serialized as JSON objects and printed to the console.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, ValueEnum)]
pub enum DiagnosticFormat {
    #[default]
    Human,
    Json,
}
