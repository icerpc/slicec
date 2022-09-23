// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod ast;
pub mod code_block;
pub mod command_line;
pub mod diagnostics;
pub mod grammar;
pub mod parse_result;
pub mod parser;
pub mod slice_file;
pub mod supported_encodings;
pub mod utils;
pub mod validators;
pub mod visitor;

use crate::command_line::SliceOptions;
use crate::parse_result::ParserResult;
use crate::validators::validate_parsed_data;

pub fn parse_from_options(options: &SliceOptions) -> ParserResult {
    parser::parse_files(options).and_then(validate_parsed_data)
}

pub fn parse_from_string(input: &str) -> ParserResult {
    parser::parse_string(input, None).and_then(validate_parsed_data)
}

pub fn parse_from_string_with_options(input: &str, options: SliceOptions) -> ParserResult {
    parser::parse_string(input, Some(options)).and_then(validate_parsed_data)
}

pub fn parse_from_strings(inputs: &[&str]) -> ParserResult {
    parser::parse_strings(inputs).and_then(validate_parsed_data)
}
