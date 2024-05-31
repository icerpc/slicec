// Copyright (c) ZeroC, Inc.

mod io;

use slicec::diagnostics::Diagnostics;
use slicec::slice_file::SliceFileHashable;
use slicec::slice_options::SliceOptions;
use slicec::utils::file_util::resolve_files_from;
use std::path::PathBuf;

/// This test is used to verify that the `hash_all` method returns a hash that is stable across releases and uses
/// the correct ordering of files.
#[test]
fn fixed_slice_file_hash() {
    // Arrange
    let fixed_hash = "df7e2e0f34d0c8d389870dad726c4d8cacc544c24f9d4516c38c88783eaac20c";
    let mut diagnostics = Diagnostics::new();
    let file1 = PathBuf::from("tests/files/test.slice");
    let file2 = PathBuf::from("tests/files/a.slice");
    let options = SliceOptions {
        sources: vec![file1.to_str().unwrap().to_owned(), file2.to_str().unwrap().to_owned()],
        ..Default::default()
    };
    let files = resolve_files_from(&options, &mut diagnostics);

    // Act
    let hash = files.as_slice().compute_sha256_hash();

    // Assert
    assert_eq!(hash, fixed_hash);
}
