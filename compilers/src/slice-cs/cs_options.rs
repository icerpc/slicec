
use structopt::StructOpt;
use slice::options::SliceOptions;

// This struct is responsible for parsing the command line options provided to slice-cs.
// Note: StructOpt automatically uses doc-comments to populate the '--help' output of slice-cs.
#[derive(StructOpt, Debug)]
#[structopt(name = "slice-cs", version = "0.1.0", rename_all = "kebab-case")]
pub struct CsOptions {
    // Options only relevant to slice-cs.
    // ...

    // Import the options common to all slice compilers.
    #[structopt(subcommand)]
    pub slice_options: SliceOptions,
}
