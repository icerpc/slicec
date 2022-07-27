// Copyright (c) ZeroC, Inc. All rights reserved.

mod slice2 {

    use crate::assert_errors_new;
    use crate::helpers::parsing_helpers::parse_for_errors;
    use slice::errors::*;
    use slice::grammar::Encoding;

    #[test]
    fn unsupported_error() {
        let slice = "
            module Test;
            class C {}
        ";
        let expected = [
            RuleKind::NotSupportedWithEncoding("class".to_owned(), "C".to_owned(), Encoding::Slice2).into(),
            ErrorKind::new("file is using the Slice2 encoding by default".to_owned()),
            ErrorKind::new(
                "to use a different encoding, specify it at the top of the slice file\nex: 'encoding = 1;'".to_owned(),
            ),
            ErrorKind::new("classes are only supported by the Slice1 encoding".to_owned()),
        ];

        let error_reporter = parse_for_errors(slice);

        assert_errors_new!(error_reporter, expected);
    }
}
