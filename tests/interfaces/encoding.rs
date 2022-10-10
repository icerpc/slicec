// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::assert_errors;
use slice::diagnostics::{Error, ErrorKind, Note};
use slice::grammar::Encoding;
use slice::parse_from_strings;

#[test]
fn operation_members_are_compatible_with_encoding() {
    // Arrange
    let slice1 = "
        encoding = 1;
        module Test;

        class C
        {
        }
    ";
    let slice2 = "
        encoding = 2;
        module Test;

        interface I
        {
            op(c: C);
        }
    ";

    // Act
    let result = parse_from_strings(&[slice1, slice2], None).err().unwrap();

    // Assert
    let expected = Error::new_with_notes(
        ErrorKind::UnsupportedType("C".to_owned(), Encoding::Slice2),
        None,
        vec![Note::new("file encoding was set to Slice2 here:", None)],
    );
    assert_errors!(result.diagnostic_reporter, [&expected]);
}
