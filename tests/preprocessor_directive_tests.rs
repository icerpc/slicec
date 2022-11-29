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

        let mut default_options = SliceOptions::default();
        default_options.definitions = vec!["Foo".to_string()];

        // Act
        let compilation_data = compile_from_strings(&[slice], Some(default_options)).unwrap();

        // Assert
        assert!(compilation_data.ast.find_element::<Operation>("Test::I::op").is_ok());
    }
}
