// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{Diagnostic, DiagnosticReporter, Error};
use crate::grammar::AttributeKind;
use crate::slice_file::Span;

// TODO All these error messages should be improved to mention the actual vs expected number of arguments.
// TODO can these error messages just use static strings instead of owned strings?

// Helper functions for parsing attribute's arguments:

pub fn check_that_no_arguments_were_provided(
    arguments: &Vec<String>,
    directive: &str,
    span: &Span,
    diagnostic_reporter: &mut DiagnosticReporter,
) {
    if !arguments.is_empty() {
        Diagnostic::new(Error::TooManyArguments {
            expected: directive.to_owned(),
        })
        .set_span(span)
        .report(diagnostic_reporter);
    }
}

pub fn check_that_arguments_were_provided(
    arguments: &Vec<String>,
    directive: &str,
    span: &Span,
    diagnostic_reporter: &mut DiagnosticReporter,
) {
    if arguments.is_empty() {
        Diagnostic::new(Error::MissingRequiredArgument {
            argument: directive.to_owned(),
        })
        .set_span(span)
        .report(diagnostic_reporter);
    }
}

pub fn check_that_exactly_one_argument_was_provided(
    arguments: &Vec<String>,
    directive: &str,
    span: &Span,
    diagnostic_reporter: &mut DiagnosticReporter,
) {
    if arguments.is_empty() {
        Diagnostic::new(Error::MissingRequiredArgument {
            argument: directive.to_owned(),
        })
        .set_span(span)
        .report(diagnostic_reporter);
    } else if arguments.len() > 1 {
        Diagnostic::new(Error::TooManyArguments {
            expected: directive.to_owned(),
        })
        .set_span(span)
        .report(diagnostic_reporter);
    }
}

pub fn check_that_at_most_one_argument_was_provided(
    arguments: &Vec<String>,
    directive: &str,
    span: &Span,
    diagnostic_reporter: &mut DiagnosticReporter,
) {
    if arguments.len() > 1 {
        Diagnostic::new(Error::TooManyArguments {
            expected: directive.to_owned(),
        })
        .set_span(span)
        .report(diagnostic_reporter);
    }
}

// Helper functions for validating what an attribute is applied to:

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
