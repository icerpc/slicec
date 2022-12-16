// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::command_line::SliceOptions;
use slice::diagnostics::DiagnosticReporter;
use slice::utils::file_util::resolve_files_from;

#[test]
fn duplicate_files_ignored() {
    // Arrange
    let options = SliceOptions {
        sources: vec!["tests/files/test.slice".to_owned()],
        references: vec!["tests/files/test.slice".to_owned()],
        ..Default::default()
    };
    let mut reporter = DiagnosticReporter::new(&options);

    // Act
    let files = resolve_files_from(&options, &mut reporter);

    // Assert
    assert_eq!(files.len(), 1);
}
