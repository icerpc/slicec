
pub mod ast;
pub mod grammar;
pub mod options;
pub mod util;
pub mod visitor;
mod parser;
mod table_builder;

use ast::{SliceAst, SliceFile};
use options::SliceOptions;
use parser::SliceParser;
use table_builder::TableBuilder;
use util::SliceError;
use visitor::Visitable;

use std::collections::HashMap;

type SliceResult = Result<(SliceAst, HashMap<String, SliceFile>, HashMap<String, usize>), Vec<SliceError>>;

pub fn parse_from_input(options: &SliceOptions) -> SliceResult {
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

    let mut table_builder = TableBuilder::new();
    for slice_file in slice_files.values() {
        slice_file.visit(&mut table_builder, &unpatched_ast);
    }
    let slice_table = table_builder.into_table();

    unimplemented!()

//    let type_table = table_builder.into_table();
//
//    let patched_ast = ast_patcher::patch_ast(unpatched_ast, &type_table);
//
//    let mut slice_validator = SliceValidator::new();
//    for slice_file in slice_files.values() {
//        slice_file.visit(&mut slice_validator, &patched_ast);
//    }
//
//    (patched_ast, slice_files, errors)
}
