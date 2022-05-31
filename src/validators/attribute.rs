// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::error::ErrorReporter;
use crate::grammar::*;
use crate::visitor::Visitor;
use std::str::FromStr;

#[derive(Debug)]
pub struct AttributeValidator<'a> {
    pub error_reporter: &'a mut ErrorReporter,
}

impl AttributeValidator<'_> {
    // Helper
    fn message_value_separator(&self, valid_strings: &[&str]) -> String {
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

    fn validate_format_attribute(&mut self, attribute: &Attribute) {
        match attribute.arguments.len() {
            // The format attribute must have arguments
            0 => self.error_reporter.report_error(
                "format attribute arguments cannot be empty",
                Some(&attribute.location),
            ),
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
                        self.error_reporter.report_error(
                            format!("invalid format attribute argument `{}`", arg),
                            Some(&attribute.location),
                        );
                        self.error_reporter.report_note(
                            format!(
                                "The valid arguments for the format attribute are {}",
                                self.message_value_separator(&["Compact", "Sliced"])
                            ),
                            Some(&attribute.location),
                        );
                    });
            }
        }
    }

    /// Validates that the `deprecated` attribute cannot be applied to operation parameters.
    fn validate_deprecated_parameters(&mut self, attributes: &[Attribute]) {
        attributes.iter().for_each(|attribute| {
            if attribute.directive.as_str() == "deprecated" {
                self.error_reporter.report_error(
                    "the deprecated attribute cannot be applied to parameters",
                    Some(&attribute.location),
                );
            }
        })
    }

    /// Validates that the `deprecated` attribute cannot be applied to data members.
    fn validate_deprecated_data_members(&mut self, attributes: &[Attribute]) {
        attributes.iter().for_each(|attribute| {
            if attribute.directive.as_str() == "deprecated" {
                self.error_reporter.report_error(
                    "the deprecated attribute cannot be applied to data members",
                    Some(&attribute.location),
                );
            }
        })
    }

    /// Validates that the `compress` attribute is not on an disallowed Attributable Elements and
    /// verifies that the user did not provide invalid arguments.
    fn validate_compress_attribute(&mut self, element: &(impl Element + Attributable)) {
        // Validates that the `compress` attribute cannot be applied to anything other than
        // interfaces and operations.
        let supported_on = ["interface", "operation"];
        let kind = element.kind();
        if !supported_on.contains(&kind) {
            match element.get_raw_attribute("compress", false) {
                Some(attribute) => {
                    self.error_reporter.report_error(
                        "the compress attribute can only be applied to interfaces and operations",
                        Some(&attribute.location),
                    );
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
                        self.error_reporter.report_error(
                            format!("invalid argument `{}` for the compress attribute", arg),
                            Some(&attribute.location),
                        );
                        self.error_reporter.report_note(
                            format!(
                                "The valid argument(s) for the compress attribute are {}",
                                self.message_value_separator(&valid_arguments).as_str(),
                            ),
                            Some(&attribute.location),
                        );
                    }
                }),
                None => (),
            }
        }
    }
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
    }
}
