// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::assert_errors;
use crate::helpers::parsing_helpers::*;
use slice::compile_from_strings;
use slice::diagnostics::{Error, ErrorKind};
use slice::grammar::Encoding;

#[test]
fn operation_members_are_compatible_with_encoding() {
    // Arrange
    let slice1 = "
        encoding = 1;
        module Test;

        class C
        {
        }
    ";
    let slice2 = "
        encoding = 2;
        module Test;

        interface I
        {
            op(c: C);
        }
    ";

    // Act
    let result = compile_from_strings(&[slice1, slice2], None).err().unwrap();

    // Assert
    let expected = Error::new(ErrorKind::UnsupportedType("C".to_owned(), Encoding::Slice2))
        .add_note("file encoding was set to Slice2 here:", None);

    assert_errors!(result.diagnostic_reporter, [&expected]);
}

#[test]
fn anyexception_cannot_be_used_without_slice1() {
    let slice = "
        module Test;

        interface I
        {
            op() throws AnyException;
        }
    ";

    // Act
    let diagnostic_reporter = parse_for_diagnostics(slice);

    // Assert
    let expected = Error::new(ErrorKind::AnyExceptionNotSupported);
    assert_errors!(diagnostic_reporter, [&expected]);
}

mod slice1 {
    use crate::helpers::parsing_helpers::*;
    use slice::grammar::*;

    #[test]
    fn operations_can_throw_anyexception() {
        let slice = "
            encoding = 1;

            module Test;

            interface I
            {
                op() throws AnyException;
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let operation = ast.find_element::<Operation>("Test::I::op").unwrap();
        assert!(matches!(&operation.throws, Throws::AnyException));
    }
}
