
pub mod grammar;
pub mod options;
pub mod parser;
pub mod util;
pub mod visitor;
pub mod writer;
mod consumer;
mod tree_patcher;

use options::SliceOptions;
use parser::{SliceAst, SliceFile};
use util::SliceError;

pub type ParseResult = Result<(SliceAst, Vec<SliceFile>), SliceError>;

pub fn parse(opts: &SliceOptions) -> ParseResult {
    // we parse everything here and combine them into one big tree
    unimplemented!() //TODO
}
