// Copyright (c) ZeroC, Inc.

use std::path::{PathBuf, MAIN_SEPARATOR};

use slicec::diagnostics::DiagnosticReporter;
use slicec::slice_options::SliceOptions;
use slicec::utils::file_util::resolve_files_from;

#[test]
fn duplicate_reference_files_ignored() {
    // Arrange
    let file = PathBuf::from(["tests", "files", "..", "files", "test.slice"].join(&MAIN_SEPARATOR.to_string()));
    let options = SliceOptions {
        sources: vec![file.to_str().unwrap().to_owned()],
        references: vec![file.to_str().unwrap().to_owned()],
        ..Default::default()
    };
    let mut reporter = DiagnosticReporter::new(&options);

    // Act
    let files = resolve_files_from(&options, &mut reporter);

    // Assert
    assert_eq!(files.len(), 1);
}

#[test]
fn duplicate_source_files_ignored() {
    // Arrange
    let file_path_one = PathBuf::from(["tests", "files", "test.slice"].join(&MAIN_SEPARATOR.to_string()));
    let file_path_two =
        PathBuf::from(["tests", "files", "..", "files", "test.slice"].join(&MAIN_SEPARATOR.to_string()));
    let options = SliceOptions {
        sources: vec![
            file_path_one.to_str().unwrap().to_owned(),
            file_path_two.to_str().unwrap().to_owned(),
        ],
        ..Default::default()
    };
    let mut reporter = DiagnosticReporter::new(&options);

    // Act
    let files = resolve_files_from(&options, &mut reporter);

    // Assert
    assert_eq!(files.len(), 1);
}
