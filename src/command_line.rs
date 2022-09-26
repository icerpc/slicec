// Copyright (c) ZeroC, Inc. All rights reserved.

use serde::Serialize;
use structopt::clap::arg_enum;
use structopt::StructOpt;

// Note: StructOpt uses the doc-comments of fields to populate the '--help' output of slice-xxx.
//       boolean flags automatically default to false, and strings automatically default to empty.

/// This struct is responsible for parsing the command line options common to all slice compilers.
/// The option parsing capabilities are generated on the struct by the `StructOpt` macro.
#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")] // Each compiler sets its own `about` message.
pub struct SliceOptions {
    /// List of slice files to compile.
    #[structopt(required = true)]
    pub sources: Vec<String>,

    /// Files that are needed for referencing, but that no code should be generated for.
    #[structopt(short = "R", long, number_of_values = 1, multiple = true)]
    pub references: Vec<String>,

    /// Prints additional debugging information to the console.
    #[structopt(short, long)]
    pub debug: bool,

    /// Instructs the compiler to treat warnings as errors.
    #[structopt(short, long)]
    pub warn_as_error: bool,

    /// Validates input files without generating code for them.
    #[structopt(long)]
    pub validate: bool,

    /// Output directory for generated code, defaults to the current working directory.
    #[structopt(long)]
    pub output_dir: Option<String>,

    /// Output format for emitted errors,
    #[structopt(case_insensitive = true, default_value = "human", long, possible_values = &["human", "json"])]
    pub diagnostic_format: DiagnosticFormat,
}

arg_enum! {

    /// This enum is used to specify the format for emitted diagnostics.
    ///
    /// # Variants
    /// * Human - Any emitted diagnostics will be printed to the console with an easily readable format.
    /// * Json - Any emitted diagnostics will be serialized as JSON objects and printed to the console.
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
    pub enum DiagnosticFormat {
        Human,
        Json,
    }
}
