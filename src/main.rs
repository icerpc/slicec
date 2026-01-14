// Copyright (c) ZeroC, Inc.

use clap::Parser;
use slicec::slice_options::SliceOptions;
use slicec::compilation_state::CompilationState;

fn main() {
    // Parse the command-line input.
    let slice_options = SliceOptions::parse();

    // Perform the compilation.
    let compilation_state = slicec::compile_from_options(&slice_options, |_| {}, |_| {});
    let CompilationState { ast, diagnostics, files } = compilation_state;

    // Process the diagnostics (filter out allowed lints, and update diagnostic levels as necessary).
    let updated_diagnostics = diagnostics.into_updated(&ast, &files, &slice_options);
    let totals = slicec::diagnostics::get_totals(&updated_diagnostics);

    // Print output to stdout.
    print!("Diagnostics: ");
    println!("{totals:?}");
    for diagnostic in updated_diagnostics {
        println!("{diagnostic:?}");
    }
    println!("{ast:?}");

    std::process::exit(i32::from(totals.1 != 0));
}
