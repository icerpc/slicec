// Copyright (c) ZeroC, Inc. All rights reserved.

use structopt::StructOpt;

// Note: StructOpt automatically uses the doc-comments of fields to populate the '--help' output of slice-xxx.
//       boolean flags automatically default to false, and strings automatically default to empty.

// TODO revisit these names, and add more options!

//------------------------------------------------------------------------------
// SliceOptions
//------------------------------------------------------------------------------
/// This struct is responsible for parsing the command line options common to all slice compilers.
/// The option parsing capabilities are automatically generated for the struct by the `StructOpt` crate.
#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case", about = "_")] // We don't set `about`, since each compiler will set their own.
pub struct SliceOptions {
    /// List of slice files to compile.
    pub sources: Vec<String>,

    /// Files that are needed for referencing, but that no code should be generated for.
    #[structopt(short = "R", long)]
    pub references: Vec<String>,

    /// Prints additional debugging information to the console.
    #[structopt(short, long)]
    pub debug: bool,

    /// Instructs the compiler to treat warnings as errors.
    #[structopt(short, long)]
    pub warn_as_error: bool,

    /// Validates input files without generating code for them.
    #[structopt(long)]
    pub dry_run: bool,
}
