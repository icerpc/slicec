// Copyright (c) ZeroC, Inc.

use slicec::diagnostics::{Diagnostic, Diagnostics, Lint};
use slicec::slice_options::SliceOptions;
use slicec::test_helpers::check_diagnostics;
use slicec::utils::file_util::resolve_files_from;
use std::path::PathBuf;

#[test]
fn file_passed_as_source_and_reference_file_is_ignored() {
    // Arrange
    let mut diagnostics = Diagnostics::new();
    let file = PathBuf::from("tests/files/../files/test.slice");
    let options = SliceOptions {
        sources: vec![file.to_str().unwrap().to_owned()],
        references: vec![file.to_str().unwrap().to_owned()],
        ..Default::default()
    };

    // Act
    let files = resolve_files_from(&options, &mut diagnostics);

    // Assert
    assert_eq!(files.len(), 1);
    assert!(diagnostics.is_empty());
}

#[test]
fn duplicate_source_files_ignored_with_warning() {
    // Arrange
    let mut diagnostics = Diagnostics::new();
    let file_path_one = PathBuf::from("tests/files/test.slice");
    let file_path_two = PathBuf::from("tests/files/../files/test.slice");
    let options = SliceOptions {
        sources: vec![
            file_path_one.to_str().unwrap().to_owned(),
            file_path_two.to_str().unwrap().to_owned(),
        ],
        ..Default::default()
    };

    // Act
    let files = resolve_files_from(&options, &mut diagnostics);

    // Assert
    assert_eq!(files.len(), 1);

    let expected = Diagnostic::new(Lint::DuplicateFile {
        path: "tests/files/../files/test.slice".to_owned(),
    });
    check_diagnostics(diagnostics.into_inner(), [expected]);
}

#[test]
fn duplicate_reference_files_ignored_with_warning() {
    // Arrange
    let mut diagnostics = Diagnostics::new();
    let file_path_one = PathBuf::from("tests/files/test.slice");
    let file_path_two = PathBuf::from("tests/files/../files/test.slice");
    let options = SliceOptions {
        references: vec![
            file_path_one.to_str().unwrap().to_owned(),
            file_path_two.to_str().unwrap().to_owned(),
        ],
        ..Default::default()
    };

    // Act
    let files = resolve_files_from(&options, &mut diagnostics);

    // Assert
    assert_eq!(files.len(), 1);

    let expected = Diagnostic::new(Lint::DuplicateFile {
        path: "tests/files/../files/test.slice".to_owned(),
    });
    check_diagnostics(diagnostics.into_inner(), [expected]);
}

#[test]
fn file_resolution_preserves_order() {
    // Arrange
    let mut diagnostics = Diagnostics::new();
    let file_path_a = PathBuf::from("tests/files/a.slice");
    let file_path_b = PathBuf::from("tests/files/b.slice");
    let file_path_c = PathBuf::from("tests/files/c.slice");
    let file_path_test = PathBuf::from("tests/files/test.slice");

    let options = SliceOptions {
        sources: vec![
            file_path_a.to_str().unwrap().to_owned(),
            file_path_b.to_str().unwrap().to_owned(),
            file_path_c.to_str().unwrap().to_owned(),
        ],
        references: vec![
            file_path_b.to_str().unwrap().to_owned(), // Should be filtered out with no diagnostic.
            file_path_test.to_str().unwrap().to_owned(),
        ],
        ..Default::default()
    };

    // Act
    let files = resolve_files_from(&options, &mut diagnostics);

    // Assert
    assert!(diagnostics.is_empty());

    assert_eq!(files.len(), 4);
    assert_eq!(files[0].relative_path, "tests/files/a.slice");
    assert_eq!(files[1].relative_path, "tests/files/b.slice");
    assert_eq!(files[2].relative_path, "tests/files/c.slice");
    assert_eq!(files[3].relative_path, "tests/files/test.slice");
}

#[test]
fn compilation_preserves_order() {
    // Arrange
    let file_path_a = PathBuf::from("tests/files/a.slice");
    let file_path_b = PathBuf::from("tests/files/b.slice");
    let file_path_c = PathBuf::from("tests/files/c.slice");
    let file_path_test = PathBuf::from("tests/files/test.slice");

    let options = SliceOptions {
        sources: vec![
            file_path_a.to_str().unwrap().to_owned(),
            file_path_b.to_str().unwrap().to_owned(),
            file_path_c.to_str().unwrap().to_owned(),
        ],
        references: vec![
            file_path_b.to_str().unwrap().to_owned(), // Should be filtered out with no diagnostic.
            file_path_test.to_str().unwrap().to_owned(),
        ],
        ..Default::default()
    };

    // Act
    let state = slicec::compile_from_options(&options, |_| {}, |_| {});

    // Assert
    assert!(state.diagnostics.is_empty());

    let files = &state.files;
    assert_eq!(files.len(), 4);
    assert_eq!(files[0].relative_path, "tests/files/a.slice");
    assert_eq!(files[1].relative_path, "tests/files/b.slice");
    assert_eq!(files[2].relative_path, "tests/files/c.slice");
    assert_eq!(files[3].relative_path, "tests/files/test.slice");
}
