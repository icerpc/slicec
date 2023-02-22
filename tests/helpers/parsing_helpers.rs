// Copyright (c) ZeroC, Inc.

use slice::ast::Ast;
use slice::compile_from_strings;
use slice::diagnostics::Diagnostic;

/// This function is used to parse a Slice file and return the AST.
pub fn parse_for_ast(slice: impl Into<String>) -> Ast {
    match compile_from_strings(&[&slice.into()], None) {
        Ok(data) => data.ast,
        Err(e) => panic!("{:?}", e.diagnostic_reporter),
    }
}

/// This function is used to parse a Slice file and return any Diagnostics that were emitted.
pub fn parse_for_diagnostics(slice: impl Into<String>) -> Vec<Diagnostic> {
    parse_multiple_for_diagnostics(&[&slice.into()])
}

/// This function is used to parse multiple Slice files and return any Diagnostics that were emitted.
pub fn parse_multiple_for_diagnostics(slice: &[&str]) -> Vec<Diagnostic> {
    let data = match compile_from_strings(slice, None) {
        Ok(data) => data,
        Err(data) => data,
    };
    data.diagnostic_reporter
        .into_diagnostics(&data.ast, &data.files)
        .collect()
}

/// Compares diagnostics emitted by the compiler to an array of expected diagnostics.
/// It ensures that the expected number of diagnostics were emitted (ie: that both lists are the same length).
///
/// If the correct number were emitted, it checks each diagnostic against the expected array in order.
/// For each diagnostic we ensure:
/// - It has the correct error code.
/// - It has the correct message.
/// - If a span was expected, that it has the correct span.
/// - If notes are expected, we check that all the notes have correct messages and spans.
///
/// If the expected diagnostics don't include spans or notes, this function doesn't check them.
/// This is useful for the majority of tests that aren't explicitly testing spans or notes.
#[rustfmt::skip]
pub fn check_diagnostics<const L: usize>(diagnostics: Vec<Diagnostic>, expected: [impl Into<Diagnostic>; L]) {
    // Check that the correct number of diagnostics were emitted.
    if expected.len() != diagnostics.len() {
        eprintln!("Expected {} diagnostics, but got {}.", expected.len(), diagnostics.len());
        eprintln!("The emitted diagnostics were:");
        for diagnostic in diagnostics {
            eprintln!("\t{diagnostic:?}");
        }
        eprintln!();
        panic!("test failure");
    }

    // Check that the emitted diagnostics match what was expected.
    for (expect, diagnostic) in expected.into_iter().zip(diagnostics.into_iter()) {
        let expect = expect.into();
        let mut failed = false;

        // Check that the error codes match.
        if expect.error_code() != diagnostic.error_code() {
            eprintln!("diagnostic codes didn't match:");
            eprintln!("\texpected '{:?}', but got '{:?}'", expect.error_code(), diagnostic.error_code());
            failed = true;
        }

        // Check that the messages match.
        if expect.message() != diagnostic.message() {
            eprintln!("diagnostic messages didn't match:");
            eprintln!("\texpected: \"{}\"", expect.message());
            eprintln!("\t but got: \"{}\"", diagnostic.message());
            failed = true;
        }

        // If a span was provided, check that it matches.
        if expect.span().is_some() && expect.span() != diagnostic.span() {
            eprintln!("diagnostic spans didn't match:");
            eprintln!("\texpected: \"{:?}\"", expect.span());
            eprintln!("\t but got: \"{:?}\"", diagnostic.span());
            failed = true;
        }

        // If notes were provided, check that they match.
        if !expect.notes().is_empty() {
            let expected_notes = expect.notes();
            let emitted_notes = diagnostic.notes();
            if expected_notes.len() != emitted_notes.len() {
                eprintln!("Expected {} notes, but got {}.", expected_notes.len(), emitted_notes.len());
                eprintln!("The emitted notes were:");
                for note in emitted_notes {
                    eprintln!("\t{note:?}");
                }
                failed = true;
            } else {
                for (expected_note, emitted_note) in expected_notes.iter().zip(emitted_notes.iter()) {
                    // Check that the messages match.
                    if expected_note.message != emitted_note.message {
                        eprintln!("note messages didn't match:");
                        eprintln!("\texpected: \"{}\"", expected_note.message);
                        eprintln!("\t but got: \"{}\"", emitted_note.message);
                        failed = true;
                    }

                    // If a span was provided, check that it matches.
                    if expected_note.span.is_some() && expected_note.span != emitted_note.span {
                        eprintln!("note spans didn't match:");
                        eprintln!("\texpected: \"{:?}\"", expected_note.span);
                        eprintln!("\t but got: \"{:?}\"", emitted_note.span);
                        failed = true;
                    }
                }
            }
        }

        // If the checks failed, panic to signal a test failure.
        if failed {
            eprintln!();
            panic!("test failure");
        }
    }
}
