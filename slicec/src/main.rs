// Copyright (c) ZeroC, Inc.

use std::fs::File;
use std::io::{Error, ErrorKind, Write};
use std::process::{Command, ExitCode, Stdio};

use clap::Parser;

use slice_codec::decoder::Decoder;
use slice_codec::encoder::Encoder;

use slicec::compilation_state::CompilationState;
use slicec::diagnostic_emitter::DiagnosticEmitter;
use slicec::slice_options::{DiagnosticFormat, SliceOptions};

use crate::definition_types::GeneratedFile;

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
        let converted_file = definition_types::SliceFile::from(parsed_file);
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

fn decode_generate_code_response(payload: &[u8])
    -> Result<(Vec<definition_types::GeneratedFile>, Vec<definition_types::Diagnostic>), slice_codec::Error>
{
    // Create a decoder over the response's payload.
    let mut slice_decoder = Decoder::from(payload);

    // Decode the response as 2 sequences, one for generated files, and one for diagnostics.
    let generated_files = slice_decoder.decode()?;
    let diagnostics = slice_decoder.decode()?;
    Ok((generated_files, diagnostics))
}

fn run_plugin_process(command: &str, slice_payload: &[u8]) -> std::io::Result<Vec<u8>> {
    // Spawn a new subprocess and setup pipes for all of its streams.
    let mut subprocess = Command::new(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Write the encoded Slice definitions to the subprocess's 'stdin'.
    let stdin = subprocess.stdin.as_mut().ok_or_else(|| Error::other("failed to access 'stdin'"))?;
    // We require that plugins must read the entire payload from 'stdin' before writing to 'stdout' or 'stderr',
    // so there's no concern of deadlock due to the pipe buffer filling up.
    stdin.write_all(slice_payload)?;

    // Wait until the subprocess finishes, then retrieve its output.
    let output = subprocess.wait_with_output()?;

    // If the subprocess wrote anything to its 'stderr', we consider this a failure and don't generate any code.
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

    // Otherwise, check the subprocess's status code to determine success.
    match output.status.code() {
        // If the subprocess exited with a status code of 0, all is good. We return the encoded response from 'stdout'.
        Some(0) => Ok(output.stdout),

        // If the subprocess exited with a non-zero status code, we consider this a failure and don't generate any code.
        Some(code) => Err(Error::other(format!("failed with status code '{code}'"))),

        // If the subprocess did not exit with a status code, it was interrupted; this is also treated as a failure.
        None => Err(Error::from(ErrorKind::Interrupted)),
    }
}

fn write_generated_file(generated_file: GeneratedFile) -> std::io::Result<()> {
    let mut file = File::create(generated_file.path)?;
    file.write_all(generated_file.contents.as_bytes())?;
    Ok(())
}

// TODO update this to use a Result type instead.
fn main() -> ExitCode {
    // Parse the command-line input.
    let slice_options = SliceOptions::parse();

    // Perform the compilation.
    let compilation_state = slicec::compile_from_options(&slice_options, |_| {}, |_| {});
    let CompilationState { ast, diagnostics, files } = compilation_state;

    // Process the diagnostics (filter out allowed lints, and update diagnostic levels as necessary).
    let mut updated_diagnostics = diagnostics.into_updated(&ast, &files, &slice_options);
    let (mut warning_count, mut error_count) = slicec::diagnostics::get_totals(&updated_diagnostics);

    // Only invoke the plugins if there were no errors in the Slice files.
    if error_count == 0 {
        // TODO START refactor this to be better.
        let encoded_response = match encode_generate_code_request(&files) {
            Ok(bytes) => bytes,
            Err(error) => {
                eprintln!("{error:?}");
                return ExitCode::from(11);
            }
        };
        let encoded_response = match run_plugin_process("slicec-cs", &encoded_response) {
            Ok(bytes) => bytes,
            Err(error) => {
                eprintln!("{error:?}");
                return ExitCode::from(12);
            }
        };
        let (generated_files, diagnostics) = match decode_generate_code_response(&encoded_response) {
            Ok(data) => data,
            Err(error) => {
                eprintln!("{error:?}");
                return ExitCode::from(13);
            }
        };

        // TODO update the number of diagnostics. Then conditionally generate the code in the right place.

        for generated_file in generated_files {
            let mut file_handle = match write_generated_file(generated_file) {
                Ok(_) => {},
                Err(error) => {
                    eprintln!("{error:?}");
                    return ExitCode::from(14);
                }
            };
        }
        // TODO END refactor this to be better.
    }

    // Print any diagnostics to the console, along with the total number of warnings and errors emitted.
    let mut stderr = console::Term::stderr();
    let mut emitter = DiagnosticEmitter::new(&mut stderr, &slice_options, &files);
    DiagnosticEmitter::emit_diagnostics(&mut emitter, updated_diagnostics).expect("failed to emit diagnostics");

    // Only emit the summary message if we're writing human-readable output.
    if slice_options.diagnostic_format == DiagnosticFormat::Human {
        slicec::diagnostic_emitter::emit_totals(warning_count, error_count).expect("failed to emit totals");
    }

    // Finished.
    ExitCode::from(error_count)
}
