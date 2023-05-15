// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{Diagnostic, DiagnosticReporter, Error};
use crate::grammar::*;
use crate::validators::{ValidationChain, Validator};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;

pub fn attribute_validators() -> ValidationChain {
    vec![
        // TODO improve this system of checking attribute applicability.
        Validator::Attributes(is_compressible),
        Validator::Attributes(check_sliced_format),
        Validator::Attributes(is_repeated),
        Validator::Parameters(cannot_be_deprecated),
    ]
}

/// Validates a list of attributes to ensure attributes which are not allowed to be repeated are not repeated.
pub fn validate_repeated_attributes(attributes: &[&Attribute], diagnostic_reporter: &mut DiagnosticReporter) {
    let mut first_attribute_occurrence = HashMap::new();

    for attribute in attributes {
        // We only care about attributes that are not allowed to repeat.
        if attribute.kind.is_repeatable() {
            continue;
        }

        let directive = attribute.directive();
        let span = attribute.span();

        match first_attribute_occurrence.entry(directive) {
            Occupied(entry) => {
                Diagnostic::new(Error::AttributeIsNotRepeatable {
                    attribute: directive.to_owned(),
                })
                .set_span(span)
                .add_note("attribute was previously used here", Some(entry.get()))
                .report(diagnostic_reporter);
            }
            Vacant(entry) => {
                entry.insert(span.clone());
            }
        }
    }
}

/// Validates that the `deprecated` attribute cannot be applied to parameters.
fn cannot_be_deprecated(parameters: &[&Parameter], diagnostic_reporter: &mut DiagnosticReporter) {
    for parameter in parameters {
        let deprecated = parameter
            .attributes(false)
            .into_iter()
            .find(|a| matches!(a.kind, AttributeKind::Deprecated { .. }));
        if let Some(attribute) = deprecated {
            Diagnostic::new(Error::UnexpectedAttribute {
                attribute: "deprecated".to_owned(),
            })
            .set_span(attribute.span())
            .add_note("parameters can not be individually deprecated", None)
            .report(diagnostic_reporter)
        }
    }
}

/// Validate that the `compress` attribute is only applied to interfaces and operations.
fn is_compressible(element: &dyn Entity, diagnostic_reporter: &mut DiagnosticReporter) {
    let supported_on = ["interface", "operation"];
    let kind = element.kind();

    if !supported_on.contains(&kind) {
        if let Some(attribute) = element
            .attributes(false)
            .into_iter()
            .find(|a| matches!(a.kind, AttributeKind::Compress { .. }))
        {
            Diagnostic::new(Error::UnexpectedAttribute {
                attribute: "compress".to_owned(),
            })
            .set_span(attribute.span())
            .add_note(
                "the compress attribute can only be applied to interfaces and operations",
                None,
            )
            .report(diagnostic_reporter);
        }
    }
}

/// Validate that the `slicedFormat` attribute is only applied to operations.
fn check_sliced_format(element: &dyn Entity, diagnostic_reporter: &mut DiagnosticReporter) {
    if element.kind() != "operation" {
        if let Some(attribute) = element
            .attributes(false)
            .into_iter()
            .find(|a| matches!(a.kind, AttributeKind::SlicedFormat { .. }))
        {
            Diagnostic::new(Error::UnexpectedAttribute {
                attribute: "slicedFormat".to_owned(),
            })
            .set_span(attribute.span())
            .add_note("the slicedFormat attribute can only be applied to operations", None)
            .report(diagnostic_reporter);
        }
    }
}

/// Validates that the common (not language specific) attributes which are not allowed to be repeated are not repeated.
fn is_repeated(element: &dyn Entity, diagnostic_reporter: &mut DiagnosticReporter) {
    let attributes = element.attributes(false);
    validate_repeated_attributes(&attributes, diagnostic_reporter);
}
