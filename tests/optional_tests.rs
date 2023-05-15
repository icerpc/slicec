// Copyright (c) ZeroC, Inc.

mod optional {
    use slice::diagnostics::{Diagnostic, Error};
    use slice::grammar::*;
    use slice::slice_file::Span;
    use slice::test_helpers::*;
    use test_case::test_case;

    #[test_case("bool"; "primitive")]
    #[test_case("Foo"; "simple")]
    #[test_case("Test::Foo"; "relatively scoped")]
    #[test_case("::Test::Foo"; "globally scoped")]
    fn optionals_are_parsed_correctly(type_name: &str) {
        // Arrange
        let slice = format!(
            "
            module Test
            struct Foo {{}}
            exception E {{
                a: {type_name}?
            }}
            "
        );

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let field = ast.find_element::<Field>("Test::E::a").unwrap();
        assert!(field.data_type.is_optional);
    }

    #[test_case("bool"; "primitive")]
    #[test_case("Foo"; "user defined")]
    fn optional_type_names_end_with_a_question_mark(type_name: &str) {
        // Arrange
        let slice = format!(
            "
            module Test
            struct Foo {{}}
            exception E {{
                a: {type_name}?
            }}
            "
        );

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let field = ast.find_element::<Field>("Test::E::a").unwrap();
        assert_eq!(field.data_type.type_string(), type_name.to_owned() + "?");
    }

    mod slice1 {
        use super::*;
        use test_case::test_case;

        #[test_case("AnyClass")]
        fn optional_builtin_types_are_allowed(type_name: &str) {
            // Arrange
            let slice = format!(
                "
                encoding = Slice1
                module Test
                exception E {{
                    a: {type_name}?
                }}
                "
            );

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let field = ast.find_element::<Field>("Test::E::a").unwrap();
            assert!(field.data_type.is_optional);
        }

        #[test_case("bool")]
        #[test_case("int8")]
        #[test_case("uint8")]
        #[test_case("int16")]
        #[test_case("uint16")]
        #[test_case("int32")]
        #[test_case("uint32")]
        #[test_case("varint32")]
        #[test_case("varuint32")]
        #[test_case("int64")]
        #[test_case("uint64")]
        #[test_case("varint62")]
        #[test_case("varuint62")]
        #[test_case("float32")]
        #[test_case("float64")]
        #[test_case("string")]
        fn optional_builtin_types_are_disallowed(type_name: &str) {
            // Arrange
            let slice = format!(
                "
                encoding = Slice1
                module Test
                exception E {{
                    a: {type_name}?
                }}
                "
            );

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Error::OptionalsNotSupported {
                encoding: Encoding::Slice1,
            })
            .set_span(&Span::new(
                (5, 24).into(),
                (5, 24 + type_name.len() + 1).into(),
                "string-0",
            ))
            .add_note(
                "file encoding was set to Slice1 here:",
                Some(&Span::new((2, 17).into(), (2, 34).into(), "string-0")),
            );
            check_diagnostics(diagnostics, [expected]);
        }

        #[test_case("class Foo {}"; "class")]
        #[test_case("interface Foo {}"; "interface")]
        #[test_case("custom Foo"; "custom type")]
        fn optional_user_defined_types_are_allowed(definition: &str) {
            // Arrange
            let slice = format!(
                "
                encoding = Slice1
                module Test
                {definition}
                exception E {{
                    a: Foo?
                }}
                "
            );

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let field = ast.find_element::<Field>("Test::E::a").unwrap();
            assert!(field.data_type.is_optional);
        }

        #[test_case("compact struct Foo {}"; "r#struct")]
        #[test_case("unchecked enum Foo {}"; "r#enum")]
        fn optional_user_defined_types_are_disallowed(definition: &str) {
            // Arrange
            let slice = format!(
                "
                encoding = Slice1
                module Test
                {definition}
                exception E {{
                    a: Foo?
                }}
                "
            );

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Error::OptionalsNotSupported {
                encoding: Encoding::Slice1,
            })
            .set_span(&Span::new((6, 24).into(), (6, 28).into(), "string-0"))
            .add_note(
                "file encoding was set to Slice1 here:",
                Some(&Span::new((2, 17).into(), (2, 34).into(), "string-0")),
            );

            check_diagnostics(diagnostics, [expected]);
        }

        #[test]
        fn sequences_of_optionals_are_disallowed() {
            // Arrange
            let slice = "
                encoding = Slice1
                module Test
                exception E {
                    a: sequence<bool?>
                }
            ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Error::OptionalsNotSupported {
                encoding: Encoding::Slice1,
            })
            .set_span(&Span::new((5, 33).into(), (5, 38).into(), "string-0"))
            .add_note(
                "file encoding was set to Slice1 here:",
                Some(&Span::new((2, 17).into(), (2, 34).into(), "string-0")),
            );

            check_diagnostics(diagnostics, [expected]);
        }

        // Using optional dictionary keys is always an error, but that check isn't until the validation phase.
        // This tests that we emit an `OptionalsNotSupported` error in the encoding compatibility phase before that.
        // Ensuring that the encoding compatibility checks correctly validate dictionary keys.
        #[test]
        fn dictionaries_with_optional_keys_are_disallowed() {
            // Arrange
            let slice = "
                encoding = Slice1
                module Test
                exception E {
                    a: dictionary<uint8?, float32>
                }
            ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Error::OptionalsNotSupported {
                encoding: Encoding::Slice1,
            })
            .set_span(&Span::new((5, 35).into(), (5, 41).into(), "string-0"))
            .add_note(
                "file encoding was set to Slice1 here:",
                Some(&Span::new((2, 17).into(), (2, 34).into(), "string-0")),
            );

            check_diagnostics(diagnostics, [expected]);
        }

        #[test]
        fn dictionaries_with_optional_values_are_disallowed() {
            // Arrange
            let slice = "
                encoding = Slice1
                module Test
                exception E {
                    a: dictionary<string, int32?>
                }
            ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Error::OptionalsNotSupported {
                encoding: Encoding::Slice1,
            })
            .set_span(&Span::new((5, 43).into(), (5, 49).into(), "string-0"))
            .add_note(
                "file encoding was set to Slice1 here:",
                Some(&Span::new((2, 17).into(), (2, 34).into(), "string-0")),
            );

            check_diagnostics(diagnostics, [expected]);
        }

        #[test]
        fn untagged_optional_parameters_are_disallowed() {
            // Arrange
            let slice = "
                encoding = Slice1
                module Test
                interface I {
                    op(a: bool?)
                }
            ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Error::OptionalsNotSupported {
                encoding: Encoding::Slice1,
            })
            .set_span(&Span::new((5, 27).into(), (5, 32).into(), "string-0"))
            .add_note(
                "file encoding was set to Slice1 here:",
                Some(&Span::new((2, 17).into(), (2, 34).into(), "string-0")),
            );

            check_diagnostics(diagnostics, [expected]);
        }

        // This test ensures that optional types are allowed in a Slice1 context if the type is tagged.
        // Note that duplicate tests in `tag_tests` are testing that tagged types are required to be optional,
        // a different behavior.
        #[test]
        fn tagged_optional_parameters_are_allowed() {
            // Arrange
            let slice = "
                encoding = Slice1
                module Test
                interface I {
                    op(tag(1) a: float32?)
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let parameter = ast.find_element::<Parameter>("Test::I::op::a").unwrap();
            assert!(parameter.is_tagged());
            assert!(parameter.data_type().is_optional);
        }

        // Return tuples are handled just like parameter lists.
        // So only testing parameters and single return types is sufficient.
        #[test]
        fn optional_return_types_are_disallowed() {
            // Arrange
            let slice = "
                encoding = Slice1
                module Test
                interface I {
                    op() -> float64?
                }
            ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Error::OptionalsNotSupported {
                encoding: Encoding::Slice1,
            })
            .set_span(&Span::new((5, 29).into(), (5, 37).into(), "string-0"))
            .add_note(
                "file encoding was set to Slice1 here:",
                Some(&Span::new((2, 17).into(), (2, 34).into(), "string-0")),
            );

            check_diagnostics(diagnostics, [expected]);
        }

        #[test]
        fn untagged_optional_fields_are_disallowed() {
            // Arrange
            let slice = "
                encoding = Slice1
                module Test
                compact struct S {
                    a: bool?
                }
            ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Error::OptionalsNotSupported {
                encoding: Encoding::Slice1,
            })
            .set_span(&Span::new((5, 24).into(), (5, 29).into(), "string-0"))
            .add_note(
                "file encoding was set to Slice1 here:",
                Some(&Span::new((2, 17).into(), (2, 34).into(), "string-0")),
            );

            check_diagnostics(diagnostics, [expected]);
        }

        // This test ensures that optional types are allowed in a Slice1 context if the type is tagged.
        // Note that duplicate tests in `tag_tests` are testing that tagged types are required to be optional,
        // a different behavior.
        #[test]
        fn tagged_optional_fields_are_allowed() {
            // Arrange
            let slice = "
                encoding = Slice1
                module Test
                exception E {
                    tag(1) a: float32?
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let member = ast.find_element::<Field>("Test::E::a").unwrap();
            assert!(member.is_tagged());
            assert!(member.data_type().is_optional);
        }
    }

    mod slice2 {
        use super::*;
        use test_case::test_case;

        #[test_case("bool")]
        #[test_case("int8")]
        #[test_case("uint8")]
        #[test_case("int16")]
        #[test_case("uint16")]
        #[test_case("int32")]
        #[test_case("uint32")]
        #[test_case("varint32")]
        #[test_case("varuint32")]
        #[test_case("int64")]
        #[test_case("uint64")]
        #[test_case("varint62")]
        #[test_case("varuint62")]
        #[test_case("float32")]
        #[test_case("float64")]
        #[test_case("string")]
        fn optional_builtin_types_are_allowed(type_name: &str) {
            // Arrange
            let slice = format!(
                "
                module Test
                exception E {{
                    a: {type_name}?
                }}
                "
            );

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let field = ast.find_element::<Field>("Test::E::a").unwrap();
            assert!(field.data_type.is_optional);
        }

        #[test_case("struct Foo {}"; "r#struct")]
        #[test_case("exception Foo {}"; "class")]
        #[test_case("unchecked enum Foo: uint8 {}"; "r#enum")]
        #[test_case("interface Foo {}"; "interface")]
        #[test_case("custom Foo"; "custom type")]
        fn optional_user_defined_types_are_allowed(definition: &str) {
            // Arrange
            let slice = format!(
                "
                module Test
                {definition}
                exception E {{
                    a: Foo?
                }}
                "
            );

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let field = ast.find_element::<Field>("Test::E::a").unwrap();
            assert!(field.data_type.is_optional);
        }

        #[test]
        fn optional_sequences_are_parsed_correctly() {
            // Arrange
            let slice = "
                module Test
                exception E {
                    a: sequence<int32>?
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let field = ast.find_element::<Field>("Test::E::a").unwrap();
            assert!(field.data_type.is_optional);

            let Types::Sequence(sequence) = field.data_type().concrete_type() else { panic!() };
            assert!(!sequence.element_type.is_optional);
        }

        #[test]
        fn sequences_with_optional_elements_are_parsed_correctly() {
            // Arrange
            let slice = "
                module Test
                exception E {
                    a: sequence<bool?>
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let field = ast.find_element::<Field>("Test::E::a").unwrap();
            assert!(!field.data_type.is_optional);

            let Types::Sequence(sequence) = field.data_type().concrete_type() else { panic!() };
            assert!(sequence.element_type.is_optional);
        }

        #[test]
        fn optional_dictionaries_are_parsed_correctly() {
            // Arrange
            let slice = "
                module Test
                exception E {
                    a: dictionary<varuint62, string>?
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let field = ast.find_element::<Field>("Test::E::a").unwrap();
            assert!(field.data_type.is_optional);

            let Types::Dictionary(dictionary) = field.data_type().concrete_type() else { panic!() };
            assert!(!dictionary.key_type.is_optional);
            assert!(!dictionary.value_type.is_optional);
        }

        // Using optional dictionary keys is an error, but this test ensures that they are parsed correctly regardless.
        #[test]
        fn dictionaries_with_optional_keys_are_parsed_correctly() {
            // Arrange
            let slice = "
                module Test
                exception E {
                    a: dictionary<varuint62?, string>
                }
            ";

            // Act
            let ast = slice::compile_from_strings(&[slice], None).ast; // Use `compile_from_strings` to ignore errors.

            // Assert
            let field = ast.find_element::<Field>("Test::E::a").unwrap();
            assert!(!field.data_type.is_optional);

            let Types::Dictionary(dictionary) = field.data_type().concrete_type() else { panic!() };
            assert!(dictionary.key_type.is_optional);
            assert!(!dictionary.value_type.is_optional);
        }

        #[test]
        fn dictionaries_with_optional_values_are_parsed_correctly() {
            // Arrange
            let slice = "
                module Test
                exception E {
                    a: dictionary<varuint62, string?>
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let field = ast.find_element::<Field>("Test::E::a").unwrap();
            assert!(!field.data_type.is_optional);

            let Types::Dictionary(dictionary) = field.data_type().concrete_type() else { panic!() };
            assert!(!dictionary.key_type.is_optional);
            assert!(dictionary.value_type.is_optional);
        }

        #[test]
        fn operations_can_use_a_mix_of_optional_and_required_parameters() {
            // Arrange
            let slice = "
                module Test
                interface I {
                    op(a: bool?, b: string, c: stream int32?) -> (x: float32, y: uint8?, z: stream int16)
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let parameter_a = ast.find_element::<Parameter>("Test::I::op::a").unwrap();
            assert!(parameter_a.data_type().is_optional);

            let parameter_b = ast.find_element::<Parameter>("Test::I::op::b").unwrap();
            assert!(!parameter_b.data_type().is_optional);

            let parameter_c = ast.find_element::<Parameter>("Test::I::op::c").unwrap();
            assert!(parameter_c.data_type().is_optional);

            let return_x = ast.find_element::<Parameter>("Test::I::op::x").unwrap();
            assert!(!return_x.data_type().is_optional);

            let return_y = ast.find_element::<Parameter>("Test::I::op::y").unwrap();
            assert!(return_y.data_type().is_optional);

            let return_z = ast.find_element::<Parameter>("Test::I::op::z").unwrap();
            assert!(!return_z.data_type().is_optional);
        }

        #[test]
        fn operations_can_return_single_optional_types() {
            // Arrange
            let slice = "
                module Test
                interface I {
                    op() -> float64?
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let return_value = ast.find_element::<Parameter>("Test::I::op::returnValue").unwrap();
            assert!(return_value.data_type().is_optional);
        }

        #[test]
        fn structs_can_use_a_mix_of_optional_and_required_fields() {
            // Arrange
            let slice = "
                module Test
                struct S {
                    a: bool?
                    b: string
                    c: int32?
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let member_a = ast.find_element::<Field>("Test::S::a").unwrap();
            assert!(member_a.data_type().is_optional);

            let member_b = ast.find_element::<Field>("Test::S::b").unwrap();
            assert!(!member_b.data_type().is_optional);

            let member_c = ast.find_element::<Field>("Test::S::c").unwrap();
            assert!(member_c.data_type().is_optional);
        }
    }
}
