// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::parse_from_string;
use slice::error::ErrorReporter;

pub fn parse(slice: &str) -> ErrorReporter {
    let (_, error_reporter) = parse_from_string(slice).ok().unwrap();

    error_reporter
}


// Notes?
// - uint8 causes compiler to crash in cycle_detection
// - encoding = 1; compact struct S { v: } complains about int8 not being supported
