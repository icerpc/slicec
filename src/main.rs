
extern crate pest_consume;

use pest_consume::Parser;
use std::fs;

#[derive(Parser)]
#[grammar = "slice.pest"]
pub struct SliceParser;

#[pest_consume::parser]
impl SliceParser {

}

fn main() {
    let slice_file = fs::read_to_string("test.ice").expect("Wheres the file?!");

    let parsed_file = SliceParser::parse(Rule::main, &slice_file);
    let result = parsed_file.unwrap_or_else(|e| panic!("{}", e));
    print!("{:?}", result);
}
