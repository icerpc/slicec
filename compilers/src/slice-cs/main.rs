
mod cs_writer;
mod cs_options;
mod cs_util;

use slice::CompilerData;
use cs_options::CsOptions;
use cs_writer::CsWriter;
use structopt::StructOpt;

pub fn main() {
    let options = CsOptions::from_args();
    let parse_result = slice::parse_from_options(&options.slice_options);

    match parse_result {
        Ok(data) => { generate_code(data); println!("SUCCESS"); },
        Err(()) => { println!("FAILED") },
    };
}

fn generate_code(compiler_data: CompilerData) {
    for slice_file in compiler_data.slice_files.values() {
        let mut writer = CsWriter::new(&slice_file.path);
        slice_file.visit(&mut writer, &compiler_data.ast);// this API feels weird, maybe add a visitor.visit_file(...) method to make this better
        writer.flush();
    }
}
