// Copyright (c) ZeroC, Inc.

mod slice1 {

    use crate::test_helpers::*;
    use slicec::diagnostics::{Diagnostic, Error};
    use slicec::grammar::CompilationMode;

    /// Verifies using the slice parser with Slice1 will emit errors when parsing
    /// non-compact structs.
    #[test]
    fn unsupported_fail() {
        // Arrange
        let slice = "
            mode = Slice1
            module Test

            struct A {}
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::NotSupportedInCompilationMode {
            kind: "struct".to_owned(),
            identifier: "A".to_owned(),
            mode: CompilationMode::Slice1,
        })
        .add_note("structs defined in Slice1 mode must be 'compact'", None);

        check_diagnostics(diagnostics, [expected]);
    }
}

mod slice2 {

    use crate::test_helpers::*;
    use slicec::diagnostics::{Diagnostic, Error};
    use slicec::grammar::CompilationMode;

    /// Verifies using the slice parser with Slice2 will emit errors when parsing
    /// structs that contain Slice1 types.
    #[test]
    fn slice1_types_fail() {
        // Arrange
        let slice = "
            module Test

            struct A {
                c: AnyClass
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::UnsupportedType {
            kind: "AnyClass".to_owned(),
            mode: CompilationMode::Slice2,
        })
        .add_note("this file's compilation mode is Slice2 by default", None);

        check_diagnostics(diagnostics, [expected]);
    }

    /// Verifies using the slice parser with Slice2 will not emit errors when parsing
    /// structs that contain Slice2 types.
    #[test]
    fn slice2_types_succeed() {
        // Arrange
        let slice = "
            module Test

            struct A {
                i: int32
                s: string?
            }
        ";

        // Act/Assert
        assert_parses(slice);
    }
}
