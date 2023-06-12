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

use super::Attributables;
use crate::diagnostics::{Diagnostic, DiagnosticReporter, Error, Warning};
use crate::slice_file::Span;

pub trait AttributeKind: std::fmt::Debug {
    fn is_repeatable(&self) -> bool;
    fn validate_on(&self, applied_on: Attributables, span: &Span, reporter: &mut DiagnosticReporter);
    fn as_any(&self) -> &dyn std::any::Any;
    fn directive(&self) -> &str;
}

#[macro_export] // We export this macro so languages can implement their own attributes.
macro_rules! implement_attribute_kind_for {
    ($type:ty, $directive:literal, $is_repeatable:literal) => {
        impl $type {
            pub fn directive() -> &'static str {
                $directive
            }
        }

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

            fn directive(&self) -> &str {
                Self::directive()
            }
        }
    };
}

pub use implement_attribute_kind_for;

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

    fn directive(&self) -> &str {
        &self.directive
    }
}

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
