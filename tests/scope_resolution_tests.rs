// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod scope_resolution {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::{parse_for_ast, parse_for_diagnostics};
    use slice::diagnostics::{Error, ErrorKind};
    use slice::grammar::*;

    #[test]
    fn file_scoped_modules_can_not_contain_sub_modules() {
        // Arrange
        let slice = "
            module T;
            module S
            {
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::FileScopedModuleCannotContainSubModules {
            identifier: "T".to_owned(),
        })
        .add_note("file level module 'T' declared here", None);
        assert_errors!(diagnostics, [&expected]);
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
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::Redefinition {
            identifier: "B".to_string(),
        })
        .add_note("'B' was previously defined here", None);
        assert_errors!(diagnostics, [&expected]);
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

                interface C
                {
                }
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::TypeMismatch {
            expected: "Type".to_string(),
            actual: "module".to_string(),
        });
        assert_errors!(diagnostics, [&expected]);
    }

    #[test]
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
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::DoesNotExist {
            identifier: "Nested::C".to_string(),
        });
        assert_errors!(diagnostics, [&expected]);
    }
}
