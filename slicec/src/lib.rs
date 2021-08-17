// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod ast;
mod comment_parser;
pub mod error;
pub mod grammar;
pub mod options;
mod parser;
mod patchers;
pub mod slice_file;
pub mod util;
pub mod util_macros;
mod validator;
pub mod visitor;
pub mod writer;

use crate::ast::Ast;
use crate::error::ErrorHandler;
use crate::options::SliceOptions;
use crate::parser::SliceParser;
use crate::patchers::{ScopePatcher, TableBuilder, TypePatcher};
use crate::slice_file::SliceFile;
use crate::validator::Validator;
use std::collections::HashMap;

#[derive(Debug)]
pub struct CompilerData {
    pub ast: Ast,
    pub slice_files: HashMap<String, SliceFile>,
    pub error_handler: ErrorHandler,
    pub lookup_table: HashMap<String, usize>,
}

pub fn parse_from_options(options: &SliceOptions) -> Result<CompilerData, ()> {
    // Parse the slice files from the command line input into an unpatched AST.
    let (mut ast, slice_files, mut error_handler) = SliceParser::parse_files(&options);
    handle_errors(options.warn_as_error, &mut error_handler, &slice_files)?;

    // Generate the tables for looking up elements by identifier and for patching element's scopes.
    let mut table_builder = TableBuilder::new(&mut error_handler);
    table_builder.generate_tables(&slice_files, &ast);
    let (lookup_table, scope_patches) = table_builder.into_tables();

    // Patch the element's scopes. We can't do this during parsing since Pest parses bottom up.
    ScopePatcher::patch_scopes(scope_patches, &mut ast);
    // Patch any user defined types to their definitions.
    TypePatcher::new(&lookup_table, &mut error_handler).patch_types(&mut ast);
    handle_errors(options.warn_as_error, &mut error_handler, &slice_files)?;

    // Visit the fully parsed slice files to check for additional errors and warnings.
    let mut validator = Validator::new(&mut error_handler);
    for slice_file in slice_files.values() {
        slice_file.visit_with(&mut validator, &ast);
    }

    // Return the data to the compiler's main function.
    Ok(CompilerData {
        ast,
        slice_files,
        error_handler,
        lookup_table,
    })
}

pub fn handle_errors(
    warn_as_error: bool,
    error_handler: &mut ErrorHandler,
    slice_files: &HashMap<String, SliceFile>,
) -> Result<(), ()> {
    error_handler.print_errors(&slice_files);
    if error_handler.has_errors(warn_as_error) {
        let counts = error_handler.get_totals();
        println!(
            "Compilation failed with {} error(s) and {} warning(s).\n",
            counts.0, counts.1
        );
        Err(())
    } else {
        Ok(())
    }
}
