// Copyright (c) ZeroC, Inc.

mod io;

use slicec::diagnostics::Diagnostics;
use slicec::slice_file::SliceFile;
use slicec::slice_options::SliceOptions;
use slicec::utils::file_util::resolve_files_from;
use std::path::PathBuf;

/// This test is used to verify that the `compute_sha256_hash` method for slices of `SliceFile` returns a hash that is
/// independent of the order of the files in the slice.
#[test]
fn fixed_slice_file_hash() {
    // Arrange
    let file1 = PathBuf::from("tests/files/test.slice").to_str().unwrap().to_owned();
    let file2 = PathBuf::from("tests/files/a.slice").to_str().unwrap().to_owned();
    let options1 = SliceOptions {
        sources: vec![file1.clone(), file2.clone()],
        ..Default::default()
    };
    let options2 = SliceOptions {
        sources: vec![file2, file1],
        ..Default::default()
    };
    let mut diagnostics = Diagnostics::new();
    let slice_files1 = resolve_files_from(&options1, &mut diagnostics);
    let slice_files2 = resolve_files_from(&options2, &mut diagnostics);

    // Act
    let hash1 = SliceFile::compute_sha256_hash(slice_files1.as_slice());
    let hash2 = SliceFile::compute_sha256_hash(slice_files2.as_slice());

    // Assert
    assert_eq!(hash1, hash2);
}
