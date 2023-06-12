// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{Diagnostic, DiagnosticReporter, Error};
use crate::grammar::AttributeKind;
use crate::slice_file::Span;

pub fn report_unexpected_attribute(
    attribute: &impl AttributeKind,
    span: &Span,
    note: Option<&str>,
    diagnostic_reporter: &mut DiagnosticReporter,
) {
    let mut diagnostic = Diagnostic::new(Error::UnexpectedAttribute {
        attribute: attribute.directive().to_owned(), // Can we just use a static string here?
    })
    .set_span(span);

    if let Some(note) = note {
        diagnostic = diagnostic.add_note(note, None);
    }

    diagnostic.report(diagnostic_reporter);
}
