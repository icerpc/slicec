// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod optional {
    use crate::helpers::parsing_helpers::{check_diagnostics, parse_for_ast, parse_for_diagnostics};
    use slice::diagnostics::{Error, ErrorKind};
    use slice::grammar::*;
    use slice::slice_file::Span;

    mod slice1 {
        use super::*;
        use test_case::test_case;

        #[test_case("ServiceAddress")]
        #[test_case("AnyClass")]
        fn optional_builtin_types_are_allowed(type_name: &str) {
            // Arrange
            let slice = format!("
                encoding = 1;
                module Test;
                typealias F = {type_name}?;
            ");

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let type_alias = &ast.find_element::<TypeAlias>("Test::F").unwrap();
            assert!(type_alias.underlying.is_optional);
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
        #[test_case("Foo"; "simple")]
        #[test_case("Test::Foo"; "relatively scoped")]
        #[test_case("::Test::Foo"; "globally scoped")]
        #[ignore]
        fn optional_builtin_types_are_disallowed(type_name: &str) {
            // Arrange
            let slice = format!("
                encoding = 1;
                module Test;
                typealias Foo = bool;
                typealias F = {type_name}?;
            ");

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Error::new(ErrorKind::OptionalsNotSupported { encoding: Encoding::Slice1 })
                .set_span(&Span::new((5, 31).into(), (5, 31 + type_name.len() + 1).into(), "string-0"))
                .add_note("file encoding was set to Slice1 here:", Some(
                    &Span::new((2, 17).into(), (2, 29).into(), "string-0")
                ));

            check_diagnostics(diagnostics, [expected]);
        }

        #[test_case("class Foo {}"; "class")]
        #[test_case("interface Foo {}"; "interface")]
        fn optional_user_defined_types_are_allowed(definition: &str) {
            // Arrange
            let slice = format!("
                encoding = 1;
                module Test;
                {definition}
                typealias F = Foo?;
            ");

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let type_alias = &ast.find_element::<TypeAlias>("Test::F").unwrap();
            assert!(type_alias.underlying.is_optional);
        }

        #[test_case("struct Foo {}"; "r#struct")]
        #[test_case("exception Foo {}"; "exception")]
        #[test_case("custom Foo;"; "custom type")]
        #[ignore]
        fn optional_user_defined_types_are_disallowed(definition: &str) {
            // Arrange
            let slice = format!("
                encoding = 1;
                module Test;
                {definition}
                typealias F = Foo?;
            ");

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Error::new(ErrorKind::OptionalsNotSupported { encoding: Encoding::Slice1 })
                .set_span(&Span::new((5, 31).into(), (5, 35).into(), "string-0"))
                .add_note("file encoding was set to Slice1 here:", Some(
                    &Span::new((2, 17).into(), (2, 29).into(), "string-0")
                ));

            check_diagnostics(diagnostics, [expected]);
        }

        #[test]
        #[ignore]
        fn sequences_of_optionals_are_disallowed() {
            // Arrange
            let slice = "
                encoding = 1;
                module Test;
                typealias S = sequence<bool?>;
            ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Error::new(ErrorKind::OptionalsNotSupported { encoding: Encoding::Slice1 })
                .set_span(&Span::new((4, 40).into(), (4, 45).into(), "string-0"))
                .add_note("file encoding was set to Slice1 here:", Some(
                    &Span::new((2, 17).into(), (2, 29).into(), "string-0")
                ));

            check_diagnostics(diagnostics, [expected]);
        }

        // This test looks like a pointless duplicate of the identically named function in the Slice2 tests.
        // But, this test ensures the compiler correctly reports a `KeyMustBeNonOptional` error for an optional key,
        // and not a generic `OptionalsNotSupported` error like for other optionals used in an `encoding = 1` context.
        #[test]
        fn dictionaries_with_optional_keys_are_disallowed() {
            // Arrange
            let slice = "
                encoding = 1;
                module Test;
                typealias D = dictionary<uint8?, float32>;
            ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Error::new(ErrorKind::KeyMustBeNonOptional)
                .set_span(&Span::new((4, 42).into(), (4, 48).into(), "string-0"));

            check_diagnostics(diagnostics, [expected]);
        }

        #[test]
        #[ignore]
        fn dictionaries_with_optional_values_are_disallowed() {
            // Arrange
            let slice = "
                encoding = 1;
                module Test;
                typealias D = dictionary<string, int32?>;
            ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Error::new(ErrorKind::OptionalsNotSupported { encoding: Encoding::Slice1 })
                .set_span(&Span::new((4, 50).into(), (4, 59).into(), "string-0"))
                .add_note("file encoding was set to Slice1 here:", Some(
                    &Span::new((2, 17).into(), (2, 29).into(), "string-0")
                ));

            check_diagnostics(diagnostics, [expected]);
        }

        #[test]
        fn optional_parameters_are_disallowed() {
            // Arrange
            let slice = "
                encoding = 1;
                module Test;
                interface I
                {
                    op(a: bool?);
                }
            ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Error::new(ErrorKind::OptionalsNotSupported { encoding: Encoding::Slice1 })
                .set_span(&Span::new((6, 27).into(), (6, 32).into(), "string-0"))
                .add_note("file encoding was set to Slice1 here:", Some(
                    &Span::new((2, 17).into(), (2, 29).into(), "string-0")
                ));

            check_diagnostics(diagnostics, [expected]);
        }

        // This test looks like a pointless duplicate of an identically named function in the Slice2 tests.
        // But, this test is ensuring that optional types are allowed in an `encoding = 1` context if the
        // type is tagged. Non-tagged optionals are disallowed and cause an `OptionalsNotSupported` error.
        #[test]
        fn tagged_parameters_can_be_optional() {
            // Arrange
            let slice = "
                encoding = 1;
                module Test;
                interface I
                {
                    op(a: tag(1) float32?);
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let parameter = ast.find_element::<Parameter>("Test::I::op::a").unwrap();
            assert!(parameter.is_tagged());
            assert!(parameter.data_type().is_optional);
        }

        #[test]
        fn optional_stream_parameters_are_disallowed() {
            // Arrange
            let slice = "
                encoding = 1;
                module Test;
                interface I
                {
                    op(a: stream string?);
                }
            ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = [
                Error::new(ErrorKind::OptionalsNotSupported { encoding: Encoding::Slice1 })
                    .set_span(&Span::new((6, 34).into(), (6, 41).into(), "string-0"))
                    .add_note("file encoding was set to Slice1 here:", Some(
                        &Span::new((2, 17).into(), (2, 29).into(), "string-0")
                    )),
                Error::new(ErrorKind::StreamedParametersNotSupported { encoding: Encoding::Slice1 }),
            ];

            check_diagnostics(diagnostics, expected);
        }

        #[test]
        fn optional_return_types_are_disallowed() {
            // Arrange
            let slice = "
                encoding = 1;
                module Test;
                interface I
                {
                    op() -> float64?;
                }
            ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Error::new(ErrorKind::OptionalsNotSupported { encoding: Encoding::Slice1 })
                .set_span(&Span::new((6, 29).into(), (6, 37).into(), "string-0"))
                .add_note("file encoding was set to Slice1 here:", Some(
                    &Span::new((2, 17).into(), (2, 29).into(), "string-0")
                ));

            check_diagnostics(diagnostics, [expected]);
        }

        // Return tuples are handled just like parameter lists. So just testing parameter lists is sufficient.

        #[test]
        fn optional_data_members_are_disallowed() {
            // Arrange
            let slice = "
                encoding = 1;
                module Test;
                compact struct S
                {
                    a: bool?,
                }
            ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Error::new(ErrorKind::OptionalsNotSupported { encoding: Encoding::Slice1 })
                .set_span(&Span::new((6, 24).into(), (6, 29).into(), "string-0"))
                .add_note("file encoding was set to Slice1 here:", Some(
                    &Span::new((2, 17).into(), (2, 29).into(), "string-0")
                ));

            check_diagnostics(diagnostics, [expected]);
        }

        // This test looks like a pointless duplicate of an identically named function in the Slice2 tests.
        // But, this test is ensuring that optional types are allowed in an `encoding = 1` context if the
        // type is tagged. Non-tagged optionals are disallowed and cause an `OptionalsNotSupported` error.
        #[test]
        fn tagged_data_members_can_be_optional() {
            // Arrange
            let slice = "
                encoding = 1;
                module Test;
                exception E
                {
                    a: tag(1) float32?,
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let member = ast.find_element::<DataMember>("Test::E::a").unwrap();
            assert!(member.is_tagged());
            assert!(member.data_type().is_optional);
        }
    }

    mod slice2 {
        use super::*;
        use test_case::test_case;

        // For `encoding = 2` files, all the primitives are parsed & validated the same way, so testing one is enough.
        #[test]
        fn optional_primitives_are_parsed_correctly() {
            // Arrange
            let slice = format!("
                module Test;
                typealias P = bool?;
            ");

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let type_alias = ast.find_element::<TypeAlias>("Test::P").unwrap();
            assert!(type_alias.underlying.is_optional);
        }

        #[test_case("Foo"; "simple")]
        #[test_case("Test::Foo"; "relatively scoped")]
        #[test_case("::Test::Foo"; "globally scoped")]
        fn optional_user_defined_types_are_parsed_correctly(type_name: &str) {
            // Arrange
            let slice = format!("
                module Test;
                typealias Foo = bool;
                typealias F = {type_name}?;
            ");

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let type_alias = ast.find_element::<TypeAlias>("Test::F").unwrap();
            assert!(type_alias.underlying.is_optional);
        }

        #[test_case("struct Foo {}"; "r#struct")]
        #[test_case("exception Foo {}"; "exception")]
        #[test_case("interface Foo {}"; "interface")]
        #[test_case("custom Foo;"; "custom type")]
        fn optional_user_defined_types_are_allowed(definition: &str) {
            // Arrange
            let slice = format!("
                module Test;
                {definition}
                typealias F = Foo?;
            ");

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let type_alias = ast.find_element::<TypeAlias>("Test::F").unwrap();
            assert!(type_alias.underlying.is_optional);
        }

        #[test]
        fn optional_sequences_are_parsed_correctly() {
            // Arrange
            let slice = "
                module Test;
                typealias S = sequence<int32>?;
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let type_alias = ast.find_element::<TypeAlias>("Test::S").unwrap();
            assert!(type_alias.underlying.is_optional);

            let Types::Sequence(sequence) = type_alias.underlying.definition().concrete_type() else { panic!() };
            assert!(!sequence.element_type.is_optional);
        }

        #[test]
        fn sequences_of_optionals_are_parsed_correctly() {
            // Arrange
            let slice = "
                module Test;
                typealias S = sequence<bool?>;
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let type_alias = ast.find_element::<TypeAlias>("Test::S").unwrap();
            assert!(!type_alias.underlying.is_optional);

            let Types::Sequence(sequence) = type_alias.underlying.definition().concrete_type() else { panic!() };
            assert!(sequence.element_type.is_optional);
        }

        #[test]
        fn optional_dictionaries_are_parsed_correctly() {
            // Arrange
            let slice = "
                module Test;
                typealias D = dictionary<varuint62, string>?;
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let type_alias = ast.find_element::<TypeAlias>("Test::D").unwrap();
            assert!(type_alias.underlying.is_optional);

            let Types::Dictionary(dictionary) = type_alias.underlying.definition().concrete_type() else { panic!() };
            assert!(!dictionary.key_type.is_optional);
            assert!(!dictionary.value_type.is_optional);
        }

        #[test]
        fn dictionaries_with_optional_keys_are_disallowed() {
            // Arrange
            let slice = "
                module Test;
                typealias D = dictionary<uint8?, float32>;
            ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Error::new(ErrorKind::KeyMustBeNonOptional)
                .set_span(&Span::new((3, 42).into(), (3, 48).into(), "string-0"));

            check_diagnostics(diagnostics, [expected]);
        }

        #[test]
        fn dictionaries_with_optional_values_are_parsed_correctly() {
            // Arrange
            let slice = "
                module Test;
                typealias D = dictionary<string, varint32?>;
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let type_alias = ast.find_element::<TypeAlias>("Test::D").unwrap();
            assert!(!type_alias.underlying.is_optional);

            let Types::Dictionary(dictionary) = type_alias.underlying.definition().concrete_type() else { panic!() };
            assert!(!dictionary.key_type.is_optional);
            assert!(dictionary.value_type.is_optional);
        }

        #[test]
        fn operations_can_take_optional_parameters() {
            // Arrange
            let slice = "
                module Test;
                interface I
                {
                    op(a: bool?);
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let parameter = ast.find_element::<Parameter>("Test::I::op::a").unwrap();
            assert!(parameter.data_type().is_optional);
        }

        #[test]
        fn tagged_parameters_can_be_optional() {
            // Arrange
            let slice = "
                module Test;
                interface I
                {
                    op(a: tag(1) float32?);
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let parameter = ast.find_element::<Parameter>("Test::I::op::a").unwrap();
            assert!(parameter.is_tagged());
            assert!(parameter.data_type().is_optional);
        }

        #[test]
        fn tagged_parameters_must_be_optional() {
            // Arrange
            let slice = "
                module Test;
                interface I
                {
                    op(a: tag(1) float32);
                }
            ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Error::new(ErrorKind::TaggedMemberMustBeOptional { member_identifier: "a".to_owned() })
                .set_span(&Span::new((5, 24).into(), (5, 41).into(), "string-0"));

            check_diagnostics(diagnostics, [expected]);
        }

        #[test]
        fn streamed_parameters_can_be_optional() {
            // Arrange
            let slice = "
                module Test;
                interface I
                {
                    op(a: stream string?);
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let parameter = ast.find_element::<Parameter>("Test::I::op::a").unwrap();
            assert!(parameter.is_streamed);
            assert!(parameter.data_type().is_optional);
        }

        #[test]
        fn operations_can_take_a_mix_of_optional_and_required_parameters() {
            // Arrange
            let slice = "
                module Test;
                interface I
                {
                    op(a: bool?, b: string, c: stream int32?) -> (x: float32, y: uint8?, z: stream int16);
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let parameter_a = ast.find_element::<Parameter>("Test::I::op::a").unwrap();
            assert!(!parameter_a.is_streamed);
            assert!(parameter_a.data_type().is_optional);

            let parameter_b = ast.find_element::<Parameter>("Test::I::op::b").unwrap();
            assert!(!parameter_b.is_streamed);
            assert!(!parameter_b.data_type().is_optional);

            let parameter_c = ast.find_element::<Parameter>("Test::I::op::c").unwrap();
            assert!(parameter_c.is_streamed);
            assert!(parameter_c.data_type().is_optional);

            let return_x = ast.find_element::<Parameter>("Test::I::op::x").unwrap();
            assert!(!return_x.is_streamed);
            assert!(!return_x.data_type().is_optional);

            let return_y = ast.find_element::<Parameter>("Test::I::op::y").unwrap();
            assert!(!return_y.is_streamed);
            assert!(return_y.data_type().is_optional);

            let return_z = ast.find_element::<Parameter>("Test::I::op::z").unwrap();
            assert!(return_z.is_streamed);
            assert!(!return_z.data_type().is_optional);
        }

        // Return tuples are parsed identically to parameter lists. Since we already have tests for parameter lists,
        // it's sufficient to only test single return types.
        #[test]
        fn operations_can_return_single_optional_types() {
            // Arrange
            let slice = "
                module Test;
                interface I
                {
                    op() -> float64?;
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let return_value = ast.find_element::<Parameter>("Test::I::op::returnValue").unwrap();
            assert!(return_value.data_type().is_optional);
        }

        #[test]
        fn structs_can_have_optional_data_members() {
            // Arrange
            let slice = "
                module Test;
                struct S
                {
                    a: bool?,
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let member = ast.find_element::<DataMember>("Test::S::a").unwrap();
            assert!(member.data_type().is_optional);
        }

        #[test]
        fn tagged_data_members_can_be_optional() {
            // Arrange
            let slice = "
                module Test;
                exception E
                {
                    a: tag(1) float32?,
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let member = ast.find_element::<DataMember>("Test::E::a").unwrap();
            assert!(member.is_tagged());
            assert!(member.data_type().is_optional);
        }

        #[test]
        fn tagged_data_members_must_be_optional() {
            // Arrange
            let slice = "
                module Test;
                exception E
                {
                    a: tag(1) float32,
                }
            ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Error::new(ErrorKind::TaggedMemberMustBeOptional { member_identifier: "a".to_owned() })
                .set_span(&Span::new((5, 21).into(), (5, 38).into(), "string-0"));

            check_diagnostics(diagnostics, [expected]);
        }

        #[test]
        fn structs_can_have_a_mix_of_optional_and_required_data_members() {
            // Arrange
            let slice = "
                module Test;
                struct S
                {
                    a: bool?,
                    b: string,
                    c: int32?,
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let member_a = ast.find_element::<DataMember>("Test::S::a").unwrap();
            assert!(member_a.data_type().is_optional);

            let member_b = ast.find_element::<DataMember>("Test::S::b").unwrap();
            assert!(!member_b.data_type().is_optional);

            let member_c = ast.find_element::<DataMember>("Test::S::c").unwrap();
            assert!(member_c.data_type().is_optional);
        }

        #[test_case("bool"; "primitive")]
        #[test_case("Foo"; "user defined")]
        fn optional_type_names_end_with_a_question_mark(type_name: &str) {
            // Arrange
            let slice = format!("
                module Test;
                struct Foo {{}}
                typealias T = {type_name}?;
            ");

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let type_alias = ast.find_element::<TypeAlias>("Test::T").unwrap();
            assert_eq!(type_alias.underlying.type_string(), type_name.to_owned() + "?");
        }
    }
}
