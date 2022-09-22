// Copyright (c) ZeroC, Inc. All rights reserved.

mod output {

    use slice::parse_from_string;

    #[test]
    fn output_to_json() {
        let slice = r#"
        module  Foo;

        interface I {
            /// @param x this is an x
            op();
        }

        enum E {}
        "#;
        let parsed_data = match parse_from_string(slice) {
            Err(data) => data,
            _ => panic!("Expected error"),
        };
        let mut output = Vec::new();
        parsed_data.emit_diagnostics(&mut output);

        // eprintln!("size: {}", output.len());
        // eprintln!("{}", String::from_utf8(output).unwrap());

        panic!("{}", String::from_utf8(output).unwrap())
    }
}
