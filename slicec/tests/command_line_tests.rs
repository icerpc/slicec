// Copyright (c) ZeroC, Inc.

use std::error::Error;

use clap::error::ErrorKind;
use clap::Parser;
use slicec::slice_options::SliceOptions;

use test_case::test_case;

mod test_helpers;

#[test]
fn plugins_can_have_no_arguments() {
    // Arrange
    let input = ["", "--generator", "foo"];

    // Act
    let result = SliceOptions::try_parse_from(input);

    // Assert
    let parsed_options = result.unwrap();
    assert_eq!(parsed_options.generators.len(), 1);

    let generator_plugin = &parsed_options.generators[0];
    assert_eq!(generator_plugin.path, "foo");
    assert!(generator_plugin.args.is_empty());
}

#[test]
fn equals_sign_has_no_special_meaning_in_plugin_path() {
    // Arrange
    let input = ["", "--generator", "../=plugin/=/pa=th"];

    // Act
    let result = SliceOptions::try_parse_from(input);

    // Assert
    let parsed_options = result.unwrap();
    assert_eq!(parsed_options.generators.len(), 1);

    let generator_plugin = &parsed_options.generators[0];
    assert_eq!(generator_plugin.path, "../=plugin/=/pa=th");
    assert!(generator_plugin.args.is_empty());
}

#[test]
fn path_characters_can_be_escaped() {
    // Arrange
    let input = ["", "--generator", "C:\\\\my\\;folder\\=cool/\\a"];

    // Act
    let result = SliceOptions::try_parse_from(input);

    // Assert
    let parsed_options = result.unwrap();
    assert_eq!(parsed_options.generators.len(), 1);

    let generator_plugin = &parsed_options.generators[0];
    assert_eq!(generator_plugin.path, "C:\\my;folder=cool/a");
    assert!(generator_plugin.args.is_empty());
}

#[test]
fn plugins_can_have_a_single_argument() {
    // Arrange
    let input = ["", "--generator", "C:\\\\my_plugin;arg1=val1"];

    // Act
    let result = SliceOptions::try_parse_from(input);

    // Assert
    let parsed_options = result.unwrap();
    assert_eq!(parsed_options.generators.len(), 1);

    let generator_plugin = &parsed_options.generators[0];
    assert_eq!(generator_plugin.path, "C:\\my_plugin");
    assert_eq!(generator_plugin.args.len(), 1);

    assert_eq!(generator_plugin.args[0].0, "arg1");
    assert_eq!(generator_plugin.args[0].1, "val1");
}

#[test]
fn plugins_arguments_have_whitespace_trimmed() {
    // Arrange
    let input = ["", "--generator", "/path/to/plugin;	arg1  = val1 	 "];

    // Act
    let result = SliceOptions::try_parse_from(input);

    // Assert
    let parsed_options = result.unwrap();
    assert_eq!(parsed_options.generators.len(), 1);

    let generator_plugin = &parsed_options.generators[0];
    assert_eq!(generator_plugin.path, "/path/to/plugin");
    assert_eq!(generator_plugin.args.len(), 1);

    assert_eq!(generator_plugin.args[0].0, "arg1");
    assert_eq!(generator_plugin.args[0].1, "val1");
}

#[test]
fn plugins_arguments_can_omit_a_value() {
    // Arrange
    let input = ["", "--generator", "../plugin/path;foo;bar"];

    // Act
    let result = SliceOptions::try_parse_from(input);

    // Assert
    let parsed_options = result.unwrap();
    assert_eq!(parsed_options.generators.len(), 1);

    let generator_plugin = &parsed_options.generators[0];
    assert_eq!(generator_plugin.path, "../plugin/path");
    assert_eq!(generator_plugin.args.len(), 2);

    assert_eq!(generator_plugin.args[0].0, "foo");
    assert_eq!(generator_plugin.args[0].1, "");
    assert_eq!(generator_plugin.args[1].0, "bar");
    assert_eq!(generator_plugin.args[1].1, "");
}

#[test]
fn plugins_can_have_multiple_arguments() {
    // Arrange
    let input = ["", "--generator", "foo;arg1=val1;arg2;arg3 = val3"];

    // Act
    let result = SliceOptions::try_parse_from(input);

    // Assert
    let parsed_options = result.unwrap();
    assert_eq!(parsed_options.generators.len(), 1);

    let generator_plugin = &parsed_options.generators[0];
    assert_eq!(generator_plugin.path, "foo");
    assert_eq!(generator_plugin.args.len(), 3);

    assert_eq!(generator_plugin.args[0].0, "arg1");
    assert_eq!(generator_plugin.args[0].1, "val1");
    assert_eq!(generator_plugin.args[1].0, "arg2");
    assert_eq!(generator_plugin.args[1].1, "");
    assert_eq!(generator_plugin.args[2].0, "arg3");
    assert_eq!(generator_plugin.args[2].1, "val3");
}

#[test_case("foo;arg="; "without trailing argument")]
#[test_case("foo;arg1=;arg2"; "with trailing argument")]
fn argument_value_must_appear_after_equals(generator_arg: &str) {
    // Arrange
    let input = ["", "--generator", generator_arg];

    // Act
    let result = SliceOptions::try_parse_from(input);

    // Assert
    let parsing_error = result.unwrap_err();
    assert_eq!(parsing_error.kind(), ErrorKind::ValueValidation);

    let error_message = parsing_error.source().unwrap().to_string();
    assert_eq!(
        error_message,
        "missing argument value (ex: 'PATH;KEY=VALUE' or 'PATH;KEY')",
    );
}

#[test_case("foo;="; "trailing equals")]
#[test_case("foo;=val"; "trailing equals with value")]
#[test_case("foo;;"; "trailing semicolon")]
fn argument_key_must_be_present(generator_arg: &str) {
    // Arrange
    let input = ["", "--generator", generator_arg];

    // Act
    let result = SliceOptions::try_parse_from(input);

    // Assert
    let parsing_error = result.unwrap_err();
    assert_eq!(parsing_error.kind(), ErrorKind::ValueValidation);

    let error_message = parsing_error.source().unwrap().to_string();
    assert_eq!(error_message, "missing argument key (ex: 'PATH;KEY=VALUE')");
}

#[test]
fn double_equals_is_disallowed() {
    // Arrange
    let input = ["", "--generator", "/paths/are=/=okay;arg1=key=1"];

    // Act
    let result = SliceOptions::try_parse_from(input);

    // Assert
    let parsing_error = result.unwrap_err();
    assert_eq!(parsing_error.kind(), ErrorKind::ValueValidation);

    let error_message = parsing_error.source().unwrap().to_string();
    assert_eq!(
        error_message,
        "'=' can only appear once per argument (for a literal '=' character, use '\\=')",
    );
}
