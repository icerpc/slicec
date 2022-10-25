// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod ast;
pub mod code_block;
pub mod command_line;
pub mod diagnostics;
pub mod grammar;
pub mod parse_result;
pub mod parser;
pub mod parsers;
pub mod slice_file;
pub mod supported_encodings;
pub mod utils;
pub mod validators;
pub mod visitor;

// Re-export the `clap` and `convert_case` dependencies.
pub extern crate clap;
pub extern crate convert_case;

use crate::command_line::SliceOptions;
use crate::parse_result::ParserResult;
use crate::validators::validate_parsed_data;

pub fn parse_from_options(options: &SliceOptions) -> ParserResult {
    parser::parse_files(options).and_then(validate_parsed_data)
}

pub fn parse_from_strings(inputs: &[&str], option: Option<SliceOptions>) -> ParserResult {
    parser::parse_strings(inputs, option).and_then(validate_parsed_data)
}
