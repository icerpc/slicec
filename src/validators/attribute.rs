// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{Diagnostic, DiagnosticReporter, Error};
use crate::grammar::*;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;

pub fn validate_attributes(attributable: &impl Attributable, diagnostic_reporter: &mut DiagnosticReporter) {
    let attributes = attributable.attributes();
    validate_repeated_attributes(&attributes, diagnostic_reporter);
    for attribute in attributes {
        attribute.kind.validate_on(
            attributable.concrete_attributable(),
            attribute.span(),
            diagnostic_reporter,
        );
    }
}

/// Validates a list of attributes to ensure attributes which are not allowed to be repeated are not repeated.
pub fn validate_repeated_attributes(attributes: &[&Attribute], diagnostic_reporter: &mut DiagnosticReporter) {
    let mut first_attribute_occurrence = HashMap::new();

    for attribute in attributes {
        // We only care about attributes that are not allowed to repeat.
        if attribute.kind.is_repeatable() {
            continue;
        }

        let directive = attribute.kind.dyn_directive();
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
