// Copyright (c) ZeroC, Inc.

use super::*;
use super::super::Attributables;
use crate::diagnostics::{Diagnostic, DiagnosticReporter, Error};

#[derive(Debug)]
pub struct SlicedFormat {
    pub sliced_args: bool,
    pub sliced_return: bool,
}

impl SlicedFormat {
    pub fn parse_from(Unparsed { directive, args }: Unparsed, span: &Span, diagnostics: &mut Vec<Diagnostic>) -> Self {
        debug_assert_eq!(directive, Self::directive());

        let (mut sliced_args, mut sliced_return) = (false, false);
        for arg in args {
            match arg.as_str() {
                "Args" => {
                    // TODO should we report a warning/error for duplicates?
                    sliced_args = true;
                }
                "Return" => {
                    // TODO should we report a warning/error for duplicates?
                    sliced_return = true;
                }
                _ => {
                    let diagnostic = Diagnostic::new(Error::ArgumentNotSupported {
                        argument: arg,
                        directive: Self::directive().to_owned(),
                    })
                    .set_span(span)
                    .add_note("'Args' and 'Return' are the only valid arguments", None);
                    diagnostics.push(diagnostic);
                }
            }
        }

        SlicedFormat { sliced_args, sliced_return }
    }

    pub fn validate_on(&self, applied_on: Attributables, span: &Span, reporter: &mut DiagnosticReporter) {
        if !matches!(applied_on, Attributables::Operation(_)) {
            let note = "the slicedFormat attribute can only be applied to operations";
            report_unexpected_attribute::<Self>(span, Some(note), reporter);
        }
    }
}

implement_attribute_kind_for!(SlicedFormat, "slicedFormat", false);
