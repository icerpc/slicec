// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::error::ErrorReporter;
use slice::parse_from_string;

pub fn parse(slice: &str) -> ErrorReporter {
    let (_, error_reporter) = parse_from_string(slice).ok().unwrap();

    error_reporter
}

#[test]
fn no_inheritance_with_slice2() {
    let error_reporter = parse(
        "
encoding = 2;
module Test;
exception A {}
exception B : A {}",
    );

    error_reporter.assert_errors(&[
        "exception inheritance is only supported by the Slice 1 encoding",
        "file encoding was set to the Slice 2 encoding here:",
    ]);
}

#[test]
fn can_be_data_members_with_slice2() {
    let error_reporter = parse(
        "
encoding = 2;
module Test;
exception E {}
struct S
{
e: E,
} ",
    );

    error_reporter.assert_errors(&[]);
}

#[test]
#[ignore] // Encoding 1 with compact struct containing exceptions is not supported, compilation should fail
fn can_not_be_data_members_with_slice1() {
    // Arrange
    let error_reporter = parse(
        "
encoding = 1;
module Test;
exception E {}
compact struct S
{
e: E,
} ",
    );

    error_reporter.assert_errors(&[
        "exception inheritance is only supported by the Slice 1 encoding",
        "file encoding was set to the Slice 2 encoding here:",
    ]);
}
