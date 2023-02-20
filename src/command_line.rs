// Copyright (c) ZeroC, Inc.

use clap::ArgAction::Append;
use clap::{Parser, ValueEnum};
use serde::Serialize;

// Note: clap uses the doc-comments of fields to populate the '--help' output of slice-xxx.
//       boolean flags automatically default to false, and strings automatically default to empty.

/// This struct is responsible for parsing the command line options common to all slice compilers.
/// The option parsing capabilities are generated on the struct by the `clap` macro.
#[derive(Debug, Default, Parser)]
#[command(rename_all = "kebab-case")]
pub struct SliceOptions {
    /// List of slice files to compile.
    #[arg(required = true)]
    pub sources: Vec<String>,

    /// Files that are needed for referencing, but that no code should be generated for.
    #[arg(short = 'R', long, num_args = 1, action = Append)]
    pub references: Vec<String>,

    /// Preprocessor Symbols defined on the command line.
    #[arg(short = 'D', long, num_args = 1, action = Append)]
    pub definitions: Vec<String>,

    /// Instructs the compiler to treat warnings as errors.
    #[arg(short, long)]
    pub warn_as_error: bool,

    /// Instructs the compiler to allow warnings. Specify a list of warnings to allow, or leave empty to allow all
    /// warnings.
    #[arg(long)]
    pub allow_warnings: Option<Vec<String>>,

    /// Validates input files without generating code for them.
    #[arg(long)]
    pub dry_run: bool,

    /// Output directory for generated code, defaults to the current working directory.
    #[arg(long)]
    pub output_dir: Option<String>,

    /// Output format for emitted errors.
    #[arg(value_enum, default_value_t = DiagnosticFormat::Human, long, ignore_case = true)]
    pub diagnostic_format: DiagnosticFormat,

    /// Disables ANSI escape code for diagnostic output.
    #[arg(long)]
    pub disable_color: bool,
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
