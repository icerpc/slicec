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
    match parser::parse_files(options) {
        Ok(mut data) => {
            let mut validator = Validator::new(&mut data.error_reporter, &data.ast);
            validator.validate(&data.files);
            data.into()
        }
        Err(data) => Err(data),
    }
}

pub fn parse_from_string(input: &str) -> ParserResult {
    match parser::parse_string(input) {
        Ok(mut data) => {
            let mut validator = Validator::new(&mut data.error_reporter, &data.ast);
            validator.validate(&data.files);
            data.into()
        }
        Err(data) => Err(data),
    }
}
