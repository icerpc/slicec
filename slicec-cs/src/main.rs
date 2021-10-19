// Copyright (c) ZeroC, Inc. All rights reserved.

mod builders;
mod class_visitor;
mod code_block;
mod code_map;
mod comments;
mod cs_options;
mod cs_util;
mod cs_validator;
mod cs_writer;
mod decoding;
mod dispatch_visitor;
mod encoded_result;
mod encoding;
mod enum_visitor;
mod exception_visitor;
mod member_util;
mod proxy_visitor;
mod struct_visitor;
mod traits;

use class_visitor::ClassVisitor;
use code_map::CodeMap;
use cs_options::CsOptions;
use cs_validator::CsValidator;
use cs_writer::CsWriter;
use dispatch_visitor::DispatchVisitor;
use enum_visitor::EnumVisitor;
use exception_visitor::ExceptionVisitor;
use proxy_visitor::ProxyVisitor;
use slice::writer::Writer;
use std::path::Path;
use struct_visitor::StructVisitor;
use structopt::StructOpt;

pub fn main() {
    std::process::exit(match try_main() {
        Ok(()) => {
            println!("SUCCESS");
            0
        }
        Err(()) => {
            println!("FAILED");
            1
        }
    })
}

fn try_main() -> Result<(), ()> {
    let options = CsOptions::from_args();
    let slice_options = &options.slice_options;
    let mut data = slice::parse_from_options(slice_options)?;

    let mut cs_validator = CsValidator::new(&mut data.error_handler);
    for slice_file in data.slice_files.values() {
        slice_file.visit_with(&mut cs_validator, &data.ast);
    }
    slice::handle_errors(
        slice_options.warn_as_error,
        &mut data.error_handler,
        &data.slice_files,
    )?;

    if !slice_options.validate {
        for slice_file in data.slice_files.values() {
            // TODO: actually check for the error

            let mut code_map = CodeMap::new();

            let mut visitor = StructVisitor { code_map: &mut code_map };
            slice_file.visit_with(&mut visitor, &data.ast);

            let mut proxy_visitor = ProxyVisitor { code_map: &mut code_map };
            slice_file.visit_with(&mut proxy_visitor, &data.ast);

            let mut dispatch_visitor = DispatchVisitor { code_map: &mut code_map };
            slice_file.visit_with(&mut dispatch_visitor, &data.ast);

            let mut exception_visitor = ExceptionVisitor { code_map: &mut code_map };
            slice_file.visit_with(&mut exception_visitor, &data.ast);

            let mut enum_visitor = EnumVisitor { code_map: &mut code_map };
            slice_file.visit_with(&mut enum_visitor, &data.ast);

            let mut class_visitor = ClassVisitor { code_map: &mut code_map };
            slice_file.visit_with(&mut class_visitor, &data.ast);

            {
                let path = match &slice_options.output_dir {
                    Some(output_dir) => Path::new(output_dir),
                    _ => Path::new("."),
                }
                .join(format!("{}.cs", &slice_file.filename))
                .to_owned();

                let mut output = Writer::new(&path).unwrap();
                let mut cs_writer = CsWriter {
                    output: &mut output,
                    code_map: &mut code_map,
                    empty_namespace_prefix: None,
                };
                slice_file.visit_with(&mut cs_writer, &data.ast);
                output.close()
            }
        }
    }

    let _ = slice::handle_errors(true, &mut data.error_handler, &data.slice_files);
    Ok(())
}
