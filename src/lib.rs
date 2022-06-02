// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod ast;
pub mod code_gen_util;
pub mod command_line;
pub mod error;
pub mod grammar;
pub mod parse_result;
pub mod parser;
pub mod ptr_util;
pub mod ptr_visitor;
pub mod slice_file;
pub mod supported_encodings;
pub mod validators;
pub mod visitor;

use crate::command_line::SliceOptions;
use crate::parse_result::ParserResult;
use crate::validators::Validator;

pub fn parse_from_options(options: &SliceOptions) -> ParserResult {
    parser::parse_files(options).and_then(|mut data| {
        let mut validator = Validator::new(&mut data.error_reporter);
        validator.validate(&data.files, &data.ast);
        data.into()
    })
}

pub fn parse_from_string(input: &str) -> ParserResult {
    parser::parse_string(input).and_then(|mut data| {
        let mut validator = Validator::new(&mut data.error_reporter);
        validator.validate(&data.files, &data.ast);
        data.into()
    })
}
