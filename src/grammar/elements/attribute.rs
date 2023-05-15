// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::diagnostics::{Diagnostic, DiagnosticReporter, Error, Warning};
use crate::slice_file::Span;

const ALLOW: &str = "allow";
const COMPRESS: &str = "compress";
const DEPRECATED: &str = "deprecated";
const SLICED_FORMAT: &str = "slicedFormat";
const ONEWAY: &str = "oneway";

#[derive(Debug)]
pub struct Attribute {
    pub kind: AttributeKind,
    pub span: Span,
}

impl Attribute {
    pub fn new(reporter: &mut DiagnosticReporter, directive: String, arguments: Vec<String>, span: Span) -> Self {
        let kind = AttributeKind::new(reporter, directive, arguments, &span);
        Self { kind, span }
    }

    pub fn directive(&self) -> &str {
        match &self.kind {
            AttributeKind::Allow { .. } => ALLOW,
            AttributeKind::Compress { .. } => COMPRESS,
            AttributeKind::Deprecated { .. } => DEPRECATED,
            AttributeKind::SlicedFormat { .. } => SLICED_FORMAT,
            AttributeKind::Oneway { .. } => ONEWAY,
            AttributeKind::LanguageKind { kind } => kind.directive(),
            AttributeKind::Other { directive, .. } => directive,
        }
    }

    pub fn match_allow_warnings(attribute: &Attribute) -> Option<&Vec<String>> {
        match &attribute.kind {
            AttributeKind::Allow { allowed_warnings } => Some(allowed_warnings),
            _ => None,
        }
    }

    pub fn match_compress(attribute: &Attribute) -> Option<(bool, bool)> {
        match &attribute.kind {
            AttributeKind::Compress {
                compress_args,
                compress_return,
            } => Some((*compress_args, *compress_return)),
            _ => None,
        }
    }

    pub fn match_deprecated(attribute: &Attribute) -> Option<Option<String>> {
        match &attribute.kind {
            AttributeKind::Deprecated { reason } => Some(reason.clone()),
            _ => None,
        }
    }

    pub fn match_sliced_format(attribute: &Attribute) -> Option<(bool, bool)> {
        match &attribute.kind {
            AttributeKind::SlicedFormat {
                sliced_args,
                sliced_return,
            } => Some((*sliced_args, *sliced_return)),
            _ => None,
        }
    }

    pub fn match_oneway(attribute: &Attribute) -> Option<()> {
        match &attribute.kind {
            AttributeKind::Oneway => Some(()),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum AttributeKind {
    Allow { allowed_warnings: Vec<String> },
    Compress { compress_args: bool, compress_return: bool },
    Deprecated { reason: Option<String> },
    Oneway,
    SlicedFormat { sliced_args: bool, sliced_return: bool },

    // The following are used for attributes that are not recognized by the compiler. They may be language mapping
    // specific attributes that will be handled by the respective language mapping.
    LanguageKind { kind: Box<dyn LanguageKind> },
    Other { directive: String, arguments: Vec<String> },
}

pub trait LanguageKind: std::fmt::Debug {
    fn directive(&self) -> &str;
    fn as_any(&self) -> &dyn std::any::Any;
    fn is_repeatable(&self) -> bool;
}

impl AttributeKind {
    pub fn new(reporter: &mut DiagnosticReporter, directive: String, arguments: Vec<String>, span: &Span) -> Self {
        match directive.as_str() {
            ALLOW => {
                // Check that the attribute has arguments.
                if arguments.is_empty() {
                    Diagnostic::new(Error::MissingRequiredArgument {
                        argument: r#"allow(<arguments>)"#.to_owned(),
                    })
                    .set_span(span)
                    .report(reporter);
                }

                // Check that each of the arguments are valid.
                validate_allow_arguments(&arguments, Some(span), reporter);

                AttributeKind::Allow {
                    allowed_warnings: arguments,
                }
            }

            COMPRESS => {
                let (mut compress_args, mut compress_return) = (false, false);
                for arg in arguments {
                    match arg.as_str() {
                        "Args" => {
                            // TODO should we report a warning/error for duplicates?
                            compress_args = true;
                        }
                        "Return" => {
                            // TODO should we report a warning/error for duplicates?
                            compress_return = true;
                        }
                        _ => {
                            Diagnostic::new(Error::ArgumentNotSupported {
                                argument: arg,
                                directive: "compress".to_owned(),
                            })
                            .set_span(span)
                            .add_note("'Args' and 'Return' are the only valid arguments", None)
                            .report(reporter);
                        }
                    }
                }

                AttributeKind::Compress {
                    compress_args,
                    compress_return,
                }
            }

            DEPRECATED => {
                if arguments.len() > 1 {
                    Diagnostic::new(Error::TooManyArguments {
                        expected: DEPRECATED.to_owned(),
                    })
                    .set_span(span)
                    .add_note("The deprecated attribute takes at most one argument", Some(span))
                    .report(reporter);
                }

                AttributeKind::Deprecated {
                    reason: arguments.into_iter().next(),
                }
            }

            SLICED_FORMAT => {
                let (mut sliced_args, mut sliced_return) = (false, false);
                for arg in arguments {
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
                            Diagnostic::new(Error::ArgumentNotSupported {
                                argument: arg,
                                directive: "slicedFormat".to_owned(),
                            })
                            .set_span(span)
                            .add_note("'Args' and 'Return' are the only valid arguments", None)
                            .report(reporter);
                        }
                    }
                }

                AttributeKind::SlicedFormat {
                    sliced_args,
                    sliced_return,
                }
            }

            ONEWAY => {
                // Check that no arguments were provided to the attribute.
                if !arguments.is_empty() {
                    Diagnostic::new(Error::TooManyArguments {
                        expected: ONEWAY.to_owned(),
                    })
                    .set_span(span)
                    .add_note("The oneway attribute does not take any arguments", None)
                    .report(reporter);
                }

                AttributeKind::Oneway
            }

            _ => AttributeKind::Other { directive, arguments },
        }
    }

    pub fn is_repeatable(&self) -> bool {
        match &self {
            AttributeKind::Allow { .. } => true,
            AttributeKind::Compress { .. } => false,
            AttributeKind::Deprecated { .. } => false,
            AttributeKind::SlicedFormat { .. } => false,
            AttributeKind::Oneway => false,
            AttributeKind::LanguageKind { kind } => kind.is_repeatable(),
            AttributeKind::Other { .. } => true,
        }
    }
}

implement_Element_for!(Attribute, "attribute");
implement_Symbol_for!(Attribute);

// This is a standalone function because it's used by both the `allow` attribute, and the `--allow` CLI option.
pub fn validate_allow_arguments(
    arguments: &[String],
    span: Option<&Span>,
    diagnostic_reporter: &mut DiagnosticReporter,
) {
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

            error.report(diagnostic_reporter);
        }
    }
}
