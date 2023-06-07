// Copyright (c) ZeroC, Inc.

mod allow;
mod compress;
mod deprecated;
mod oneway;
mod sliced_format;

pub use allow::*;
pub use compress::*;
pub use deprecated::*;
pub use oneway::*;
pub use sliced_format::*;

pub use super::Attributables;
use crate::diagnostics::{Diagnostic, DiagnosticReporter, Error, Warning};
use crate::slice_file::Span;

#[derive(Debug)]
pub struct Unparsed {
    pub directive: String,
    pub args: Vec<String>,
}

impl AttributeKind for Unparsed {
    fn is_repeatable(&self) -> bool {
        true
    }

    // We perform no additional validation for unparsed attributes.
    fn validate_on(&self, _: Attributables, _: &Span, _: &mut DiagnosticReporter) {}

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    // TODO EXPLAIN THIS
    fn dyn_directive(&self) -> &str {
        unreachable!("attempted to get directive of unparsed attribute");
    }
}

pub trait AttributeKind: std::fmt::Debug {
    fn is_repeatable(&self) -> bool;
    fn validate_on(&self, applied_on: Attributables, span: &Span, reporter: &mut DiagnosticReporter);
    fn as_any(&self) -> &dyn std::any::Any;
    fn dyn_directive(&self) -> &str; // Only for error reporting. You should probably use directive if possible.
}

// TODO COMMENT
// Driver trait for attribute parsing. All attributes (except Unparsed) should implement this, allowing them to be automatically parsed.
pub trait ParseableAttributeKind: AttributeKind {
    fn directive() -> &'static str;
    fn parse_from(unparsed: Unparsed, span: &Span, diagnostics: &mut Vec<Diagnostic>) -> Self;
}

macro_rules! implement_attribute_kind_for {
    ($type:ty, $directive:literal, $is_repeatable:literal) => {
        impl AttributeKind for $type {
            fn is_repeatable(&self) -> bool {
                $is_repeatable
            }

            fn validate_on(&self, applied_on: Attributables, span: &Span, reporter: &mut DiagnosticReporter) {
                Self::validate_on(&self, applied_on, span, reporter);
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn dyn_directive(&self) -> &str {
                Self::directive()
            }
        }

        impl ParseableAttributeKind for $type {
            fn directive() -> &'static str {
                $directive
            }

            fn parse_from(unparsed: Unparsed, span: &Span, diagnostics: &mut Vec<Diagnostic>) -> Self {
                Self::parse_from(unparsed, span, diagnostics)
            }
        }
    };
}

pub(self) use implement_attribute_kind_for;

pub fn report_unexpected_attribute<T: ParseableAttributeKind>(
    span: &Span,
    note: Option<&str>,
    diagnostic_reporter: &mut DiagnosticReporter,
) {
    let mut diagnostic = Diagnostic::new(Error::UnexpectedAttribute {
        attribute: T::directive().to_owned(), // Can we just use a static string here?
    })
    .set_span(span);

    if let Some(note) = note {
        diagnostic = diagnostic.add_note(note, None);
    }

    diagnostic.report(diagnostic_reporter);
}

// This is a standalone function because it's used by both the `allow` attribute, and the `--allow` CLI option.
pub fn validate_allow_arguments(arguments: &[String], span: Option<&Span>, diagnostics: &mut Vec<Diagnostic>) {
    for argument in arguments {
        let argument_str = &argument.as_str();
        let mut is_valid = Warning::ALLOWABLE_WARNING_IDENTIFIERS.contains(argument_str);

        // We don't allow `DuplicateFile` to be suppressed by attributes, because it's a command-line specific warning.
        // This check works because `span` is `None` for command line flags.
        if argument == "DuplicateFile" && span.is_some() {
            is_valid = false;
        }

        // Emit an error if the argument wasn't valid.
        if !is_valid {
            // TODO we should emit a link to the warnings page when we write it!
            let mut error = Diagnostic::new(Error::ArgumentNotSupported {
                argument: argument.to_owned(),
                directive: "allow".to_owned(),
            });

            if let Some(unwrapped_span) = span {
                error = error.set_span(unwrapped_span);
            }

            // Check if the argument only differs in case from a valid one.
            let suggestion = Warning::ALLOWABLE_WARNING_IDENTIFIERS
                .iter()
                .find(|identifier| identifier.eq_ignore_ascii_case(argument_str));
            if let Some(identifier) = suggestion {
                let message = format!("attribute arguments are case sensitive, perhaps you meant '{identifier}'?");
                error = error.add_note(message, None);
            }

            diagnostics.push(error);
        }
    }
}
