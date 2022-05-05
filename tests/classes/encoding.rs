// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::assert_errors;
use crate::helpers::parsing_helpers::parse_for_errors;

mod slice2 {
    use super::*;

    #[test]
    fn unsupported_error() {
        let slice = "
            module Test;
            class C {}
        ";

        let error_reporter = parse_for_errors(slice);

        assert_errors!(error_reporter, &[
            "classes are only supported by the Slice 1 encoding",
            "file encoding was set to the Slice 2 encoding here:",
        ]);
    }
}
