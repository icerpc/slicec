// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod scope_resolution {

    use crate::helpers::parsing_helpers::{parse_for_ast, parse_for_diagnostics};
    use crate::{assert_errors, assert_errors_new};
    use slice::diagnostics::{Diagnostic, DiagnosticKind, LogicErrorKind, Note};
    use slice::grammar::*;

    #[test]
    fn file_level_modules_can_not_contain_sub_modules() {
        // Arrange
        let slice = "
            module T;
            module S {}
        ";

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new_with_notes(
            DiagnosticKind::SyntaxError("file level modules cannot contain sub-modules".to_owned()),
            None,
            vec![Note::new("file level module 'T' declared here", None)],
        );
        assert_errors_new!(diagnostic_reporter, [&expected]);
    }

    #[test]
    fn identifier_exists_in_module_and_submodule() {
        // Arrange
        let slice = "
            module A
            {
                typealias S = int32;

                module B
                {
                    struct S
                    {
                        v: string,
                    }
                }

                struct C
                {
                    s1: S,
                    s2: A::S,
                    s3: B::S,
                    s4: A::B::S,
                }
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let s1_type = ast.find_element::<DataMember>("A::C::s1").unwrap().data_type();
        let s2_type = ast.find_element::<DataMember>("A::C::s2").unwrap().data_type();
        let s3_type = ast.find_element::<DataMember>("A::C::s3").unwrap().data_type();
        let s4_type = ast.find_element::<DataMember>("A::C::s4").unwrap().data_type();

        assert!(matches!(s1_type.concrete_type(), Types::Primitive(Primitive::Int32)));
        assert!(matches!(s2_type.concrete_type(), Types::Primitive(Primitive::Int32)));
        assert!(matches!(s3_type.concrete_type(), Types::Struct(_)));
        assert!(matches!(s4_type.concrete_type(), Types::Struct(_)));
    }

    #[test]
    fn identifier_exists_in_module_and_parent_module() {
        // Arrange
        let slice = "
            module A
            {
                typealias S = int32;

                module B
                {
                    typealias S = string;

                    struct C
                    {
                        s1: S,
                        s2: B::S,
                        s3: A::B::S,
                        s4: A::S,
                    }
                }
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let s1_type = ast.find_element::<DataMember>("A::B::C::s1").unwrap().data_type();
        let s2_type = ast.find_element::<DataMember>("A::B::C::s2").unwrap().data_type();
        let s3_type = ast.find_element::<DataMember>("A::B::C::s3").unwrap().data_type();
        let s4_type = ast.find_element::<DataMember>("A::B::C::s4").unwrap().data_type();

        assert!(matches!(s1_type.concrete_type(), Types::Primitive(Primitive::String)));
        assert!(matches!(s2_type.concrete_type(), Types::Primitive(Primitive::String)));
        assert!(matches!(s3_type.concrete_type(), Types::Primitive(Primitive::String)));
        assert!(matches!(s4_type.concrete_type(), Types::Primitive(Primitive::Int32)));
    }

    #[test]
    fn identifier_exists_in_multiple_parent_modules() {
        // Arrange
        let slice = "
            module A
            {
                typealias S = int32;

                module B
                {

                    struct S
                    {
                        v: string,
                    }

                    module B
                    {

                        struct C
                        {
                            s1: S,
                            s2: B::S,
                            s3: A::S,
                        }
                    }
                }
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let s1_type = ast.find_element::<DataMember>("A::B::B::C::s1").unwrap().data_type();
        let s2_type = ast.find_element::<DataMember>("A::B::B::C::s2").unwrap().data_type();
        let s3_type = ast.find_element::<DataMember>("A::B::B::C::s3").unwrap().data_type();

        assert!(matches!(s1_type.concrete_type(), Types::Struct(_)));
        assert!(matches!(s2_type.concrete_type(), Types::Struct(_)));
        assert!(matches!(s3_type.concrete_type(), Types::Primitive(Primitive::Int32)));
    }

    #[test]
    fn identifier_exists_in_multiple_modules_with_common_partial_scope() {
        // Arrange
        let slice = "
            module A
            {
                module B
                {
                    typealias S = string;

                    module A
                    {
                        module B
                        {
                            typealias S = int32;

                            struct C
                            {
                                s1: A::B::S,
                                s2: ::A::B::S,
                            }
                        }
                    }
                }

                struct C
                {
                    s1: A::B::S,
                    s2: ::A::B::S,
                }
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let nested_s1_type = ast.find_element::<DataMember>("A::B::A::B::C::s1").unwrap().data_type();
        let nested_s2_type = ast.find_element::<DataMember>("A::B::A::B::C::s2").unwrap().data_type();
        let s1_type = ast.find_element::<DataMember>("A::C::s1").unwrap().data_type();
        let s2_type = ast.find_element::<DataMember>("A::C::s2").unwrap().data_type();

        assert!(matches!(
            nested_s1_type.concrete_type(),
            Types::Primitive(Primitive::Int32),
        ));
        assert!(matches!(
            nested_s2_type.concrete_type(),
            Types::Primitive(Primitive::String),
        ));
        assert!(matches!(s1_type.concrete_type(), Types::Primitive(Primitive::String)));
        assert!(matches!(s2_type.concrete_type(), Types::Primitive(Primitive::String)));
    }

    #[test]
    fn interface_has_same_identifier_as_module() {
        // Arrange
        let slice = "
            module A
            {
                module B
                {
                }

                interface B
                {
                }

                struct S
                {
                    b: B,
                }
            }
        ";

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        let expected =
            Diagnostic::new_with_notes(LogicErrorKind::Redefinition("B".to_string()), None, vec![Note::new(
                "`B` was previously defined here",
                None,
            )]);
        assert_errors_new!(diagnostic_reporter, [&expected]);
    }

    #[test]
    fn relative_scope_is_module_before_interface() {
        // Arrange
        let slice = "
            module A
            {
                module B
                {
                    module C
                    {
                        struct S
                        {
                            c: C,
                        }
                    }
                }

                interface C {}
            }
        ";

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        let expected: DiagnosticKind = LogicErrorKind::TypeMismatch("Type".to_string(), "module".to_string()).into();
        assert_errors_new!(diagnostic_reporter, [&expected]);
    }

    #[test]
    #[ignore = "reason: TODO Need to update AST Error emission"]
    fn missing_type_should_fail() {
        // Arrange
        let slice = "
            module A
            {
                struct C
                {
                    b: Nested::C,
                }
            }
        ";

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        assert_errors!(diagnostic_reporter, [
            "no element with identifier `Nested::C` exists in the scope `A`",
        ]);
    }
}
