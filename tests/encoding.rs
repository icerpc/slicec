// Copyright (c) ZeroC, Inc. All rights reserved.

mod exceptions {
    use slice::parse_from_string;

    #[test]
    fn no_inheritance_with_slice2() {
        let (_, error_reporter) = parse_from_string(
            "
encoding = 2;
module Test;
exception A {}
exception B : A {}",
        )
        .ok()
        .unwrap();

        error_reporter.assert_errors(&[
            "exception inheritance is only supported by the Slice 1 encoding",
            "file encoding was set to the Slice 2 encoding here:",
        ]);
    }
}

mod compact_structs {}

mod structs {
    use slice::parse_from_string;

    #[test]
    fn success_with_slice2_types() {
        let (_, error_reporter) = parse_from_string(
            "
encoding = 2;
module Test;
trait T;
struct A
{
    i: int32,
    s: string?,
    t: T,
}",
        )
        .ok()
        .unwrap();

        error_reporter.assert_errors(&[]);
    }

    #[test]
    fn unsupported_with_slice1() {
        let (_, error_reporter) = parse_from_string(
            "
encoding = 1;
module Test;
struct A {}",
        )
        .ok()
        .unwrap();

        error_reporter.assert_errors(&[
            "non-compact structs are not supported by the Slice 1 encoding",
            "file encoding was set to the Slice 1 encoding here:",
        ]);
    }

    #[test]
    fn unsupported_with_slice1_types() {
        let (_, error_reporter) = parse_from_string(
            "
encoding = 2;
module Test;
struct A
{
    c: AnyClass
}",
        )
        .ok()
        .unwrap();

        error_reporter.assert_errors(&[
            "'any class' is not supported by the Slice 2 encoding",
            "file encoding was set to the Slice 2 encoding here:",
        ]);
    }
}
