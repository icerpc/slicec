// Copyright (c) ZeroC, Inc.

use slice::diagnostics::{Diagnostic, Error};
use slice::grammar::Encoding;
use slice::test_helpers::*;

#[test]
fn operation_members_are_compatible_with_encoding() {
    // Arrange
    let slice1 = "
        encoding = Slice1
        module Test

        class C {}
    ";
    let slice2 = "
        encoding = Slice2
        module Test

        interface I {
            op(c: C)
        }
    ";

    // Act
    let diagnostics = parse_multiple_for_diagnostics(&[slice1, slice2]);

    // Assert
    let expected = Diagnostic::new(Error::UnsupportedType {
        kind: "C".to_owned(),
        encoding: Encoding::Slice2,
    })
    .add_note("file encoding was set to Slice2 here:", None);

    check_diagnostics(diagnostics, [expected]);
}

#[test]
fn any_exception_cannot_be_used_without_slice1() {
    let slice = "
        module Test

        interface I {
            op() throws AnyException
        }
    ";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::new(Error::AnyExceptionNotSupported);
    check_diagnostics(diagnostics, [expected]);
}

mod slice1 {
    use slice::grammar::*;
    use slice::test_helpers::*;

    #[test]
    fn operations_can_throw_any_exception() {
        let slice = "
            encoding = Slice1
            module Test

            interface I {
                op() throws AnyException
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let operation = ast.find_element::<Operation>("Test::I::op").unwrap();
        assert!(matches!(&operation.throws, Throws::AnyException));
    }
}
