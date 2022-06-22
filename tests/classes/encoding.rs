// Copyright (c) ZeroC, Inc. All rights reserved.

mod slice2 {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::parse_for_errors;

    #[test]
    fn unsupported_error() {
        let slice = "
            module Test;
            class C {}
        ";

        let error_reporter = parse_for_errors(slice);

        assert_errors!(error_reporter, [
            "class `C` is not supported by the Slice2 encoding",
            "file is using the Slice2 encoding by default",
            "to use a different encoding, specify it at the top of the slice file\nex: 'encoding = 1;'",
            "classes are only supported by the Slice1 encoding",
        ]);
    }
}
