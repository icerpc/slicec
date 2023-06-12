// Copyright (c) ZeroC, Inc.

use super::*;

#[derive(Debug)]
pub struct Deprecated {
    pub reason: Option<String>,
}

impl Deprecated {
    pub fn parse_from(Unparsed { directive, args }: &Unparsed, span: &Span, reporter: &mut DiagnosticReporter) -> Self {
        debug_assert_eq!(directive, Self::directive());

        if args.len() > 1 {
            Diagnostic::new(Error::TooManyArguments {
                expected: Self::directive().to_owned(),
            })
            .set_span(span)
            .add_note("The deprecated attribute takes at most one argument", Some(span))
            .report(reporter);
        }

        Deprecated {
            reason: args.first().cloned(),
        }
    }

    pub fn validate_on(&self, applied_on: Attributables, span: &Span, reporter: &mut DiagnosticReporter) {
        match applied_on {
            Attributables::Module(_) | Attributables::TypeRef(_) | Attributables::SliceFile(_) => {
                report_unexpected_attribute(self, span, None, reporter);
            }
            Attributables::Parameter(_) => {
                let note = "parameters cannot be individually deprecated";
                report_unexpected_attribute(self, span, Some(note), reporter);
            }
            _ => {}
        }
    }
}

implement_attribute_kind_for!(Deprecated, "deprecated", false);
