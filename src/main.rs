
mod parser;
mod definitions;
mod visitor;
mod cs_generator;

use visitor::Visitor;
use std::io::Write;
use std::fs::{read_to_string, File};

fn main() {
    let slice_file = read_to_string("test.ice").expect("Wheres the file?!");
    let tree = parser::parse(&slice_file).unwrap();

    let mut generator = cs_generator::CsGenerator { output: "".to_owned() };
    generator.generate_from(&tree);

    let mut output_file = File::create("result.cs").expect("Unable to create file!!");
    output_file.write_all(generator.output.as_bytes()).expect("Failed to write!!");
}
