
extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;
use std::fs;

#[derive(Parser)]
#[grammar = "slice.pest"]
pub struct SliceParser;

fn main() {
    let slice_file = fs::read_to_string("test.ice").expect("Wheres the file?!");

    let parsed_file = SliceParser::parse(Rule::main, &slice_file);
    let result = parsed_file.unwrap_or_else(|e| panic!("{}", e));
    print!("{:?}", result);
}
