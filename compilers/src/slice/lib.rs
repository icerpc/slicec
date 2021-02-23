
#[macro_use] pub mod util;
pub mod ast;
pub mod grammar;
pub mod options;
pub mod visitor;
mod ast_patcher;
mod parser;
mod table_builder;

use ast::{SliceAst, SliceFile};
use options::SliceOptions;
use parser::SliceParser;
use table_builder::TableBuilder;
use util::SliceError;
use visitor::Visitable;

use std::collections::HashMap;

pub fn parse_from_input(options: &SliceOptions) -> Result<(SliceAst, HashMap<String, SliceFile>), Vec<SliceError>> {
    // Parse the slice file into an unpatched AST.
    let mut slice_parser = SliceParser::new(options);
    for path in options.sources.iter() { // TODO: make this able to handle directories and relative paths and stuff!
        slice_parser.parse_file(path.clone(), true);
    }
    for path in options.references.iter() { // TODO: make this able to handle directories and relative paths and stuff!
        slice_parser.parse_file(path.clone(), false);
    }
    let (unpatched_ast, slice_files, errors) = slice_parser.into_data();
    if !errors.is_empty() {
        return Err(errors);
    }

    // Generate a lookup table from the unpatched AST.
    let mut table_builder = TableBuilder::new();
    for slice_file in slice_files.values() {
        slice_file.visit(&mut table_builder, &unpatched_ast);
    }
    let lookup_table = table_builder.into_table();

    // Patch the AST with the lookup table.
    let mut ast_patcher = AstPatcher::new();
    for slice_file in slice_files.values() {
        slice_file.visit(&mut ast_patcher, &unpatched_ast);
    }
    let (patched_ast, errors) = ast_patcher.into_ast();
    if !errors.is_empty() {
        return Err(errors);
    }

    // Check for any remaining errors or warnings in the patched AST.
    let mut slice_validator = SliceValidator::new();
    for slice_file in slice_files.values() {
        slice_file.visit(&mut ast_patcher, &unpatched_ast);
    }
    let errors = slice_validator.into_errors();
    if !errors.is_empty() {
        return Err(errors);
    }

    Ok((patched_ast, slice_files))
}
