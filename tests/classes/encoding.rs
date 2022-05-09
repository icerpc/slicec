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

        assert_errors!(error_reporter, &[
            "classes are only supported by the Slice 1 encoding",
            "file is using the Slice 2 encoding by default",
            "to use a different encoding, specify it at the top of the slice file\nex: 'encoding = 1;'"
        ]);
    }
}
