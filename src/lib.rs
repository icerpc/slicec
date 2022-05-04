// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod ast;
pub mod code_gen_util;
pub mod command_line;
pub mod error;
pub mod grammar;
pub mod parser;
pub mod ptr_util;
pub mod ptr_visitor;
pub mod slice_file;
pub mod supported_encodings;
pub mod validator;
pub mod visitor;

use crate::ast::Ast;
use crate::command_line::SliceOptions;
use crate::error::{Error, ErrorLevel, ErrorReporter};
use crate::parser::parse_string;
use crate::slice_file::SliceFile;
use crate::validator::Validator;
use std::collections::HashMap;

pub fn parse_from_options(options: &SliceOptions) -> Result<(Ast, ErrorReporter, HashMap<String, SliceFile>), Error> {
    let mut ast = Ast::new();
    let mut error_reporter = ErrorReporter::default();

    let slice_files = parser::parse_files(options, &mut ast, &mut error_reporter)?;
    handle_errors(options.warn_as_error, &slice_files, &mut error_reporter)?;

    let mut validator = Validator { error_reporter: &mut error_reporter };
    for slice_file in slice_files.values() {
        slice_file.visit_with(&mut validator);
    }
    validator.validate_dictionary_key_types(&ast);

    Ok((ast, error_reporter, slice_files))
}

pub fn parse_from_string(input: &str) -> Result<(Ast, ErrorReporter), Error> {
    let mut ast = Ast::new();
    let mut error_reporter = ErrorReporter::default();

    let slice_files = parse_string(input, &mut ast, &mut error_reporter)?;

    let mut validator = Validator { error_reporter: &mut error_reporter };

    for slice_file in slice_files.values() {
        slice_file.visit_with(&mut validator);
    }
    validator.validate_dictionary_key_types(&ast);

    Ok((ast, error_reporter))
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
        Err(Error{
            message,
            location: None,
            severity: ErrorLevel::Critical,
        })
    } else {
        Ok(())
    }
}
