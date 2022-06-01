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
use crate::error::{Error, ErrorLevel, ErrorReporter};
use crate::parse_result::ParserResult;
use crate::slice_file::SliceFile;
use crate::validators::Validator;
use std::collections::HashMap;

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

pub fn handle_errors(
    warn_as_error: bool,
    slice_files: &HashMap<String, SliceFile>,
    error_reporter: &mut ErrorReporter,
) -> Result<(), Error> {
    error_reporter.print_errors(slice_files);
    if error_reporter.has_errors(warn_as_error) {
        let counts = error_reporter.get_totals();
        let message = format!(
            "Compilation failed with {} error(s) and {} warning(s).\n",
            counts.0, counts.1
        );

        println!("{}", &message);
        Err(Error { message, location: None, severity: ErrorLevel::Critical })
    } else {
        Ok(())
    }
}
