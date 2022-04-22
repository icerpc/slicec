// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::parse_from_string;
use slice::error::ErrorReporter;

pub fn parse(slice: &str) -> ErrorReporter {
    let (_, error_reporter) = parse_from_string(slice).ok().unwrap();

    error_reporter
}


mod compact_structs {}

mod structs {
    use super::*;

    #[test]
    fn success_with_slice2_types() {
        let error_reporter = parse(
            "
encoding = 2;
module Test;
trait T;
struct A
{
    i: int32,
    s: string?,
    t: T,
}");

        error_reporter.assert_errors(&[]);
    }

    #[test]
    fn unsupported_with_slice1() {
        let error_reporter = parse(
            "
encoding = 1;
module Test;
struct A {}");

        error_reporter.assert_errors(&[
            "non-compact structs are not supported by the Slice 1 encoding",
            "file encoding was set to the Slice 1 encoding here:",
        ]);
    }

    #[test]
    fn unsupported_with_slice1_types() {
        let error_reporter = parse(
            "
encoding = 2;
module Test;
struct A
{
    c: AnyClass
}");

        error_reporter.assert_errors(&[
            "'AnyClass' is not supported by the Slice 2 encoding",
            "file encoding was set to the Slice 2 encoding here:",
        ]);
    }

    mod enums {
        use super::*;

        #[test]
        fn no_underlying_types_with_slice1() {
            let error_reporter = parse("
encoding = 1;
module Test;
enum E : int32 {}");

            error_reporter.assert_errors(&[
                "enums with underlying types are not supported by the Slice 1 encoding",
                "file encoding was set to the Slice 1 encoding here:",
            ]);
        }
    }
}