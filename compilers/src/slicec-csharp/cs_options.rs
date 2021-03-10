// Copyright (c) ZeroC, Inc. All rights reserved.

use structopt::StructOpt;
use slice::options::SliceOptions;

// Note: StructOpt automatically uses the doc-comments of fields to populate the '--help' output of slicec-csharp.
//       boolean flags automatically default to false, and strings automatically default to empty.

//------------------------------------------------------------------------------
// CsOptions
//------------------------------------------------------------------------------
/// This struct is responsible for parsing the command line options specified to the 'slicec-csharp' compiler.
/// The option parsing capabilities are automatically generated for the struct by the `StructOpt` crate.
#[derive(StructOpt, Debug)]
#[structopt(name = "slicec-csharp", version = "0.1.0", rename_all = "kebab-case", about = ABOUT_MESSAGE)]
pub struct CsOptions {
    // Options only relevant to slicec-csharp.
    // TODO add more options!

    // Import the options common to all slice compilers.
    #[structopt(flatten)]
    pub slice_options: SliceOptions,
}

/// Short description of slicec-csharp that is displayed in it's help dialogue.
const ABOUT_MESSAGE: &str = "The slice compiler for C#.\nGenerates C# code for use with icerpc from slice files.";
