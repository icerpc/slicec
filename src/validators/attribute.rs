// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::error::Error;
use crate::grammar::*;
use crate::validators::{Validate, ValidationChain, ValidationResult};
use std::str::FromStr;

pub fn attribute_validators() -> ValidationChain {
    vec![
        Validate::Attributable(validate_compress_attribute),
        Validate::Operation(validate_format_attribute),
        Validate::Parameter(validate_deprecated_parameters),
        Validate::Members(validate_deprecated_data_members),
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
fn validate_format_attribute(operation: &Operation) -> ValidationResult {
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
fn validate_deprecated_parameters(parameter: &Parameter) -> ValidationResult {
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
fn validate_deprecated_data_members(members: &[&DataMember]) -> ValidationResult {
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
fn validate_compress_attribute(element: &dyn Attributable) -> ValidationResult {
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
<<<<<<< HEAD
    match errors.is_empty() {
        true => Ok(()),
        false => Err(errors),
=======
}

impl<'a> Visitor for AttributeValidator<'a> {
    fn visit_interface_start(&mut self, interface_def: &Interface) {
        self.validate_compress_attribute(interface_def);
    }

    fn visit_operation_start(&mut self, operation: &Operation) {
        self.validate_compress_attribute(operation);
        if let Some(attribute) = operation.get_raw_attribute("format", false) {
            self.validate_format_attribute(attribute);
        }
    }

    fn visit_struct_start(&mut self, struct_def: &Struct) {
        self.validate_compress_attribute(struct_def);
    }

    fn visit_parameter(&mut self, parameter: &Parameter) {
        self.validate_deprecated_parameters(parameter.attributes());
        self.validate_compress_attribute(parameter);
    }

    fn visit_return_member(&mut self, parameter: &Parameter) {
        self.validate_deprecated_parameters(parameter.attributes());
        self.validate_compress_attribute(parameter);
    }

    fn visit_data_member(&mut self, data_member: &DataMember) {
        self.validate_deprecated_data_members(data_member.attributes());
        self.validate_compress_attribute(data_member);
    }

    fn visit_enum_start(&mut self, enum_def: &Enum) {
        self.validate_compress_attribute(enum_def);
    }

    fn visit_exception_start(&mut self, exception_def: &Exception) {
        self.validate_compress_attribute(exception_def);
>>>>>>> origin/main
    }
}
