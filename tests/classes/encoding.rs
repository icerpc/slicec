// Copyright (c) ZeroC, Inc. All rights reserved.

mod slice2 {

    use crate::assert_errors_new;
    use crate::helpers::parsing_helpers::parse_for_errors;
    use slice::errors::*;

    #[test]
    fn unsupported_error() {
        let slice = "
            module Test;
            class C {}
        ";
        let expected: [&dyn ErrorType; 4] = [
            &RuleKind::from(InvalidEncodingKind::NotSupported {
                kind: "class".to_owned(),
                identifier: "C".to_owned(),
                encoding: "2".to_owned(),
            }),
            &Note::new("file is using the Slice2 encoding by default"),
            &Note::new("to use a different encoding, specify it at the top of the slice file\nex: 'encoding = 1;'"),
            &Note::new("classes are only supported by the Slice1 encoding"),
        ];

        let error_reporter = parse_for_errors(slice);

        assert_errors_new!(error_reporter, expected);
    }
}
