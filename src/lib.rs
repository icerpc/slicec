// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod ast;
pub mod code_block;
pub mod command_line;
pub mod compilation_result;
pub mod diagnostics;
pub mod grammar;
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
use crate::compilation_result::CompilationResult;
use crate::validators::validate_compilation_data;

pub fn parse_from_options(options: &SliceOptions) -> CompilationResult {
    parser::parse_files(options).and_then(validate_compilation_data)
}

pub fn parse_from_strings(inputs: &[&str], option: Option<SliceOptions>) -> CompilationResult {
    parser::parse_strings(inputs, option).and_then(validate_compilation_data)
}
