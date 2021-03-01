
pub mod ast;
pub mod error;
pub mod grammar;
pub mod options;
pub mod util;
pub mod visitor;
mod parser;
mod patchers;
mod table_builder;
mod validator;

use ast::SliceAst;
use error::ErrorHandler;
use options::SliceOptions;
use parser::SliceParser;
use patchers::{ScopePatcher, TypePatcher};
use table_builder::TableBuilder;
use util::SliceFile;
use validator::Validator;
use std::collections::HashMap;

//------------------------------------------------------------------------------
// Entry Function
//------------------------------------------------------------------------------
pub fn parse_from_options(options: &SliceOptions) -> Result<CompilerData, ()> {
    let (mut ast, slice_files, constructed_table, mut error_handler) = SliceParser::parse_files(&options);
    if error_handler.has_errors(options.warn_as_error) {
        return Err(error_handler.print_errors(&slice_files))
    }

    let defined_table = TableBuilder::build_lookup_table(&slice_files, &ast);
    ScopePatcher::patch_scopes(&mut ast, &defined_table);
    TypePatcher::patch_types(&mut ast, &defined_table, &mut error_handler);
    if error_handler.has_errors(options.warn_as_error) {
        return Err(error_handler.print_errors(&slice_files))
    }

    let mut validator = Validator::new(&mut error_handler);
    for slice_file in slice_files.values() {
        slice_file.visit(&mut validator, &ast);
    }
    if error_handler.has_errors(options.warn_as_error) {
        return Err(error_handler.print_errors(&slice_files))
    }

    Ok(CompilerData { ast, slice_files, error_handler, constructed_table, defined_table })
}

//------------------------------------------------------------------------------
// CompilerData
//------------------------------------------------------------------------------
#[derive(Debug, Default)]
pub struct CompilerData {
    pub ast: SliceAst,
    pub slice_files: HashMap<String, SliceFile>,
    pub error_handler: ErrorHandler,
    pub constructed_table: HashMap<String, usize>,
    pub defined_table: HashMap<String, usize>,
}

// We need to add support for passing directories to the slice compiler!
// We need to do this BEFORE we pass the options into the Slice Parser, as it expects nothing but files!
// It probably makes the most sense to add this functionality into the `options` module.

// Implement the following compiler flags!
// pub debug: bool,
// pub warn_as_error: bool,
// pub dry_run: bool,

// TODO MAKE ALL THESE MAIN CALLS INTO A RESULT SO WE CAN '?' ON THEM!
