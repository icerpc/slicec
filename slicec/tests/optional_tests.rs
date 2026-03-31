// Copyright (c) ZeroC, Inc.

mod test_helpers;

mod optional {
    use crate::test_helpers::*;
    use slicec::grammar::*;
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
            struct S {{
                a: {type_name}?
            }}
            "
        );

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let field = ast.find_element::<Field>("Test::S::a").unwrap();
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
            struct S {{
                a: {type_name}?
            }}
            "
        );

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let field = ast.find_element::<Field>("Test::S::a").unwrap();
        assert_eq!(field.data_type.type_string(), type_name.to_owned() + "?");
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
    fn optional_builtin_types_are_allowed(type_name: &str) {
        // Arrange
        let slice = format!(
            "
                module Test
                struct S {{
                    a: {type_name}?
                }}
                "
        );

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let field = ast.find_element::<Field>("Test::S::a").unwrap();
        assert!(field.data_type.is_optional);
    }

    #[test_case("struct Foo {}"; "r#struct")]
    #[test_case("unchecked enum Foo: uint8 {}"; "r#enum")]
    #[test_case("custom Foo"; "custom type")]
    fn optional_user_defined_types_are_allowed(definition: &str) {
        // Arrange
        let slice = format!(
            "
                module Test
                {definition}
                struct S {{
                    a: Foo?
                }}
                "
        );

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let field = ast.find_element::<Field>("Test::S::a").unwrap();
        assert!(field.data_type.is_optional);
    }

    #[test]
    fn optional_results_are_parsed_correctly() {
        // Arrange
        let slice = "
                module Test
                struct S {
                    a: Result<varuint62, string>?
                }
            ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let field = ast.find_element::<Field>("Test::S::a").unwrap();
        assert!(field.data_type.is_optional);

        let Types::ResultType(result_type) = field.data_type().concrete_type() else { panic!() };
        assert!(!result_type.success_type.is_optional);
        assert!(!result_type.failure_type.is_optional);
    }

    #[test]
    fn results_with_optional_success_types_are_parsed_correctly() {
        // Arrange
        let slice = "
                module Test
                struct S {
                    a: Result<varuint62?, string>
                }
            ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let field = ast.find_element::<Field>("Test::S::a").unwrap();
        assert!(!field.data_type.is_optional);

        let Types::ResultType(result_type) = field.data_type().concrete_type() else { panic!() };
        assert!(result_type.success_type.is_optional);
        assert!(!result_type.failure_type.is_optional);
    }

    #[test]
    fn results_with_optional_failure_types_are_parsed_correctly() {
        // Arrange
        let slice = "
                module Test
                struct S {
                    a: Result<varuint62, string?>
                }
            ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let field = ast.find_element::<Field>("Test::S::a").unwrap();
        assert!(!field.data_type.is_optional);

        let Types::ResultType(result_type) = field.data_type().concrete_type() else { panic!() };
        assert!(!result_type.success_type.is_optional);
        assert!(result_type.failure_type.is_optional);
    }

    #[test]
    fn optional_sequences_are_parsed_correctly() {
        // Arrange
        let slice = "
                module Test
                struct S {
                    a: Sequence<int32>?
                }
            ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let field = ast.find_element::<Field>("Test::S::a").unwrap();
        assert!(field.data_type.is_optional);

        let Types::Sequence(sequence) = field.data_type().concrete_type() else { panic!() };
        assert!(!sequence.element_type.is_optional);
    }

    #[test]
    fn sequences_with_optional_elements_are_parsed_correctly() {
        // Arrange
        let slice = "
                module Test
                struct S {
                    a: Sequence<bool?>
                }
            ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let field = ast.find_element::<Field>("Test::S::a").unwrap();
        assert!(!field.data_type.is_optional);

        let Types::Sequence(sequence) = field.data_type().concrete_type() else { panic!() };
        assert!(sequence.element_type.is_optional);
    }

    #[test]
    fn optional_dictionaries_are_parsed_correctly() {
        // Arrange
        let slice = "
                module Test
                struct S {
                    a: Dictionary<varuint62, string>?
                }
            ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let field = ast.find_element::<Field>("Test::S::a").unwrap();
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
                struct S {
                    a: Dictionary<varuint62?, string>
                }
            ";

        // Act
        let ast = parse(slice, None).ast; // use `parse` to ignore errors.

        // Assert
        let field = ast.find_element::<Field>("Test::S::a").unwrap();
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
                struct S {
                    a: Dictionary<varuint62, string?>
                }
            ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let field = ast.find_element::<Field>("Test::S::a").unwrap();
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
