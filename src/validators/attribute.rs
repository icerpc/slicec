// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{Diagnostic, DiagnosticReporter, Error};
use crate::grammar::*;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;

macro_rules! validate_attributes {
    // Validate the common attributes that almost every type can have (for exceptions see @allow_common_except).
    ($attributable:ident, $diagnostic_reporter:expr $(, $allowed_attribute:ident)* ) => {
        let attributes = $attributable.attributes(false);
        validate_repeated_attributes(&attributes, $diagnostic_reporter);
        for attribute in attributes {
            match attribute.kind {
                $(
                    AttributeKind::$allowed_attribute { .. } => {}
                )*
                _ => validate_common_attribute(attribute, $diagnostic_reporter),
            }
        }
    };

    // Validate the common attributes with the exception of the specified attributes (with an additional note).
    (@allow_common_except $attributable:ident, $diagnostic_reporter:expr $(, $allowed_attribute:ident, $note:expr)+ ) => {
        let attributes = $attributable.attributes(false);
        validate_repeated_attributes(&attributes, $diagnostic_reporter);
        for attribute in attributes {
            match attribute.kind {
                $(
                    AttributeKind::$allowed_attribute { .. } => {
                        Diagnostic::new(Error::UnexpectedAttribute {
                            attribute: attribute.directive().to_owned(),
                        })
                        .set_span(attribute.span())
                        .add_note($note, None)
                        .report($diagnostic_reporter);
                    }
                )+
                _ => validate_common_attribute(attribute, $diagnostic_reporter),
            }
        }
    };

    // Deny all attributes except the specified ones.
    (@deny_all_except $attributable:ident, $diagnostic_reporter:expr $(, $allowed_attribute:ident)+ ) => {
        let attributes = $attributable.attributes(false);
        validate_repeated_attributes(&attributes, $diagnostic_reporter);
        for attribute in attributes {
            match attribute.kind {
                $(
                    AttributeKind::$allowed_attribute { .. } => {}
                )+
                _ => {
                    Diagnostic::new(Error::UnexpectedAttribute {
                        attribute: attribute.directive().to_owned(),
                    })
                    .set_span(attribute.span())
                    .report($diagnostic_reporter);
                }
            }
        }
    };
}

pub(crate) use validate_attributes;

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

pub fn validate_common_attribute(attribute: &Attribute, diagnostic_reporter: &mut DiagnosticReporter) {
    match attribute.kind {
        AttributeKind::Allow { .. } => {}
        AttributeKind::Deprecated { .. } => {}
        AttributeKind::LanguageKind { .. } => {} // Validated by the language code generator.
        AttributeKind::Other { .. } => {}        // Allow unknown attributes through.
        _ => report_unexpected_attribute(attribute, diagnostic_reporter),
    }
}

fn report_unexpected_attribute(attribute: &Attribute, diagnostic_reporter: &mut DiagnosticReporter) {
    let note = match attribute.kind {
        AttributeKind::Compress { .. } => {
            Some("the compress attribute can only be applied to interfaces and operations")
        }
        AttributeKind::SlicedFormat { .. } => Some("the slicedFormat attribute can only be applied to operations"),
        AttributeKind::Oneway { .. } => Some("the oneway attribute can only be applied to operations"),
        _ => None,
    };

    let mut diagnostic = Diagnostic::new(Error::UnexpectedAttribute {
        attribute: attribute.directive().to_owned(),
    })
    .set_span(&attribute.span);

    if let Some(note) = note {
        diagnostic = diagnostic.add_note(note, None);
    }

    diagnostic.report(diagnostic_reporter);
}
