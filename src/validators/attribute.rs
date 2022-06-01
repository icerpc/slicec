// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::error::Error;
use crate::grammar::*;
use crate::validators::{ValidationFunction, ValidatorResult};
use std::str::FromStr;

pub fn attribute_validators() -> Vec<ValidationFunction> {
    return vec![
        ValidationFunction::Attributable(Box::new(validate_compress_attribute)),
        ValidationFunction::Operation(Box::new(validate_format_attribute)),
        ValidationFunction::Parameter(Box::new(validate_deprecated_parameters)),
        ValidationFunction::Members(Box::new(validate_deprecated_data_members)),
    ];
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
fn validate_format_attribute(operation: &Operation) -> ValidatorResult {
    let mut errors = vec![];
    if let Some(attribute) = operation.get_raw_attribute("format", false) {
        match attribute.arguments.len() {
            // The format attribute must have arguments
            0 => errors.push(Error {
                message: "format attribute arguments cannot be empty".to_owned(),
                location: Some(attribute.location.clone()),
                severity: crate::error::ErrorLevel::Error,
            }),
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
                        errors.push(Error {
                            message: format!("invalid format attribute argument `{}`", arg),
                            location: Some(attribute.location.clone()),
                            severity: crate::error::ErrorLevel::Error,
                        });
                        errors.push(Error {
                            message: format!(
                                "The valid arguments for the format attribute are {}",
                                message_value_separator(&["Compact", "Sliced"])
                            ),
                            location: Some(attribute.location.clone()),
                            severity: crate::error::ErrorLevel::Error,
                        });
                    });
            }
        }
    }

    match errors.is_empty() {
        true => Ok(()),
        false => Err(errors),
    }
}

/// Validates that the `deprecated` attribute cannot be applied to operation parameters.
fn validate_deprecated_parameters(parameter: &Parameter) -> ValidatorResult {
    let mut errors = vec![];
    let attributes = parameter.attributes();
    attributes.iter().for_each(|attribute| {
        if attribute.directive.as_str() == "deprecated" {
            errors.push(Error {
                message: "the deprecated attribute cannot be applied to parameters".to_owned(),
                location: Some(attribute.location.clone()),
                severity: crate::error::ErrorLevel::Error,
            });
        }
    });
    match errors.is_empty() {
        true => Ok(()),
        false => Err(errors),
    }
}

/// Validates that the `deprecated` attribute cannot be applied to data members.
fn validate_deprecated_data_members(members: &[&DataMember]) -> ValidatorResult {
    let mut errors = vec![];
    members.iter().for_each(|member| {
        let attributes = member.attributes();
        attributes.iter().for_each(|attribute| {
            if attribute.directive.as_str() == "deprecated" {
                errors.push(Error {
                    message: "the deprecated attribute cannot be applied to data members"
                        .to_owned(),
                    location: Some(attribute.location.clone()),
                    severity: crate::error::ErrorLevel::Error,
                });
            }
        });
    });
    match errors.is_empty() {
        true => Ok(()),
        false => Err(errors),
    }
}

/// Validates that the `compress` attribute is not on an disallowed Attributable Elements and
/// verifies that the user did not provide invalid arguments.
fn validate_compress_attribute(element: &dyn Attributable) -> ValidatorResult {
    // Validates that the `compress` attribute cannot be applied to anything other than
    // interfaces and operations.
    let mut errors = vec![];
    let supported_on = ["interface", "operation"];
    let kind = element.kind();
    if !supported_on.contains(&kind) {
        match element.get_raw_attribute("compress", false) {
            Some(attribute) => errors.push(Error {
                message: "the compress attribute can only be applied to interfaces and operations"
                    .to_owned(),
                location: Some(attribute.location.clone()),
                severity: crate::error::ErrorLevel::Error,
            }),
            None => (),
        }
    }

    // Validate the arguments for the `compress` attribute.
    if supported_on.contains(&kind) {
        let valid_arguments = ["Args", "Return"];
        match element.get_raw_attribute("compress", false) {
            Some(attribute) => attribute.arguments.iter().for_each(|arg| {
                if !valid_arguments.contains(&arg.as_str()) {
                    errors.push(Error {
                        message: format!("invalid argument `{}` for the compress attribute", arg),
                        location: Some(attribute.location.clone()),
                        severity: crate::error::ErrorLevel::Error,
                    });
                    errors.push(Error {
                        message: format!(
                            "The valid argument(s) for the compress attribute are {}",
                            message_value_separator(&valid_arguments).as_str(),
                        ),
                        location: Some(attribute.location.clone()),
                        severity: crate::error::ErrorLevel::Error,
                    });
                }
            }),
            None => (),
        }
    }
    match errors.is_empty() {
        true => Ok(()),
        false => Err(errors),
    }
}
