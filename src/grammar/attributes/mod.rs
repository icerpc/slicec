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
    fn parse_from(unparsed: &Unparsed, span: &Span, reporter: &mut DiagnosticReporter) -> Self;
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

            fn parse_from(unparsed: &Unparsed, span: &Span, reporter: &mut DiagnosticReporter) -> Self {
                Self::parse_from(unparsed, span, reporter)
            }
        }
    };
}

pub(self) use implement_attribute_kind_for;

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
