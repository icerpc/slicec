// Copyright (c) ZeroC, Inc.

use super::*;
use super::super::Attributables;
use crate::diagnostics::{Diagnostic, DiagnosticReporter, Error};

#[derive(Debug)]
pub struct Allow {
    pub allowed_warnings: Vec<String>,
}

impl Allow {
    pub fn parse_from(Unparsed { directive, args }: Unparsed, span: &Span, diagnostics: &mut Vec<Diagnostic>) -> Self {
        debug_assert_eq!(directive, Self::directive());

        // Check that the attribute has arguments.
        if args.is_empty() {
            let diagnostic = Diagnostic::new(Error::MissingRequiredArgument {
                argument: r#"allow(<arguments>)"#.to_owned(),
            })
            .set_span(span);
            diagnostics.push(diagnostic);
        }

        // Check that each of the arguments are valid.
        validate_allow_arguments(&args, Some(span), diagnostics);

        Allow { allowed_warnings: args }
    }

    pub fn validate_on(&self, applied_on: Attributables, span: &Span, reporter: &mut DiagnosticReporter) {
        if matches!(applied_on, Attributables::Module(_) | Attributables::TypeRef(_)) {
            report_unexpected_attribute::<Self>(span, None, reporter);
        }
    }
}

implement_attribute_kind_for!(Allow, "allow", true);

