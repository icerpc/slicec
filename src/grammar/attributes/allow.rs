// Copyright (c) ZeroC, Inc.

use super::super::Attributables;
use super::*;
use crate::diagnostics::{Diagnostic, DiagnosticReporter, Error};

#[derive(Debug)]
pub struct Allow {
    pub allowed_warnings: Vec<String>,
}

impl Allow {
    pub fn parse_from(Unparsed { directive, args }: &Unparsed, span: &Span, reporter: &mut DiagnosticReporter) -> Self {
        debug_assert_eq!(directive, Self::directive());

        // Check that the attribute has arguments.
        if args.is_empty() {
            Diagnostic::new(Error::MissingRequiredArgument {
                argument: r#"allow(<arguments>)"#.to_owned(),
            })
            .set_span(span)
            .report(reporter);
        }

        // Check that each of the arguments are valid.
        validate_allow_arguments(args, Some(span), reporter);

        let allowed_warnings = args.clone();
        Allow { allowed_warnings }
    }

    pub fn validate_on(&self, applied_on: Attributables, span: &Span, reporter: &mut DiagnosticReporter) {
        if matches!(applied_on, Attributables::Module(_) | Attributables::TypeRef(_)) {
            report_unexpected_attribute(self, span, None, reporter);
        }
    }
}

implement_attribute_kind_for!(Allow, "allow", true);

// This is a standalone function because it's used by both the `allow` attribute, and the `--allow` CLI option.
pub fn validate_allow_arguments(arguments: &[String], span: Option<&Span>, reporter: &mut DiagnosticReporter) {
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

            error.report(reporter);
        }
    }
}
