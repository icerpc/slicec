// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{Diagnostic, Diagnostics, Error};
use crate::grammar::AttributeKind;
use crate::slice_file::Span;

/// Reports an error if the provided list of arguments is empty.
pub fn check_that_arguments_were_provided(
    arguments: &[String],
    directive: &str,
    span: &Span,
    diagnostics: &mut Diagnostics,
) {
    if arguments.is_empty() {
        Diagnostic::new(Error::MissingRequiredArgument {
            argument: directive.to_owned(),
        })
        .set_span(span)
        .push_into(diagnostics);
    }
}

/// Reports an error if the provided list of arguments is non-empty.
pub fn check_that_no_arguments_were_provided(
    arguments: &[String],
    directive: &str,
    span: &Span,
    diagnostics: &mut Diagnostics,
) {
    if !arguments.is_empty() {
        Diagnostic::new(Error::TooManyArguments {
            expected: directive.to_owned(),
        })
        .set_span(span)
        .push_into(diagnostics);
    }
}

/// Reports an error if the provided list of arguments has more than 1 element.
pub fn check_that_at_most_one_argument_was_provided(
    arguments: &[String],
    directive: &str,
    span: &Span,
    diagnostics: &mut Diagnostics,
) {
    if arguments.len() > 1 {
        Diagnostic::new(Error::TooManyArguments {
            expected: directive.to_owned(),
        })
        .set_span(span)
        .push_into(diagnostics);
    }
}

/// Reports an error if the provided list of arguments doesn't have exactly 1 element.
pub fn check_that_exactly_one_argument_was_provided(
    arguments: &[String],
    directive: &str,
    span: &Span,
    diagnostics: &mut Diagnostics,
) {
    check_that_arguments_were_provided(arguments, directive, span, diagnostics);
    check_that_at_most_one_argument_was_provided(arguments, directive, span, diagnostics);
}

/// Used to report an error when an attribute is applied to something it shouldn't be.
/// This is only called by attributes in their `validate_on` functions.
pub fn report_unexpected_attribute(
    attribute: &impl AttributeKind,
    span: &Span,
    note: Option<&str>,
    diagnostics: &mut Diagnostics,
) {
    let mut diagnostic = Diagnostic::new(Error::UnexpectedAttribute {
        attribute: attribute.directive().to_owned(),
    })
    .set_span(span);

    if let Some(note) = note {
        diagnostic = diagnostic.add_note(note, None);
    }

    diagnostic.push_into(diagnostics);
}
