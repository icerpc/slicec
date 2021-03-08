
pub mod ast;
pub mod error;
pub mod grammar;
pub mod options;
pub mod util;
pub mod visitor;
pub mod writer;
mod parser;
mod patchers;
mod table_builder;
mod validator;

use ast::Ast;
use error::ErrorHandler;
use options::SliceOptions;
use parser::SliceParser;
use patchers::{ScopePatcher, TypePatcher};
use table_builder::TableBuilder;
use util::SliceFile;
use validator::Validator;
use std::collections::HashMap;

///
pub fn parse_from_options(options: &SliceOptions) -> Result<CompilerData, ()> {
    let (mut ast, slice_files, constructed_table, mut error_handler) = SliceParser::parse_files(&options);
    handle_errors(options, &mut error_handler, &slice_files)?;

    let defined_table = TableBuilder::new(&mut error_handler).build_lookup_table(&slice_files, &ast);
    ScopePatcher::patch_scopes(&mut ast, &defined_table);
    TypePatcher::patch_types(&mut ast, &defined_table, &mut error_handler);
    handle_errors(options, &mut error_handler, &slice_files)?;

    let mut validator = Validator::new(&mut error_handler);
    for slice_file in slice_files.values() {
        slice_file.visit_with(&mut validator, &ast);
    }
    handle_errors(options, &mut error_handler, &slice_files)?;

    Ok(CompilerData { ast, slice_files, error_handler, constructed_table, defined_table })
}

///
fn handle_errors(options: &SliceOptions, error_handler: &mut ErrorHandler, slice_files: &HashMap<String, SliceFile>) -> Result<(), ()> {
    error_handler.print_errors(&slice_files);
    if error_handler.has_errors(options.warn_as_error) {
        let counts = error_handler.get_totals();
        println!("Compilation failed with {} error(s) and {} warning(s).\n", counts.0, counts.1);
        Err(())
    } else {
        Ok(())
    }
}

//------------------------------------------------------------------------------
// CompilerData
//------------------------------------------------------------------------------
#[derive(Debug, Default)]
pub struct CompilerData {
    pub ast: Ast,
    pub slice_files: HashMap<String, SliceFile>,
    pub error_handler: ErrorHandler,
    pub constructed_table: HashMap<String, usize>,
    pub defined_table: HashMap<String, usize>,
}

// implement the debug, and dry_run settings
// implement the output directory setting
// implement a check for redefinitions
// improve the snippeting system
// introduce support for comments
// clean up and comment all my code
// actually write the slicec-cs project for real
// write a 'writer' struct for simplifying the code generation process.
// Add support for passing directoes into the slice compiler
//      We need to do this BEFORE we pass the options into the Slice Parser, as it expects nothing but files!
//      It probably makes the most sense to add this functionality into the `options` module.
//  We should also support preservation of relative paths. So if you parse the 'Hello/' directory, it's sub-structure should be preserved in the generated code.
//      Hello
//      --Foo
//        --thing.ice
//  Should get mapped to
//      %outputdir%
//      --Foo
//        --thing.cs
// Make sure that the compiler can cope with random strings passed in as files. The SliceFile code parses it with `unwraps` so we should be careful.
// It's worth noting that the identifiers in this compiler are case sensative, so `IFoo` is different than `iFoo`.

// Come up with better names for things

// For the main function to do:
// if error_handler.has_errors(true) {
//     error_handler.print_errors(&slice_files);
//     let counts = error_handler.get_totals();
//     println!("Compilation failed with {} error(s) and {} warning(s).\n", counts.0, counts.1);
// }
