// Copyright (c) ZeroC, Inc.

use crate::test_helpers::*;
use slice::diagnostics::{Error, ErrorKind};
use slice::grammar::*;

#[test]
fn supports_single_inheritance() {
    // Arrange
    let slice = "
        module Test

        interface I {}

        interface J : I {}
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let interface_i_def = ast.find_element::<Interface>("Test::I").unwrap();
    let interface_j_def = ast.find_element::<Interface>("Test::J").unwrap();
    let interface_j_bases = interface_j_def.base_interfaces();

    assert!(interface_i_def.base_interfaces().is_empty());
    assert_eq!(interface_j_bases.len(), 1);
    assert_eq!(
        interface_j_bases[0].module_scoped_identifier(),
        interface_i_def.module_scoped_identifier(),
    );
}

#[test]
fn supports_multiple_inheritance() {
    // Arrange
    let slice = "
        module Test

        interface I {}

        interface J {}

        interface K : I, J {}
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let interface_i_def = ast.find_element::<Interface>("Test::I").unwrap();
    let interface_j_def = ast.find_element::<Interface>("Test::J").unwrap();
    let interface_k_def = ast.find_element::<Interface>("Test::K").unwrap();
    let interface_k_bases = interface_k_def.base_interfaces();

    assert!(interface_i_def.base_interfaces().is_empty());
    assert!(interface_j_def.base_interfaces().is_empty());
    assert_eq!(interface_k_bases.len(), 2);
    assert_eq!(
        interface_k_bases[0].module_scoped_identifier(),
        interface_i_def.module_scoped_identifier(),
    );
    assert_eq!(
        interface_k_bases[1].module_scoped_identifier(),
        interface_j_def.module_scoped_identifier(),
    );
}

#[test]
fn must_inherit_from_interface() {
    // Arrange
    let slice = "
        encoding = 1
        module Test

        class C {}

        interface I : C {}
    ";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Error::new(ErrorKind::TypeMismatch {
        expected: "interface".to_owned(),
        actual: "class".to_owned(),
        is_concrete: true,
    });
    check_diagnostics(diagnostics, [expected]);
}

#[test]
fn operation_shadowing_is_disallowed() {
    // Arrange
    let slice = "
        module Test

        interface I {
            op()
        }

        interface J : I {
            op()
        }
    ";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Error::new(ErrorKind::Shadows {
        identifier: "op".to_owned(),
    })
    .add_note("'op' was previously defined here", None);

    check_diagnostics(diagnostics, [expected]);
}

#[test]
fn inherits_correct_operations() {
    // Arrange
    let slice = "
        module Test

        interface A {
            opA()
        }

        interface B : A {
            opB()
        }

        interface C : A {}

        interface D : B, C {
            opD()
        }
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let interface_a_def = ast.find_element::<Interface>("Test::A").unwrap();
    let interface_b_def = ast.find_element::<Interface>("Test::B").unwrap();
    let interface_d_def = ast.find_element::<Interface>("Test::D").unwrap();

    assert_eq!(interface_a_def.operations().len(), 1);
    assert_eq!(interface_a_def.all_inherited_operations().len(), 0);
    assert_eq!(interface_a_def.all_operations().len(), 1);
    assert_eq!(interface_a_def.operations()[0].identifier(), "opA");

    assert_eq!(interface_b_def.operations().len(), 1);
    assert_eq!(interface_b_def.all_inherited_operations().len(), 1);
    assert_eq!(interface_b_def.all_operations().len(), 2);
    assert_eq!(interface_b_def.operations()[0].identifier(), "opB");
    assert_eq!(interface_b_def.all_inherited_operations()[0].identifier(), "opA");

    assert_eq!(interface_d_def.operations().len(), 1);
    assert_eq!(interface_d_def.all_inherited_operations().len(), 2);
    assert_eq!(interface_d_def.all_operations().len(), 3);
    assert_eq!(interface_d_def.operations()[0].identifier(), "opD");

    assert_eq!(interface_d_def.all_inherited_operations()[0].identifier(), "opB");
    assert_eq!(interface_d_def.all_inherited_operations()[1].identifier(), "opA");
}
