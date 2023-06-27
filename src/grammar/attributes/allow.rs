// Copyright (c) ZeroC, Inc.

use super::*;

#[derive(Debug)]
pub struct Allow {
    pub allowed_warnings: Vec<String>,
}

impl Allow {
    pub fn parse_from(Unparsed { directive, args }: &Unparsed, span: &Span, reporter: &mut DiagnosticReporter) -> Self {
        debug_assert_eq!(directive, Self::directive());

        check_that_arguments_were_provided(args, Self::directive(), span, reporter);

        for arg in args {
            let mut is_valid = Warning::ALLOWABLE_WARNING_IDENTIFIERS.contains(&arg.as_str());

            // The `DuplicateFile` lint can't be configured by attributes because it's a command-line specific warning.
            if arg == "DuplicateFile" {
                is_valid = false;
            }

            // Emit an error if the argument wasn't valid.
            if !is_valid {
                // TODO we should emit a link to the warnings page when we write it!
                let mut error = Diagnostic::new(Error::ArgumentNotSupported {
                    argument: arg.to_owned(),
                    directive: "allow".to_owned(),
                })
                .set_span(span);

                // Check if the argument only differs in case from a valid one.
                let suggestion = Warning::ALLOWABLE_WARNING_IDENTIFIERS
                    .iter()
                    .find(|identifier| identifier.eq_ignore_ascii_case(arg));
                if let Some(identifier) = suggestion {
                    let message = format!("attribute arguments are case sensitive, perhaps you meant '{identifier}'?");
                    error = error.add_note(message, None);
                }

                error.report(reporter);
            }
        }

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
