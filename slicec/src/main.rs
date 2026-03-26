// Copyright (c) ZeroC, Inc.

use std::fs::File;
use std::io::{Error, ErrorKind, Write};
use std::process::{Child, Command, Stdio};

use clap::Parser;

use slice_codec::decoder::Decoder;
use slice_codec::encoder::Encoder;

use slicec::compilation_state::CompilationState;
use slicec::diagnostic_emitter::DiagnosticEmitter;
use slicec::diagnostics::Diagnostics;
use slicec::slice_options::{DiagnosticFormat, SliceOptions};

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

fn spawn_plugin_process(command: &str, slice_payload: &[u8]) -> std::io::Result<Child> {
    // Spawn a new subprocess and set up pipes for all of its streams.
    let mut subprocess = Command::new(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Write the encoded Slice definitions to the subprocess's 'stdin'.
    let stdin = subprocess.stdin.as_mut().ok_or(ErrorKind::BrokenPipe)?;
    // We require that plugins must read the entire payload from 'stdin' before writing to 'stdout' or 'stderr',
    // so there's no concern of deadlock due to the pipe buffer filling up.
    stdin.write_all(slice_payload)?;

    // Return a handle to the subprocess so we can wait on it to complete.
    Ok(subprocess)
}

fn collect_plugin_output(subprocess: Child) -> std::io::Result<Vec<u8>> {
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

fn handle_plugin_response(response_payload: Vec<u8>) -> std::io::Result<Diagnostics> {
    // Decode the plugin's response. It consists of 2 sequences, one of generated files and one of diagnostics.
    let mut slice_decoder = Decoder::from(&response_payload);
    let generated_files: Vec<definition_types::GeneratedFile> = slice_decoder.decode()?;
    let plugin_diagnostics: Vec<definition_types::Diagnostic> = slice_decoder.decode()?;

    // TODO: Convert the diagnostics we decode from the plugin, into diagnostics that slicec can handle.
    //       To do this requires re-working the diagnostic API fairly substantially.
    for plugin_diagnostic in plugin_diagnostics {
        println!("{}", plugin_diagnostic.message);
    }
    let mut diagnostics = Diagnostics::new();

    // If no errors were reported, attempt to generate the files in the response.
    if !diagnostics.has_errors() {
        for generated_file in &generated_files {
            // Try to write the generated file to disk.
            if let Err(io_error) = write_generated_file(generated_file) {
                // If an error occurred during writing the file, create a diagnostic that slicec can report.
                let diagnostic = slicec::diagnostics::Error::IO {
                    action: "write generated file",
                    path: generated_file.path.to_owned(),
                    error: io_error,
                };
                slicec::diagnostics::Diagnostic::new(diagnostic).push_into(&mut diagnostics);
            }
        }
    }

    // Return any decoded diagnostics, so slicec can emit them at the end.
    Ok(diagnostics)
}

fn write_generated_file(generated_file: &definition_types::GeneratedFile) -> std::io::Result<()> {
    let mut file = File::create(&generated_file.path)?;
    file.write_all(generated_file.contents.as_bytes())?;
    Ok(())
}

fn convert_plugin_error_to_diagnostic(plugin_name: &str, plugin_error: std::io::Error) -> Diagnostics {
    let mapped_io_error = slicec::diagnostics::Error::IO {
        action: "run code-generator plugin",
        path: plugin_name.to_owned(),
        error: plugin_error,
    };

    let mut diagnostics = Diagnostics::new();
    slicec::diagnostics::Diagnostic::new(mapped_io_error).push_into(&mut diagnostics);
    diagnostics
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse the command-line input.
    let slice_options = SliceOptions::parse();

    // Perform the compilation.
    let compilation_state = slicec::compile_from_options(&slice_options, |_| {}, |_| {});
    let CompilationState {
        ast,
        mut diagnostics,
        files,
    } = compilation_state;

    // Only invoke the plugins if there were no errors in the Slice files.
    if !diagnostics.has_errors() {
        // Encode the request which will be sent to each of the code-generation plugins.
        let encoded_request = encode_generate_code_request(&files)?;

        // TODO: add a CLI to choose which plugins are run, instead of hard-coding them here.
        #[cfg(target_os = "windows")]
        let plugins = ["ZeroC.Slice.Generator.exe", "IceRpc.Slice.Generator.exe"];
        #[cfg(not(target_os = "windows"))]
        let plugins = ["ZeroC.Slice.Generator", "IceRpc.Slice.Generator"];

        // Start each of the plugins in parallel.
        let plugin_processes = plugins.map(|plugin| (plugin, spawn_plugin_process(plugin, &encoded_request)));

        // Block on each plugin process until they're finished. If a plugin completes successfully,
        // we get the response payload from it, write any generated files in the payload, and store any diagnostics
        // the plugin reported so we can emit them at the end along with all the others.
        for (plugin_name, plugin_process) in plugin_processes {
            let plugin_diagnostics = plugin_process
                .and_then(collect_plugin_output) // Returns the response payload if the plugin ran successfully.
                .and_then(handle_plugin_response) // Returns plugin diagnostics if the payload was decoded successfully.
                .unwrap_or_else(|err| convert_plugin_error_to_diagnostic(plugin_name, err));

            diagnostics.extend(plugin_diagnostics); // Store the plugin diagnostics for later emission.
        }
    }

    // Process the diagnostics (filter out allowed lints, and update diagnostic levels as necessary).
    let updated_diagnostics = diagnostics.into_updated(&ast, &files, &slice_options);
    let (warning_count, error_count) = slicec::diagnostics::get_totals(&updated_diagnostics);

    // Print any diagnostics to the console, along with the total number of warnings and errors emitted.
    let mut stderr = console::Term::stderr();
    let mut emitter = DiagnosticEmitter::new(&mut stderr, &slice_options, &files);
    DiagnosticEmitter::emit_diagnostics(&mut emitter, updated_diagnostics).expect("failed to emit diagnostics");

    // Only emit the summary message if we're writing human-readable output.
    if slice_options.diagnostic_format == DiagnosticFormat::Human {
        slicec::diagnostic_emitter::emit_totals(warning_count, error_count).expect("failed to emit totals");
    }

    // Finished.
    match error_count {
        0 => Ok(()),
        _ => Err("".into()),
    }
}
