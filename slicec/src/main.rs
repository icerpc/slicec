// Copyright (c) ZeroC, Inc.

use std::io::Write;

use clap::Parser;

use slice_codec::encoder::Encoder;

use slicec::compilation_state::CompilationState;
use slicec::slice_options::SliceOptions;

pub mod definition_types;
pub mod slice_file_converter;

/// Attempts to encode a set of parsed Slice files into a byte-buffer.
/// If the encoding succeeds, this returns `Ok` with the encoded bytes,
/// otherwise this returns `Err` with an error describing the failure.
fn encode_generate_code_request(parsed_files: &[slicec::slice_file::SliceFile]) -> Result<Vec<u8>, slice_codec::Error> {
    // Create a buffer to encode into, and an encoder over-top of it.
    let mut encoding_buffer: Vec<u8> = Vec::new();
    let mut slice_encoder = Encoder::from(&mut encoding_buffer);

    // Encode the 'operation name'.
    slice_encoder.encode("generateCode")?;

    // Sort the parsed files into two groups: source files and reference files.
    // We also convert from the AST representation to the Slice representation at this time.
    let mut source_files = Vec::new();
    let mut reference_files = Vec::new();
    for parsed_file in parsed_files {
        // Convert the Slice file from AST representation to Slice representation.
        let converted_file = crate::definition_types::SliceFile::from(parsed_file);
        // Determine whether this is a source or reference file and place it accordingly.
        match parsed_file.is_source {
            true => source_files.push(converted_file),
            false => reference_files.push(converted_file),
        }
    }

    // Encode the Slice-files as 2 sequences; one of source files, and one of reference files.
    slice_encoder.encode(&source_files)?;
    slice_encoder.encode(&reference_files)?;

    // We're done!
    Ok(encoding_buffer)
}

fn main() {
    // Parse the command-line input.
    let slice_options = SliceOptions::parse();

    // Perform the compilation.
    let compilation_state = slicec::compile_from_options(&slice_options, |_| {}, |_| {});
    let CompilationState { ast, diagnostics, files } = compilation_state;

    // Process the diagnostics (filter out allowed lints, and update diagnostic levels as necessary).
    let updated_diagnostics = diagnostics.into_updated(&ast, &files, &slice_options);
    let totals = slicec::diagnostics::get_totals(&updated_diagnostics);

    // TODO: replace this by forking a code-gen plugin once they exist.
    // For now, if there are any diagnostics, we emit those and NOT the encoded definitions.
    // Code-generators can tell if it's okay to decode or not by the presence of the `"generateCode"` string.
    if totals.0 + totals.1 > 0 {
        // If there were diagnostics, print them to 'stdout' and don't encode the Slice definitions.
        print!("Diagnostics: ");
        println!("{totals:?}");
        for diagnostic in updated_diagnostics {
            println!("{diagnostic:?}");
        }
    } else {
        // Encode the parsed Slice definitions.
        let encoded_bytes = match encode_generate_code_request(&files) {
            Ok(bytes) => bytes,
            Err(error) => {
                eprintln!("{error:?}");
                std::process::exit(11);
            }
        };

        // Obtain an exclusive handle to 'stdout', and write the encoded bytes to it.
        let mut stdout = std::io::stdout().lock();
        match stdout.write_all(&encoded_bytes) {
            Ok(_) => {}
            Err(error) => {
                eprintln!("{error:?}");
                std::process::exit(12);
            }
        }
    }

    // Success.
    std::process::exit(0);
}
