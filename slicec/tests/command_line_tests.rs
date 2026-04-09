// Copyright (c) ZeroC, Inc.

use std::error::Error;

use clap::error::ErrorKind;
use clap::Parser;
use slicec::slice_options::SliceOptions;

use test_case::test_case;

mod test_helpers;

#[test_case("foo"; "without trailing separator")]
#[test_case("foo,"; "with trailing separator")]
fn plugins_can_have_no_arguments(value: &str) {
    // Arrange
    let input = ["", "--generator", value];

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
fn backslash_escapes_special_characters_in_path() {
    // Arrange
    let input = ["", "--generator", "C:\\my\\,folder\\=cool/\\a"];

    // Act
    let result = SliceOptions::try_parse_from(input);

    // Assert
    let parsed_options = result.unwrap();
    assert_eq!(parsed_options.generators.len(), 1);

    let generator_plugin = &parsed_options.generators[0];
    assert_eq!(generator_plugin.path, "C:\\my,folder=cool/\\a");
    assert!(generator_plugin.args.is_empty());
}

#[test]
fn backslash_escapes_special_characters_in_arguments() {
    // Arrange
    let input = ["", "--generator", "C:\\plugin,my\\=name\\,=my\\=value\\,,"];

    // Act
    let result = SliceOptions::try_parse_from(input);

    // Assert
    let parsed_options = result.unwrap();
    assert_eq!(parsed_options.generators.len(), 1);

    let generator_plugin = &parsed_options.generators[0];
    assert_eq!(generator_plugin.path, "C:\\plugin");
    assert_eq!(generator_plugin.args.len(), 1);

    assert_eq!(generator_plugin.args[0].0, "my=name,");
    assert_eq!(generator_plugin.args[0].1, "my=value,");
}

#[test]
fn backslash_does_not_escape_normal_characters() {
    // Arrange
    let input = ["", "--generator", "C:\\Users\\Me\\_plugin,na\\me=\\value\\"];

    // Act
    let result = SliceOptions::try_parse_from(input);

    // Assert
    let parsed_options = result.unwrap();
    assert_eq!(parsed_options.generators.len(), 1);

    let generator_plugin = &parsed_options.generators[0];
    assert_eq!(generator_plugin.path, "C:\\Users\\Me\\_plugin");
    assert_eq!(generator_plugin.args.len(), 1);

    assert_eq!(generator_plugin.args[0].0, "na\\me");
    assert_eq!(generator_plugin.args[0].1, "\\value\\");
}

#[test_case("C:\\my_plugin,arg1=val1"; "without trailing separator")]
#[test_case("C:\\my_plugin,arg1=val1,"; "with trailing separator")]
fn plugins_can_have_a_single_argument(value: &str) {
    // Arrange
    let input = ["", "--generator", value];

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
    let input = ["", "--generator", "/path/to/plugin,	arg1  = val1 	 "];

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

#[test_case("../plugin/path,foo,bar"; "without trailing equals")]
#[test_case("../plugin/path,foo=,bar="; "with trailing equals")]
#[test_case("../plugin/path,foo,bar=,"; "with trailing equals and separator")]
fn plugins_arguments_can_omit_a_value(value: &str) {
    // Arrange
    let input = ["", "--generator", value];

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
    let input = ["", "--generator", "foo,arg1=val1,arg2, arg3 = val3 "];

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

#[test_case("foo,="; "trailing equals")]
#[test_case("foo,=val"; "trailing equals with value")]
#[test_case("foo,,"; "trailing separator")]
fn argument_key_must_be_present(generator_arg: &str) {
    // Arrange
    let input = ["", "--generator", generator_arg];

    // Act
    let result = SliceOptions::try_parse_from(input);

    // Assert
    let parsing_error = result.unwrap_err();
    assert_eq!(parsing_error.kind(), ErrorKind::ValueValidation);

    let error_message = parsing_error.source().unwrap().to_string();
    assert_eq!(error_message, "missing argument key (ex: 'PATH,KEY=VALUE')");
}

#[test]
fn double_equals_is_disallowed() {
    // Arrange
    let input = ["", "--generator", "/paths/are=/=okay,arg1=key=1"];

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
