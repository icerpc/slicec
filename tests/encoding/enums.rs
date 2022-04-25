// Copyright (c) ZeroC, Inc. All rights reserved.

mod enums {
    use super::*;

    #[test]
    fn no_underlying_types_with_slice1() {
        let error_reporter = parse(
            "
encoding = 1;
module Test;
enum E : int32 {}",
        );

        error_reporter.assert_errors(&[
            "enums with underlying types are not supported by the Slice 1 encoding",
            "file encoding was set to the Slice 1 encoding here:",
        ]);
    }
}
