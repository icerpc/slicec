// Copyright (c) ZeroC, Inc.

use crate::diagnostics::Lint;
use clap::ArgAction::Append;
use clap::{Parser, ValueEnum};

// Note: clap uses the doc-comments of fields to populate the '--help' output of slicec-xxx.
//       boolean flags automatically default to false, and strings automatically default to empty.

/// This struct is responsible for parsing the command line options common to all slice compilers.
/// The option parsing capabilities are generated on the struct by the `clap` macro.
#[derive(Debug, Default, Parser)]
#[command(rename_all = "kebab-case")]
pub struct SliceOptions {
    /// List of Slice files to compile.
    #[arg(required = true)]
    pub sources: Vec<String>,

    /// Add a directory or Slice file to the list of references.
    #[arg(short = 'R', num_args = 1, action = Append, value_name = "REFERENCE")]
    pub references: Vec<String>,

    /// Define a preprocessor symbol.
    #[arg(short = 'D', num_args = 1, action = Append, value_name = "SYMBOL")]
    pub defined_symbols: Vec<String>,

    /// Instruct the compiler to allow the specified lint.
    // TODO add a link to the lint reference in this doc comment!
    #[arg(short = 'A', long = "allow", num_args = 1, action = Append, value_name = "LINT_NAME", value_parser = Lint::ALLOWABLE_LINT_IDENTIFIERS, hide_possible_values = true, ignore_case = true)]
    pub allowed_lints: Vec<String>,

    /// Validate input files without generating code for them.
    #[arg(long)]
    pub dry_run: bool,

    /// Set the output directory for the generated code. Defaults to the current working directory.
    #[arg(short = 'O', long, value_name = "DIRECTORY")]
    pub output_dir: Option<String>,

    /// Specify how the compiler should emit errors and warnings.
    #[arg(long, value_name = "FORMAT", value_enum, default_value_t = DiagnosticFormat::Human, ignore_case = true)]
    pub diagnostic_format: DiagnosticFormat,

    /// Disable ANSI color codes in diagnostic output.
    #[arg(long)]
    pub disable_color: bool,
}

/// This enum is used to specify the format for emitted diagnostics.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum DiagnosticFormat {
    /// Diagnostics are printed to the console in an easily readable format with source code snippets when possible.
    #[default]
    Human,

    /// Diagnostics will be serialized as JSON objects and printed to the console, one diagnostic per line.
    Json,
}
