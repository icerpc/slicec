// Copyright (c) ZeroC, Inc.

use std::io::{Error, ErrorKind, Write};
use std::process::{Command, ExitCode, Stdio};

use clap::Parser;

use slice_codec::decoder::Decoder;
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

fn decode_generate_code_response(payload: &[u8]) {
    // Create a decoder over the response's payload.
    let mut slice_decoder = Decoder::from(payload);

    // Decode the response as 2 sequences, one for generated files, and one for diagnostics.
    let generatedFiles = slice_decoder.decode::<Vec<crate::definition_types::GeneratedFile>>();
    let diagnostics = slice_decoder.decode::<Vec<crate::definition_types::Diagnostic>>();
}

fn run_plugin_process(command: &str, slice_payload: &[u8]) -> std::io::Result<Vec<u8>> {
    // Spawn a new process and setup pipes for all of its streams.
    let mut process = Command::new(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Write the encoded Slice definitions to the process's 'stdin'.
    let stdin = process.stdin.as_mut().ok_or_else(|| Error::other("failed to access 'stdin'"))?;
    // We require that plugins must read the entire payload from 'stdin' before writing to 'stdout' or 'stderr',
    // so there's no concern of deadlock due to the pipe buffer filling up.
    stdin.write_all(slice_payload)?;

    // Wait until the process finishes, then retrieve its output.
    let output = process.wait_with_output()?;

    // If the process wrote anything to its 'stderr', we consider this a failure and don't generate any code.
    if !output.stderr.is_empty() {
        // Obtain an exclusive handle to this process's 'stderr'.
        let mut stderr = std::io::stderr().lock();

        // Pipe the output from the subprocess's 'stderr' to this process's 'stderr', and then return.
        let error = match stderr.write_all(&output.stderr) {
            Ok(_) => Error::other("errors reported on 'stderr'"),
            Err(err) => Error::new(ErrorKind::BrokenPipe, err),
        };
        return Err(error);
    }
    
    // Otherwise, check the process's status code to determine success.
    match output.status.code() {
        // If the process exited with a status code of 0, all is good and we return the encoded response from 'stdout'.
        Some(0) => Ok(output.stdout),

        // If the process exited with a non-zero status code, we consider this a failure and don't generate any code.
        // TODO: we treat non-zero codes as terminal errors. Should we?
        Some(code) => Err(Error::other(format!("failed with status code '{code}'"))),

        // If the process did not exit with a status code, it means it was interrupted by a signal.
        None => Err(Error::from(ErrorKind::Interrupted)),
    }
}

fn main() -> ExitCode {
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
    let (warnings, errors) = totals;
    if warnings + errors > 0 {
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
                return ExitCode::from(11);
            }
        };

        let command_string = "echo";
    }

    // Success.
    ExitCode::from(0)
}
