// Copyright (c) ZeroC, Inc.

use slicec::diagnostics::{Diagnostic, DiagnosticReporter, Lint};
use slicec::slice_options::SliceOptions;
use slicec::test_helpers::check_diagnostics;
use slicec::utils::file_util::resolve_files_from;
use std::path::PathBuf;

#[test]
fn file_passed_as_source_and_reference_file_is_ignored() {
    // Arrange
    let file = PathBuf::from("tests/files/../files/test.slice");
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
    assert!(reporter.diagnostics.is_empty());
}

#[test]
fn duplicate_source_files_ignored_with_warning() {
    // Arrange
    let file_path_one = PathBuf::from("tests/files/test.slice");
    let file_path_two = PathBuf::from("tests/files/../files/test.slice");
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

    let expected = Diagnostic::new(Lint::DuplicateFile {
        path: "tests/files/../files/test.slice".to_owned(),
    });
    check_diagnostics(reporter.diagnostics, [expected]);
}

#[test]
fn duplicate_reference_files_ignored_with_warning() {
    // Arrange
    let file_path_one = PathBuf::from("tests/files/test.slice");
    let file_path_two = PathBuf::from("tests/files/../files/test.slice");
    let options = SliceOptions {
        references: vec![
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

    let expected = Diagnostic::new(Lint::DuplicateFile {
        path: "tests/files/../files/test.slice".to_owned(),
    });
    check_diagnostics(reporter.diagnostics, [expected]);
}
