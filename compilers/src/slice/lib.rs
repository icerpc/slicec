
pub mod grammar;
pub mod options;
pub mod util;
pub mod visitor;
pub mod writer;
mod parser;
mod type_patcher;

use options::SliceOptions;
use util::ParseResult;

pub fn parse(opts: &SliceOptions) -> ParseResult {
    // we parse everything here and combine them into one big tree
}
