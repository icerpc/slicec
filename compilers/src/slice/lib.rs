
pub mod ast;
pub mod error;
pub mod grammar;
pub mod options;
pub mod util;
pub mod visitor;
mod parser;

//struct CompilerData {
//    slice_files: HashMap<String, SliceFile>,
//    ast: SliceAst,
//    errors: Vec<SliceError>,
//    error_count: usize,
//    warning_count: usize,
//}
//
//pub fn parse_from_options(options: &SliceOptions) -> Result<CompilerData {
//
//}


// We need to add support for passing directories to the slice compiler!
// We need to do this BEFORE we pass the options into the Slice Parser, as it expects nothing but files!
// It probably makes the most sense to add this functionality into the `options` module.


// Implement the following compiler flags!
// pub sources: Vec<String>,
// pub references: Vec<String>,
// pub debug: bool,
// pub warn_as_error: bool,
// pub dry_run: bool,

use structopt::StructOpt;

pub fn temp() -> ast::SliceAst {
    let o = options::SliceOptions::from_args();
    let result = parser::SliceParser::parse_files(&o);
    result.0
}
