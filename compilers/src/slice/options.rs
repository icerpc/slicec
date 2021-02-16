
use structopt::StructOpt;

// This struct is responsible for parsing the command line options common to all slice compilers.
// Note: StructOpt automatically uses doc-comments to populate the '--help' output of slice-cs.
#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case")]
pub struct SliceOptions {
    /// List of slice files to compile.
    pub inputs: Vec<String>,

    /// Files that are needed for referencing, but that no code should be generated for.
    #[structopt(short = "I", long)]
    pub includes: Vec<String>,

    /// Prints additional debugging information to the console.
    #[structopt(short, long)]
    pub debug: bool,

    /// Validates input files without generating anything.
    #[structopt(long)]
    pub dry_run: bool,
}
