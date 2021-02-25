
pub mod ast;
pub mod error;
pub mod grammar;
pub mod util;
pub mod visitor;

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

// TODO add a note about how we have to convert all the \r\n and \r into \n
// TODO add a 'warn-as-error' flag.
