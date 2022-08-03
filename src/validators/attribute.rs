// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::errors::*;
use crate::grammar::*;
use crate::validators::{ValidationChain, Validator};
use std::str::FromStr;

pub fn attribute_validators() -> ValidationChain {
    vec![
        Validator::Attributes(is_compressible),
        Validator::Operations(validate_format_attribute),
        Validator::Members(cannot_be_deprecated),
    ]
}

/// Helper
fn message_value_separator(valid_strings: &[&str]) -> String {
    let separator = match valid_strings.len() {
        0 | 1 => "",
        2 => " and ",
        _ => ", ",
    };
    let mut backtick_strings = valid_strings
        .iter()
        .map(|arg| "`".to_owned() + arg + "`")
        .collect::<Vec<_>>();
    match valid_strings.len() {
        0 | 1 | 2 => backtick_strings.join(separator),
        _ => {
            let last = backtick_strings.pop().unwrap();
            backtick_strings.join(separator) + ", and " + &last
        }
    }
}

/// Attribute validators
fn validate_format_attribute(operation: &Operation, error_reporter: &mut ErrorReporter) {
    if let Some(attribute) = operation.get_raw_attribute("format", false) {
        match attribute.arguments.len() {
            // The format attribute must have arguments
            0 => error_reporter.report(LogicKind::CannotBeEmpty("format attribute"), Some(attribute.span())),
            _ => {
                // Validate format attributes are allowed ones.
                attribute
                    .arguments
                    .iter()
                    .filter(|arg| {
                        let format = ClassFormat::from_str(arg.as_str());
                        format.is_err()
                    })
                    .for_each(|arg| {
                        error_reporter.report(
                            LogicKind::ArgumentNotSupported(arg.to_owned(), "format attribute".to_owned()),
                            Some(attribute.span()),
                        );
                        error_reporter.report(
                            ErrorKind::new_note(format!(
                                "The valid arguments for the format attribute are {}",
                                message_value_separator(&["Compact", "Sliced"])
                            )),
                            Some(attribute.span()),
                        );
                    });
            }
        }
    }
}

/// Validates that the `deprecated` attribute cannot be applied to members.
fn cannot_be_deprecated(members: Vec<&dyn Member>, error_reporter: &mut ErrorReporter) {
    members.iter().for_each(|m| {
        if m.has_attribute("deprecated", false) {
            error_reporter.report(
                LogicKind::DeprecatedAttributeCannotBeApplied(m.kind().to_owned() + "(s)"),
                Some(m.span()),
            );
        }
    });
}

/// Validates that the `compress` attribute is not on an disallowed Attributable Elements and
/// verifies that the user did not provide invalid arguments.
fn is_compressible(element: &dyn Attributable, error_reporter: &mut ErrorReporter) {
    // Validates that the `compress` attribute cannot be applied to anything other than
    // interfaces and operations.
    let supported_on = ["interface", "operation"];
    let kind = element.kind();
    if !supported_on.contains(&kind) {
        match element.get_raw_attribute("compress", false) {
            Some(attribute) => {
                error_reporter.report(LogicKind::CompressAttributeCannotBeApplied, Some(attribute.span()))
            }
            None => (),
        }
    }

    // Validate the arguments for the `compress` attribute.
    if supported_on.contains(&kind) {
        let valid_arguments = ["Args", "Return"];
        match element.get_raw_attribute("compress", false) {
            Some(attribute) => attribute.arguments.iter().for_each(|arg| {
                if !valid_arguments.contains(&arg.as_str()) {
                    error_reporter.report(
                        LogicKind::ArgumentNotSupported(arg.to_owned(), "compress attribute".to_owned()),
                        Some(attribute.span()),
                    );
                    error_reporter.report(
                        ErrorKind::new_note(format!(
                            "The valid argument(s) for the compress attribute are {}",
                            message_value_separator(&valid_arguments).as_str(),
                        )),
                        Some(attribute.span()),
                    );
                }
            }),
            None => (),
        }
    }
}
