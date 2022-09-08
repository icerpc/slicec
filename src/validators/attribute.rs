// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::diagnostics::*;
use crate::grammar::*;
use crate::validators::{ValidationChain, Validator};

use std::str::FromStr;

pub fn attribute_validators() -> ValidationChain {
    vec![
        Validator::Attributes(is_compressible),
        Validator::Operations(validate_format_attribute),
        Validator::Parameters(cannot_be_deprecated),
        Validator::DataMember(cannot_use_deprecated_type),
        Validator::Interface(cannot_inherit_deprecated_type),
        Validator::TypeAlias(cannot_type_alias_deprecated_type),
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
fn validate_format_attribute(operation: &Operation, diagnostic_reporter: &mut DiagnosticReporter) {
    if let Some(attribute) = operation.get_raw_attribute("format", false) {
        match attribute.arguments.len() {
            // The format attribute must have arguments
            0 => diagnostic_reporter.report(Diagnostic::new(
                LogicErrorKind::CannotBeEmpty("format attribute".to_owned()),
                Some(attribute.span()),
            )),
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
                        let diagnostic = Diagnostic::new_with_notes(
                            LogicErrorKind::ArgumentNotSupported(arg.to_owned(), "format attribute".to_owned()),
                            Some(attribute.span()),
                            vec![Note::new(
                                format!(
                                    "The valid arguments for the format attribute are {}",
                                    message_value_separator(&["Compact", "Sliced"])
                                ),
                                Some(attribute.span()),
                            )],
                        );
                        diagnostic_reporter.report(diagnostic);
                    });
            }
        }
    }
}
/// Validates that the `deprecated` attribute cannot be applied to parameters.
fn cannot_be_deprecated(parameters: &[&Parameter], diagnostic_reporter: &mut DiagnosticReporter) {
    parameters.iter().for_each(|m| {
        if m.has_attribute("deprecated", false) {
            let diagnostic = Diagnostic::new(
                LogicErrorKind::DeprecatedAttributeCannotBeApplied(m.kind().to_owned() + "(s)"),
                Some(m.span()),
            );
            diagnostic_reporter.report(diagnostic);
        }
    });
}

// Validates that a `DataMember` cannot have a deprecated datatype
fn cannot_use_deprecated_type(data_member: &[&DataMember], diagnostic_reporter: &mut DiagnosticReporter) {
    for member in data_member.iter() {
        if underlying_is_deprecated(member.data_type().concrete_type()) {
            diagnostic_reporter.report(Diagnostic::new(WarningKind::UseOfDeprecatedEntity, Some(member.span())));
        }
    }
}

fn underlying_is_deprecated(concrete_type: Types) -> bool {
    match concrete_type {
        Types::Class(class_def) => class_def.has_attribute("deprecated", false),
        Types::Struct(struct_def) => struct_def.has_attribute("deprecated", false),
        Types::Enum(enum_def) => enum_def.has_attribute("deprecated", false),
        Types::Exception(exception_def) => exception_def.has_attribute("deprecated", false),
        Types::Interface(interface_def) => interface_def.has_attribute("deprecated", false),
        _ => false,
    }
}

// Validates that an interface cannot have a deprecated underlying type
fn cannot_inherit_deprecated_type(interface: &Interface, diagnostic_reporter: &mut DiagnosticReporter) {
    for i in interface.base_interfaces() {
        if i.has_attribute("deprecated", false) {
            diagnostic_reporter.report(Diagnostic::new(WarningKind::UseOfDeprecatedEntity, Some(i.span())));
        }
    }
}

fn cannot_type_alias_deprecated_type(type_alias: &TypeAlias, diagnostic_reporter: &mut DiagnosticReporter) {
    if underlying_is_deprecated(type_alias.underlying.concrete_type()) {
        diagnostic_reporter.report(Diagnostic::new(
            WarningKind::UseOfDeprecatedEntity,
            Some(type_alias.span()),
        ));
    }
}

/// Validates that the `compress` attribute is not on an disallowed Attributable Elements and
/// verifies that the user did not provide invalid arguments.
fn is_compressible(element: &dyn Attributable, diagnostic_reporter: &mut DiagnosticReporter) {
    // Validates that the `compress` attribute cannot be applied to anything other than
    // interfaces and operations.
    let supported_on = ["interface", "operation"];
    let kind = element.kind();
    if !supported_on.contains(&kind) {
        if let Some(attribute) = element.get_raw_attribute("compress", false) {
            diagnostic_reporter.report(Diagnostic::new(
                LogicErrorKind::CompressAttributeCannotBeApplied,
                Some(attribute.span()),
            ));
        }
    }

    // Validate the arguments for the `compress` attribute.
    if supported_on.contains(&kind) {
        let valid_arguments = ["Args", "Return"];
        if let Some(attribute) = element.get_raw_attribute("compress", false) {
            attribute.arguments.iter().for_each(|arg| {
                if !valid_arguments.contains(&arg.as_str()) {
                    let diagnostic = Diagnostic::new_with_notes(
                        LogicErrorKind::ArgumentNotSupported(arg.to_owned(), "compress attribute".to_owned()),
                        Some(attribute.span()),
                        vec![Note::new(
                            format!(
                                "The valid argument(s) for the compress attribute are {}",
                                message_value_separator(&valid_arguments).as_str(),
                            ),
                            Some(attribute.span()),
                        )],
                    );
                    diagnostic_reporter.report(diagnostic);
                }
            })
        }
    }
}
