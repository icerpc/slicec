// Copyright (c) ZeroC, Inc. All rights reserved.

mod command_line {

    use slice::command_line::SliceOptions;
    use slice::compile_from_strings;
    use slice::grammar::*;

    #[test]
    fn command_line_defined_symbols() {
        // Arrange
        let slice = "
        module Test;

        # if Foo
        interface I
        {
            op();
        }
        # endif
        ";

        let options = SliceOptions {
            definitions: vec!["Foo".to_string()],
            ..Default::default()
        };

        // Act
        let compilation_data = compile_from_strings(&[slice], Some(options)).unwrap();

        // Assert
        assert!(compilation_data.ast.find_element::<Operation>("Test::I::op").is_ok());
    }

    #[test]
    fn undefined_preprocessor_directive_blocks_are_consumed() {
        // Arrange
        let slice = "
            #if Foo
            module Test;
            interface I {}
            #endif
        ";

        // Act
        let compilation_data = compile_from_strings(&[slice], Some(SliceOptions::default())).unwrap();

        // Assert
        assert!(compilation_data.ast.find_element::<Interface>("Test::I").is_err());
        assert!(!compilation_data.diagnostic_reporter.has_errors());
    }

    #[test]
    fn preprocessor_consumes_comments() {
        // Arrange
        let slice = "// This is a comment";

        // Act
        let compilation_data = compile_from_strings(&[slice], Some(SliceOptions::default())).unwrap();

        // Assert
        assert!(!compilation_data.diagnostic_reporter.has_errors());
    }
}
