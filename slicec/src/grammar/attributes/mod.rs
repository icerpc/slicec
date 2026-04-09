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
use crate::diagnostics::{Diagnostic, Diagnostics, Error, Lint};
use crate::slice_file::Span;

use std::ops::Range;

pub trait AttributeKind: std::fmt::Debug {
    fn is_repeatable(&self) -> bool;
    fn validate_on(&self, applied_on: Attributables, span: &Span, diagnostics: &mut Diagnostics);
    fn as_any(&self) -> &dyn std::any::Any;
    fn directive(&self) -> &str;
}

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

            fn validate_on(&self, applied_on: Attributables, span: &Span, diagnostics: &mut Diagnostics) {
                Self::validate_on(&self, applied_on, span, diagnostics);
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
pub(crate) use implement_attribute_kind_for;

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
    fn validate_on(&self, _: Attributables, _: &Span, _: &mut Diagnostics) {}

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn directive(&self) -> &str {
        &self.directive
    }
}

/// Reports an error when an attribute is applied to something it shouldn't be.
fn report_invalid_attribute(
    attribute: &impl AttributeKind,
    span: &Span,
    note: Option<&str>,
    diagnostics: &mut Diagnostics,
) {
    let mut diagnostic = Diagnostic::new(Error::InvalidAttribute {
        directive: attribute.directive().to_owned(),
    })
    .set_span(span);

    if let Some(note) = note {
        diagnostic = diagnostic.add_note(note, None);
    }

    diagnostic.push_into(diagnostics);
}

/// Reports an error if an incorrect number of attributes was provided to the specified attribute.
///
/// # Arguments
///
/// * `range` - Range containing the allowed number of arguments for the attribute
/// * `arguments` - The arguments that were provided to the attribute
/// * `directive` - The attribute's directive
/// * `span` - The attribute's span
/// * `diagnostics` - Diagnostics into which errors will be reported
fn check_argument_count_is_within(
    range: Range<usize>,
    arguments: &[String],
    directive: &str,
    span: &Span,
    diagnostics: &mut Diagnostics,
) {
    if !range.contains(&arguments.len()) {
        Diagnostic::new(Error::IncorrectAttributeArgumentCount {
            directive: directive.to_owned(),
            expected_count: range,
            actual_count: arguments.len(),
        })
        .set_span(span)
        .push_into(diagnostics);
    }
}
