// Copyright (c) ZeroC, Inc.

use crate::test_helpers::*;
use slicec::diagnostics::{Diagnostic, Error};
use slicec::grammar::*;
use test_case::test_case;

#[test_case("10", "expected one of 'identifier', 'doc comment', '[', or '}', but found '10'"; "numeric identifier")]
#[test_case("😊", "unknown symbol '😊'"; "unicode identifier")]
fn enumerator_invalid_identifiers(identifier: &str, expected_message: &str) {
    // Arrange
    let slice = format!(
        "
            module Test
            enum E : uint8 {{
                {identifier}
            }}
        "
    );

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::new(Error::Syntax {
        message: expected_message.to_owned(),
    });
    check_diagnostics(diagnostics, [expected]);
}

mod associated_fields {
    use super::*;
    use test_case::test_case;

    #[test]
    fn enumerator_fields_can_be_tagged() {
        // Arrange
        let slice = "
        module Test
        enum E {
            A(tag(1) b: bool?),
        }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let field = ast.find_element::<Field>("Test::E::A::b").unwrap();
        assert!(field.is_tagged());
    }

    #[test]
    fn tags_are_disallowed_in_compact_enums() {
        // Arrange
        let slice = "
        module Test
        compact enum E {
            A(tag(1) b: bool?),
        }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::CompactTypeCannotContainTaggedFields { kind: "enum" })
            .add_note("enum 'E' is declared compact here", None);

        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn explicit_values_are_allowed() {
        // Arrange
        let slice = "
            module Test
            enum E {
                A
                B = 7
                C(a: int8)
                D(b: bool) = 4
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let enumerator_a = ast.find_element::<Enumerator>("Test::E::A").unwrap();
        assert!(matches!(enumerator_a.value, EnumeratorValue::Implicit(0)));
        assert_eq!(enumerator_a.value(), 0);

        let enumerator_b = ast.find_element::<Enumerator>("Test::E::B").unwrap();
        assert!(matches!(enumerator_b.value, EnumeratorValue::Explicit(_)));
        assert_eq!(enumerator_b.value(), 7);

        let enumerator_c = ast.find_element::<Enumerator>("Test::E::C").unwrap();
        assert!(matches!(enumerator_c.value, EnumeratorValue::Implicit(8)));
        assert_eq!(enumerator_c.value(), 8);

        let enumerator_d = ast.find_element::<Enumerator>("Test::E::D").unwrap();
        assert!(matches!(enumerator_d.value, EnumeratorValue::Explicit(_)));
        assert_eq!(enumerator_d.value(), 4);
    }

    #[test]
    fn explicit_values_must_be_within_range() {
        // Arrange
        let slice = "
            module Test
            enum E {
                ImplicitOkay                           //  0
                ExplicitNegative = -3                  // -3
                ImplicitNegative(tag(4) s: string?)    // -2
                Okay(b: bool) = 2_147_483_647          // 2_147_483_647
                ImplicitOverflow                       // 2_147_483_648
                ExplicitOverflow = 0x686921203A7629    // something big
                ExplicitOkay(a: int8) = 79             // 79
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Arrange
        let expected = [
            Diagnostic::new(Error::EnumeratorValueOutOfBounds {
                enumerator_identifier: "ExplicitNegative".to_owned(),
                value: -3,
                min: 0,
                max: i32::MAX as i128,
            }),
            Diagnostic::new(Error::EnumeratorValueOutOfBounds {
                enumerator_identifier: "ImplicitNegative".to_owned(),
                value: -2,
                min: 0,
                max: i32::MAX as i128,
            }),
            Diagnostic::new(Error::EnumeratorValueOutOfBounds {
                enumerator_identifier: "ImplicitOverflow".to_owned(),
                value: 2_147_483_648,
                min: 0,
                max: i32::MAX as i128,
            }),
            Diagnostic::new(Error::EnumeratorValueOutOfBounds {
                enumerator_identifier: "ExplicitOverflow".to_owned(),
                value: 0x686921203A7629,
                min: 0,
                max: i32::MAX as i128,
            }),
        ];
        check_diagnostics(diagnostics, expected);
    }

    #[test]
    fn associated_fields_are_scoped_correctly() {
        // Arrange
        let slice = "
            module Test

            enum Foo {
                Bar(baz: Sequence<bool>)
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        assert!(ast.find_element::<Field>("Test::Foo::Bar::baz").is_ok());
    }

    #[test]
    fn associated_fields_are_parsed_correctly() {
        // Arrange
        let slice = "
            module Test

            enum E {
                A
                B(b: bool)
                C(i: int32, tag(2) s: string?)
                D()
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let a = ast.find_element::<Enumerator>("Test::E::A").unwrap();
        assert!(matches!(a.value, EnumeratorValue::Implicit(0)));
        assert!(a.fields.is_none());

        let b = ast.find_element::<Enumerator>("Test::E::B").unwrap();
        assert!(matches!(b.value, EnumeratorValue::Implicit(1)));
        assert!(b.fields.as_ref().unwrap().len() == 1);

        let c = ast.find_element::<Enumerator>("Test::E::C").unwrap();
        assert!(matches!(c.value, EnumeratorValue::Implicit(2)));
        assert!(c.fields.as_ref().unwrap().len() == 2);

        let d = ast.find_element::<Enumerator>("Test::E::D").unwrap();
        assert!(matches!(d.value, EnumeratorValue::Implicit(3)));
        assert!(d.fields.as_ref().unwrap().len() == 0);
    }

    #[test_case("unchecked enum", true ; "unchecked")]
    #[test_case("enum", false ; "checked")]
    fn test_presence_of_unchecked(enum_definition: &str, expected: bool) {
        // Arrange
        let slice = format!(
            "
                module Test
                {enum_definition} E {{
                    A
                    B
                }}
            "
        );

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let enum_def = ast.find_element::<Enum>("Test::E").unwrap();
        assert_eq!(enum_def.is_unchecked, expected);
    }

    #[test]
    fn checked_enums_can_not_be_empty() {
        // Arrange
        let slice = "
            module Test

            enum E {}
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::MustContainEnumerators {
            enum_identifier: "E".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn unchecked_enums_can_be_empty() {
        // Arrange
        let slice = "
            module Test

            unchecked enum E {}
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let enum_def = ast.find_element::<Enum>("Test::E").unwrap();
        assert_eq!(enum_def.enumerators.len(), 0);
    }

    #[test]
    fn cannot_redefine_enumerators() {
        // Arrange
        let slice = "
            module Test

            enum E  { A, A }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::Redefinition {
            identifier: "A".to_string(),
        })
        .add_note("'A' was previously defined here", None);

        check_diagnostics(diagnostics, [expected]);
    }
}

mod underlying_type {
    use super::*;
    use test_case::test_case;

    #[test]
    fn associated_fields_are_not_allowed() {
        // Arrange
        let slice = "
            module Test
            enum E: uint8 {
                A
                B(b: bool)
                C
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::EnumeratorCannotContainFields {
            enumerator_identifier: "B".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn enumerator_default_values() {
        // Arrange
        let slice = "
            module Test
            enum E : uint8 {
                A
                B
                C
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let enumerators = ast.find_element::<Enum>("Test::E").unwrap().enumerators();
        assert_eq!(enumerators[0].value(), 0);
        assert_eq!(enumerators[1].value(), 1);
        assert_eq!(enumerators[2].value(), 2);
    }

    #[test]
    fn subsequent_unsigned_value_is_incremented_previous_value() {
        // Arrange
        let slice = "
            module Test
            enum E : uint8 {
                A = 2
                B
                C
            }
            ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let enumerators = ast.find_element::<Enum>("Test::E").unwrap().enumerators();
        assert_eq!(enumerators[1].value(), 3);
        assert_eq!(enumerators[2].value(), 4);
    }

    #[test]
    fn implicit_enumerator_values_overflow_cleanly() {
        // Arrange
        let slice = "
            module Test
            enum E : varint32 {
                A
                B = 170141183460469231731687303715884105727 // i128::MAX
                C
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = [
            Diagnostic::new(Error::EnumeratorValueOutOfBounds {
                enumerator_identifier: "B".to_owned(),
                value: i128::MAX,
                min: -2147483648,
                max: 2147483647,
            }),
            Diagnostic::new(Error::EnumeratorValueOutOfBounds {
                enumerator_identifier: "C".to_owned(),
                value: i128::MIN,
                min: -2147483648,
                max: 2147483647,
            }),
        ];
        check_diagnostics(diagnostics, expected);
    }

    #[test]
    fn enumerator_values_can_be_out_of_order() {
        // Arrange
        let slice = "
                module Test
                enum E : uint8 {
                    A = 2
                    B = 1
                }
            ";

        // Act/Assert
        assert_parses(slice);
    }

    #[test]
    fn validate_backing_type_out_of_bounds() {
        // Arranges
        let out_of_bounds_value = i16::MAX as i128 + 1;
        let slice = format!(
            "
                module Test
                enum E : int16 {{
                    A = {out_of_bounds_value}
                }}
            "
        );

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::EnumeratorValueOutOfBounds {
            enumerator_identifier: "A".to_owned(),
            value: out_of_bounds_value,
            min: -32768_i128,
            max: 32767_i128,
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn validate_backing_type_bounds() {
        // Arranges
        let min = i16::MIN;
        let max = i16::MAX;
        let slice = format!(
            "
                module Test
                enum E : int16 {{
                    A = {min}
                    B = {max}
                }}
            "
        );

        // Act/Assert
        assert_parses(slice);
    }

    #[test]
    fn enumerators_must_have_unique_values() {
        // Arrange
        let slice = "
            module Test

            enum E : uint8 {
                A = 1
                B = 1
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::DuplicateEnumeratorValue { enumerator_value: 1 })
            .add_note("the value was previously used by 'A' here:", None);

        check_diagnostics(diagnostics, [expected]);
    }

    #[test_case("unchecked enum", true ; "unchecked")]
    #[test_case("enum", false ; "checked")]
    fn test_presence_of_unchecked(enum_definition: &str, expected: bool) {
        // Arrange
        let slice = format!(
            "
                module Test
                {enum_definition} E : uint8 {{
                    A
                    B
                }}
            "
        );

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let enum_def = ast.find_element::<Enum>("Test::E").unwrap();
        assert_eq!(enum_def.is_unchecked, expected);
    }

    #[test]
    fn checked_enums_can_not_be_empty() {
        // Arrange
        let slice = "
            module Test

            enum E : uint8 {}
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::MustContainEnumerators {
            enum_identifier: "E".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn unchecked_enums_can_be_empty() {
        // Arrange
        let slice = "
            module Test

            unchecked enum E : uint8 {}
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let enum_def = ast.find_element::<Enum>("Test::E").unwrap();
        assert_eq!(enum_def.enumerators.len(), 0);
    }

    #[test]
    fn enumerator_values_support_different_base_literals() {
        // Arrange
        let slice = "
            module Test

            enum E : varint32 {
                B = 0b1001111
                D = 128
                H = 0xA4FD
                N = -0xbc81
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        assert_eq!(ast.find_element::<Enumerator>("Test::E::B").unwrap().value(), 0b1001111);
        assert_eq!(ast.find_element::<Enumerator>("Test::E::D").unwrap().value(), 128);
        assert_eq!(ast.find_element::<Enumerator>("Test::E::H").unwrap().value(), 0xA4FD);
        assert_eq!(ast.find_element::<Enumerator>("Test::E::N").unwrap().value(), -0xbc81);
    }

    #[test]
    fn duplicate_enumerator_values_are_disallowed_across_different_bases() {
        // Arrange
        let slice = "
            module Test

            enum E : uint16 {
                B = 0b1001111
                D = 79
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::DuplicateEnumeratorValue { enumerator_value: 79 });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn cannot_redefine_enumerators() {
        // Arrange
        let slice = "
            module Test

            enum E : uint32 {
                A, A
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::Redefinition {
            identifier: "A".to_string(),
        })
        .add_note("'A' was previously defined here", None);

        check_diagnostics(diagnostics, [expected]);
    }

    mod slice1 {

        use crate::test_helpers::*;
        use slicec::diagnostics::{Diagnostic, Error};

        #[test]
        fn enumerators_cannot_contain_negative_values() {
            // Arrange
            let slice = "
                mode = Slice1
                module Test

                enum E {
                    A = -1
                    B = -2
                    C = -3
                }
            ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            const MAX_VALUE: i128 = i32::MAX as i128;
            let expected = [
                Diagnostic::new(Error::EnumeratorValueOutOfBounds {
                    enumerator_identifier: "A".to_owned(),
                    value: -1,
                    min: 0,
                    max: MAX_VALUE,
                }),
                Diagnostic::new(Error::EnumeratorValueOutOfBounds {
                    enumerator_identifier: "B".to_owned(),
                    value: -2,
                    min: 0,
                    max: MAX_VALUE,
                }),
                Diagnostic::new(Error::EnumeratorValueOutOfBounds {
                    enumerator_identifier: "C".to_owned(),
                    value: -3,
                    min: 0,
                    max: MAX_VALUE,
                }),
            ];
            check_diagnostics(diagnostics, expected);
        }

        #[test]
        fn enumerators_cannot_contain_out_of_bound_values() {
            // Arrange
            let value = i32::MAX as i128 + 1;
            let slice = format!(
                "
                    mode = Slice1
                    module Test

                    enum E {{
                        A = {value}
                    }}
                "
            );

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Error::EnumeratorValueOutOfBounds {
                enumerator_identifier: "A".to_owned(),
                value,
                min: 0,
                max: i32::MAX as i128,
            });
            check_diagnostics(diagnostics, [expected]);
        }
    }

    mod slice2 {

        use crate::test_helpers::*;
        use slicec::grammar::*;

        #[test]
        fn enumerators_can_contain_negative_values() {
            // Arrange
            let slice = "
                module Test

                enum E : int32 {
                    A = -1
                    B = -2
                    C = -3
                }
            ";

            // Act/Assert
            assert_parses(slice);
        }

        #[test]
        fn enumerators_can_contain_values() {
            // Arrange
            let slice = "
                module Test

                enum E : int16 {
                    A = 1
                    B = 2
                    C = 3
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let enum_def = ast.find_element::<Enum>("Test::E").unwrap();
            let enumerators = enum_def.enumerators();

            assert_eq!(enumerators.len(), 3);
            assert_eq!(enumerators[0].identifier(), "A");
            assert_eq!(enumerators[1].identifier(), "B");
            assert_eq!(enumerators[2].identifier(), "C");
            assert_eq!(enumerators[0].value(), 1);
            assert_eq!(enumerators[1].value(), 2);
            assert_eq!(enumerators[2].value(), 3);
            assert!(matches!(
                enum_def.underlying.as_ref().unwrap().definition(),
                Primitive::Int16,
            ));
        }

        #[test]
        fn explicit_enumerator_value_kinds() {
            let slice = "
            module Test

            enum A : uint8 {
                u = 1
                v = 2
                w = 3
            }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let enum_def_a = ast.find_element::<Enum>("Test::A").unwrap();
            let enumerators_a = enum_def_a.enumerators();

            assert!(matches!(enumerators_a[0].value, EnumeratorValue::Explicit(..)));
            assert!(matches!(enumerators_a[1].value, EnumeratorValue::Explicit(..)));
            assert!(matches!(enumerators_a[2].value, EnumeratorValue::Explicit(..)));
        }
    }
}
