// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::diagnostics::{Warning, WarningKind};

pub mod macros;
pub mod parsing_helpers;

// This helper method is used for generating test spans for warnings. The `assert_errors!` macro currently does not
// verify if the spans of emitted diagnostics are correct. However, `Warning` requires a span to be constructed.
pub fn new_warning(kind: WarningKind) -> Warning {
    let span = slice::slice_file::Span {
        start: slice::slice_file::Location { row: 0, col: 0 },
        end: slice::slice_file::Location { row: 0, col: 0 },
        file: "string".to_string(),
    };
    Warning::new(kind).set_span(&span)
}
