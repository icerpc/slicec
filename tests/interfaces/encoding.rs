// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::assert_errors;
use slice::parse_from_strings;

#[test]
fn operation_members_are_compatible_with_encoding() {
    let slice1 = "
        encoding = 1;
        module Test;
        class C {}
        ";

    let slice2 = "
        encoding = 2;
        module Test;
        interface I {
            op(c: C);
        }
    ";

    let result = parse_from_strings(&[slice1, slice2]).err().unwrap();

    assert_errors!(result.error_reporter, [
        "operation 'op' contains members that are not compatible with its encoding (Slice 2).",
        "file encoding was set to Slice2 here:"
    ]);
}
