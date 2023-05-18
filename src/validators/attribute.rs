// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{Diagnostic, DiagnosticReporter, Error};
use crate::grammar::*;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;

// Validate the common attributes that almost every type can have.
macro_rules! validate_attributes {
    ($attributable:ident, $diagnostic_reporter:expr) => {
        let attributes = $attributable.attributes(false);
        validate_repeated_attributes(&attributes, $diagnostic_reporter);
        for attribute in attributes {
            match attribute.kind {
                _ => validate_common_attribute(attribute, $diagnostic_reporter),
            }
        }
    };
}

// Validate the common attributes plus the specified attributes.
macro_rules! validate_attributes_including {
    ($attributable:ident, $diagnostic_reporter:expr $(, $attribute:ident)+ ) => {
        let attributes = $attributable.attributes(false);
        validate_repeated_attributes(&attributes, $diagnostic_reporter);
        for attribute in attributes {
            match attribute.kind {
                $(
                    AttributeKind::$attribute { .. } => {}
                )*
                _ => validate_common_attribute(attribute, $diagnostic_reporter),
            }
        }
    };
}

// Validate the common attributes with the exception of the specified attributes (with an additional note).
macro_rules! validate_attributes_excluding {
    ($attributable:ident, $diagnostic_reporter:expr $(, $attribute:ident, $note:expr)+ ) => {
        let attributes = $attributable.attributes(false);
        validate_repeated_attributes(&attributes, $diagnostic_reporter);
        for attribute in attributes {
            match attribute.kind {
                $(
                     AttributeKind::$attribute { .. } =>
                        report_unexpected_attribute(attribute, $diagnostic_reporter, $note),
                )+
                _ => validate_common_attribute(attribute, $diagnostic_reporter),
            }
        }
    };
}

/// Rejects all "known" attributes.
macro_rules! reject_attributes {
    ($attributable:ident, $diagnostic_reporter:expr) => {
        let attributes = $attributable.attributes(false);
        validate_repeated_attributes(&attributes, $diagnostic_reporter);
        for attribute in attributes {
            match attribute.kind {
                _ => reject_known_attribute(attribute, $diagnostic_reporter),
            }
        }
    };
}

pub(crate) use {reject_attributes, validate_attributes, validate_attributes_excluding, validate_attributes_including};

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

pub fn report_unexpected_attribute(
    attribute: &Attribute,
    diagnostic_reporter: &mut DiagnosticReporter,
    unexpected_note: Option<&str>,
) {
    let note = unexpected_note.or(match attribute.kind {
        AttributeKind::Compress { .. } => {
            Some("the compress attribute can only be applied to interfaces and operations")
        }
        AttributeKind::SlicedFormat { .. } => Some("the slicedFormat attribute can only be applied to operations"),
        AttributeKind::Oneway { .. } => Some("the oneway attribute can only be applied to operations"),
        _ => None,
    });

    let mut diagnostic = Diagnostic::new(Error::UnexpectedAttribute {
        attribute: attribute.directive().to_owned(),
    })
    .set_span(&attribute.span);

    if let Some(note) = note {
        diagnostic = diagnostic.add_note(note, None);
    }

    diagnostic.report(diagnostic_reporter);
}

pub fn validate_common_attribute(attribute: &Attribute, diagnostic_reporter: &mut DiagnosticReporter) {
    match &attribute.kind {
        AttributeKind::Allow { .. } => {}
        AttributeKind::Deprecated { .. } => {}
        // Validated by the language code generator.
        AttributeKind::LanguageKind { .. } => {}
        // Allow other language attributes (directives that contain "::" ) through.
        // This is a sufficient check since the compiler rejects `::`, `x::`, and `::x` as invalid identifiers.
        AttributeKind::Other { directive, .. } if directive.contains("::") => {}
        _ => report_unexpected_attribute(attribute, diagnostic_reporter, None),
    }
}

/// Rejects all "known" attributes. For the purposes LanguageKind and Other are "unknown".
pub fn reject_known_attribute(attribute: &Attribute, diagnostic_reporter: &mut DiagnosticReporter) {
    match &attribute.kind {
        // Validated by the language code generator.
        AttributeKind::LanguageKind { .. } => {}
        // Allow other language attributes (directives that contain "::" ) through.
        // This is a sufficient check since the compiler rejects `::`, `x::`, and `::x` as invalid identifiers.
        AttributeKind::Other { directive, .. } if directive.contains("::") => {}
        _ => report_unexpected_attribute(attribute, diagnostic_reporter, None),
    }
}
