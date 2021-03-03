
use structopt::StructOpt;
use slice::options::SliceOptions;

// Note: StructOpt automatically uses the doc-comments of fields to populate the '--help' output of slice-cs.
//       boolean flags automatically default to false, and strings automatically default to empty.

//------------------------------------------------------------------------------
// CsOptions
//------------------------------------------------------------------------------
/// This struct is responsible for parsing the command line options specified to the 'slice-cs' compiler.
/// The option parsing capabilities are automatically generated for the struct by the `StructOpt` crate.
#[derive(StructOpt, Debug)]
#[structopt(name = "slice-cs", version = "0.1.0", rename_all = "kebab-case")]
pub struct CsOptions {
    // Options only relevant to slice-cs.
    // TODO add more options!

    // Import the options common to all slice compilers.
    #[structopt(flatten)]
    pub slice_options: SliceOptions,
}
