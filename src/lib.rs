// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod ast;
pub mod errors;
pub mod grammar;
pub mod parser;
pub mod utils;
pub mod validators;

use crate::validators::validate_parsed_data;
use utils::command_line::SliceOptions;
use utils::parse_result::ParserResult;

pub fn parse_from_options(options: &SliceOptions) -> ParserResult {
    parser::parse_files(options).and_then(validate_parsed_data)
}

pub fn parse_from_string(input: &str) -> ParserResult {
    parser::parse_string(input).and_then(validate_parsed_data)
}

pub fn parse_from_strings(inputs: &[&str]) -> ParserResult {
    parser::parse_strings(inputs).and_then(validate_parsed_data)
}
