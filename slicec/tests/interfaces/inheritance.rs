// Copyright (c) ZeroC, Inc.

use crate::test_helpers::*;
use slicec::diagnostics::{Diagnostic, Error};
use slicec::grammar::*;
use test_case::test_case;

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
    let interface_i_def = ast.find_symbol_by_id::<Interface>("Test::I").unwrap();
    let interface_j_def = ast.find_symbol_by_id::<Interface>("Test::J").unwrap();
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
    let interface_i_def = ast.find_symbol_by_id::<Interface>("Test::I").unwrap();
    let interface_j_def = ast.find_symbol_by_id::<Interface>("Test::J").unwrap();
    let interface_k_def = ast.find_symbol_by_id::<Interface>("Test::K").unwrap();
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

#[test_case("uint8", "uint8"; "primitive")]
#[test_case("S", "struct"; "r#struct")]
#[test_case("Sequence<bool>", "sequence"; "sequence")]
#[test_case("Dictionary<int8, string>?", "dictionary"; "dictionary")]
#[test_case("Result<int32, int32>", "result"; "result")]
fn must_inherit_from_interface(base_type: &str, expected_kind: &str) {
    // Arrange
    let slice = format!(
        "
        module Test

        struct S {{}}

        interface I : {base_type} {{}}
        "
    );

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::from_error(Error::TypeMismatch {
        expected: "interface".to_owned(),
        actual: expected_kind.to_owned(),
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
    let expected = Diagnostic::from_error(Error::Shadows {
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
    let interface_a_def = ast.find_symbol_by_id::<Interface>("Test::A").unwrap();
    let interface_b_def = ast.find_symbol_by_id::<Interface>("Test::B").unwrap();
    let interface_d_def = ast.find_symbol_by_id::<Interface>("Test::D").unwrap();

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
