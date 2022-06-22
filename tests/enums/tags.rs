// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::assert_errors;
use crate::helpers::parsing_helpers::parse_for_errors;

#[test]
// TODO, should we? This is just a syntax error in my opinion. There isn't even a type to tag here.
#[should_panic] // TODO: We would have error messages explaining that you cannot have tags on enums.
fn cannot_contain_tags() {
    // Arrange
    let slice = "
        module Test;
        enum E: int32 {
            A = 1,
            B: tag(10) = 2,
        }
    ";

    // Act & Assert
    assert_errors!(parse_for_errors(slice));
}
