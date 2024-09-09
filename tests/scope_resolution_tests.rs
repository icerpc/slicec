// Copyright (c) ZeroC, Inc.

mod test_helpers;

mod scope_resolution {

    use crate::test_helpers::*;
    use slicec::diagnostics::{Diagnostic, Error};
    use slicec::grammar::*;

    #[test]
    fn identifier_exists_in_module_and_submodule() {
        // Arrange
        let slice1 = "
            module A

            typealias S = int32
            struct C {
                s1: S
                s2: A::S
                s3: B::S
                s4: A::B::S
            }
        ";
        let slice2 = "
            module A::B

            struct S {
                v: string
            }
        ";

        // Act
        let ast = parse_multiple_for_ast(&[slice1, slice2]);

        // Assert
        let s1_type = ast.find_element::<Field>("A::C::s1").unwrap().data_type();
        let s2_type = ast.find_element::<Field>("A::C::s2").unwrap().data_type();
        let s3_type = ast.find_element::<Field>("A::C::s3").unwrap().data_type();
        let s4_type = ast.find_element::<Field>("A::C::s4").unwrap().data_type();

        assert!(matches!(s1_type.concrete_type(), Types::Primitive(Primitive::Int32)));
        assert!(matches!(s2_type.concrete_type(), Types::Primitive(Primitive::Int32)));
        assert!(matches!(s3_type.concrete_type(), Types::Struct(_)));
        assert!(matches!(s4_type.concrete_type(), Types::Struct(_)));
    }

    #[test]
    fn identifier_exists_in_module_and_parent_module() {
        // Arrange
        let slice1 = "
            module A

            typealias S = int32
        ";
        let slice2 = "
            module A::B

            typealias S = string

            struct C {
                s1: S
                s2: B::S
                s3: A::B::S
                s4: A::S
            }
        ";

        // Act
        let ast = parse_multiple_for_ast(&[slice1, slice2]);

        // Assert
        let s1_type = ast.find_element::<Field>("A::B::C::s1").unwrap().data_type();
        let s2_type = ast.find_element::<Field>("A::B::C::s2").unwrap().data_type();
        let s3_type = ast.find_element::<Field>("A::B::C::s3").unwrap().data_type();
        let s4_type = ast.find_element::<Field>("A::B::C::s4").unwrap().data_type();

        assert!(matches!(s1_type.concrete_type(), Types::Primitive(Primitive::String)));
        assert!(matches!(s2_type.concrete_type(), Types::Primitive(Primitive::String)));
        assert!(matches!(s3_type.concrete_type(), Types::Primitive(Primitive::String)));
        assert!(matches!(s4_type.concrete_type(), Types::Primitive(Primitive::Int32)));
    }

    #[test]
    fn identifier_exists_in_multiple_parent_modules() {
        // Arrange
        let slice1 = "
            module A
            typealias S = int32
        ";
        let slice2 = "
            module A::B
            struct S {
                v: string
            }
        ";
        let slice3 = "
            module A::B::B
            struct C {
                s1: S
                s2: B::S
                s3: A::S
            }
        ";

        // Act
        let ast = parse_multiple_for_ast(&[slice1, slice2, slice3]);

        // Assert
        let s1_type = ast.find_element::<Field>("A::B::B::C::s1").unwrap().data_type();
        let s2_type = ast.find_element::<Field>("A::B::B::C::s2").unwrap().data_type();
        let s3_type = ast.find_element::<Field>("A::B::B::C::s3").unwrap().data_type();

        assert!(matches!(s1_type.concrete_type(), Types::Struct(_)));
        assert!(matches!(s2_type.concrete_type(), Types::Struct(_)));
        assert!(matches!(s3_type.concrete_type(), Types::Primitive(Primitive::Int32)));
    }

    #[test]
    fn identifier_exists_in_multiple_modules_with_common_partial_scope() {
        // Arrange
        let slice1 = "
            module A
            struct C {
                s1: A::B::S
                s2: ::A::B::S
            }
        ";
        let slice2 = "
            module A::B
            typealias S = string
        ";
        let slice3 = "
            module A::B::A::B
            typealias S = int32
            struct C {
                s1: A::B::S
                s2: ::A::B::S
            }
        ";

        // Act
        let ast = parse_multiple_for_ast(&[slice1, slice2, slice3]);

        // Assert
        let nested_s1_type = ast.find_element::<Field>("A::B::A::B::C::s1").unwrap().data_type();
        let nested_s2_type = ast.find_element::<Field>("A::B::A::B::C::s2").unwrap().data_type();
        let s1_type = ast.find_element::<Field>("A::C::s1").unwrap().data_type();
        let s2_type = ast.find_element::<Field>("A::C::s2").unwrap().data_type();

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
    fn relative_scope_is_module_before_interface() {
        // Arrange
        let slice1 = "
            module A::B::C
            struct S {
                c: C
            }
        ";
        let slice2 = "
            module A
            interface C {}
        ";

        // Act
        let diagnostics = parse_multiple_for_diagnostics(&[slice1, slice2]);

        // Assert
        let expected = Diagnostic::new(Error::TypeMismatch {
            expected: "type".to_string(),
            actual: "module".to_string(),
            is_concrete: false,
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn missing_type_should_fail() {
        // Arrange
        let slice = "
            module A

            struct C {
                b: Nested::C
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::DoesNotExist {
            identifier: "Nested::C".to_string(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn redefinitions_of_the_same_type_are_disallowed() {
        // Arrange
        let slice = "
            mode = Slice1 // Just so we can compare `struct`s and `class`s.

            module A

            compact struct C {
                i: int32
            }

            class C {
                i: int8
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::Redefinition {
            identifier: "C".to_string(),
        })
        .add_note("'C' was previously defined here", None);

        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn redefinitions_of_different_types_are_disallowed() {
        // Arrange
        let slice = "
            module A

            struct C {
                i: int32
            }

            struct C {
                i: int32
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::Redefinition {
            identifier: "C".to_string(),
        })
        .add_note("'C' was previously defined here", None);

        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn multiple_definitions_are_disallowed() {
        // Arrange
        let slice = "
            module Test

            struct A {
                i: int32
            }

            enum A {
                i
            }

            struct A {
                b: bool
                i: int64
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = [
            Diagnostic::new(Error::Redefinition { identifier: "A".to_string() }),
            Diagnostic::new(Error::Redefinition { identifier: "A".to_string() }),
        ];

        check_diagnostics(diagnostics, expected);
    }
}
