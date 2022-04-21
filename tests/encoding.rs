// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::parse_from_string;
use slice::error::ErrorReporter;

fn parse(slice: &str) -> ErrorReporter {
    let (_, error_reporter) = parse_from_string(slice).ok().unwrap();

    error_reporter
}

mod exceptions {
    use super::*;

    #[test]
    fn no_inheritance_with_slice2() {
        let error_reporter = parse(
            "
encoding = 2;
module Test;
exception A {}
exception B : A {}");

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
} ");

        error_reporter.assert_errors(&[]);
    }

    #[test]
    #[ignore]
    fn can_not_be_data_members_with_slice1() {
        let error_reporter = parse(
            "
encoding = 1;
module Test;
exception E {}
compact struct S
{
    e: E,
} ");

        error_reporter.assert_errors(&[
            "exception inheritance is only supported by the Slice 1 encoding",
            "file encoding was set to the Slice 2 encoding here:",
        ]);
    }

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

    mod builtin_types {
        use super::*;
        use std::collections::HashMap;

        #[test]
        #[ignore]
        fn unsupported_types() {
            let types = HashMap::from([
                ("1", vec!["varint32"]),
                ("2", vec!["AnyClass"])
            ]);

            for (encoding, values) in types {
                for value in values {
                    let error_reporter = parse(&format!("
encoding = {encoding};
module Test;
compact struct S
{{
    v: {value},
}}",
                        encoding = encoding,
                        value = value,
                    ));

                    error_reporter.assert_errors(&[
                        &format!("'{}' is not supported by the Slice {} encoding", value, encoding),
                        &format!("file encoding was set to the Slice {} encoding here:", encoding),
                    ]);
                }
            }
        }
    }
}


// Notes?
// - uint8 causes compiler to crash in cycle_detection
// - encoding = 1; compact struct S { v: } complains about int8 not being supported
