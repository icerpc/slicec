// Copyright (c) ZeroC, Inc. All rights reserved.

mod builders;
mod code_block;
mod comments;
mod cs_options;
mod cs_util;
mod cs_validator;
mod cs_writer;
mod decoding;
mod encoding;
mod proxy_visitor;

use slice::writer::Writer;

use cs_options::CsOptions;
use cs_validator::CsValidator;
use cs_writer::CsWriter;
use proxy_visitor::ProxyVisitor;
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
            let mut output = Writer::new(&format!("{}.cs", slice_file.filename)).unwrap();

            {
                let mut cs_writer = CsWriter::new(&mut output);
                slice_file.visit_with(&mut cs_writer, &data.ast);
            }

            {
                let mut proxy_visitor = ProxyVisitor::new(&mut output);
                slice_file.visit_with(&mut proxy_visitor, &data.ast);
            }

            output.close()
        }
    }

    let _ = slice::handle_errors(true, &mut data.error_handler, &data.slice_files);
    Ok(())
}
