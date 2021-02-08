
mod parser;
mod definitions;
mod cs_generator;

use std::fs;

fn main() {
    let slice_file = fs::read_to_string("test.ice").expect("Wheres the file?!");
    let tree = parser::parse(&slice_file).unwrap();
    //cs_generator::generate_from(tree);
}
