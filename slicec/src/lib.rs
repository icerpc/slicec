// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod ast;
pub mod error;
pub mod grammar;
pub mod options;
pub mod util;
pub mod visitor;
pub mod writer;
mod parser;
mod patchers;
mod validator;

use crate::ast::Ast;
use crate::error::ErrorHandler;
use crate::options::SliceOptions;
use crate::parser::SliceParser;
use crate::patchers::{ScopePatcher, TypePatcher};
use crate::util::SliceFile;
use crate::validator::Validator;
use std::collections::HashMap;

#[derive(Debug)]
pub struct CompilerData {
    pub ast: Ast,
    pub slice_files: HashMap<String, SliceFile>,
    pub error_handler: ErrorHandler,
    pub constructed_table: HashMap<String, usize>,
}

pub fn parse_from_options(options: &SliceOptions) -> Result<CompilerData, ()> {
    // Parse the slice files from the command line input into an unpatched AST.
    let (mut ast, slice_files, mut error_handler) = SliceParser::parse_files(&options);
    handle_errors(options.warn_as_error, &mut error_handler, &slice_files)?;

    // Patch the scopes in the AST in-place, and use them to generate a lookup table for use-defined types.
    let mut scope_patcher = ScopePatcher::new(&mut error_handler);
    scope_patcher.patch_scopes(&slice_files, &mut ast);
    let constructed_table = scope_patcher.into_lookup_table(&ast);
    // Patch the type references in the AST in-place.
    TypePatcher::new(&mut error_handler).patch_types(&mut ast, &constructed_table);
    handle_errors(options.warn_as_error, &mut error_handler, &slice_files)?;

    // Visit the fully parsed slice files to check for additional errors and warnings.
    let mut validator = Validator::new(&mut error_handler);
    for slice_file in slice_files.values() {
        slice_file.visit_with(&mut validator, &ast);
    }

    // Return the data to the compiler's main function.
    Ok(CompilerData { ast, slice_files, error_handler, constructed_table })
}

pub fn handle_errors(warn_as_error: bool, error_handler: &mut ErrorHandler, slice_files: &HashMap<String, SliceFile>) -> Result<(), ()> {
    error_handler.print_errors(&slice_files);
    if error_handler.has_errors(warn_as_error) {
        let counts = error_handler.get_totals();
        println!("Compilation failed with {} error(s) and {} warning(s).\n", counts.0, counts.1);
        Err(())
    } else {
        Ok(())
    }
}
