// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::diagnostics::{Diagnostic, DiagnosticReporter, Error, Warning};
use crate::slice_file::Span;
use std::str::FromStr;

const ALLOW: &str = "allow";
const COMPRESS: &str = "compress";
const COMPRESS_ARGS: [&str; 2] = ["Args", "Return"]; // The valid arguments for the `compress` attribute.
const DEPRECATED: &str = "deprecated";
const FORMAT: &str = "format";
const ONEWAY: &str = "oneway";

#[derive(Debug)]
pub struct Attribute {
    pub kind: AttributeKind,
    pub span: Span,
}

impl Attribute {
    pub fn new(reporter: &mut DiagnosticReporter, directive: &String, arguments: Vec<String>, span: Span) -> Self {
        let kind = AttributeKind::new(reporter, directive, &arguments, &span);
        Self { kind, span }
    }

    pub fn directive(&self) -> &str {
        match &self.kind {
            AttributeKind::Allow { .. } => ALLOW,
            AttributeKind::ClassFormat { .. } => FORMAT,
            AttributeKind::Compress { .. } => COMPRESS,
            AttributeKind::Deprecated { .. } => DEPRECATED,
            AttributeKind::Oneway { .. } => ONEWAY,
            AttributeKind::LanguageKind { kind } => kind.directive(),
            AttributeKind::Other { directive, .. } => directive,
        }
    }

    pub fn match_deprecated(attribute: &Attribute) -> Option<Option<String>> {
        match &attribute.kind {
            AttributeKind::Deprecated { reason } => Some(reason.clone()),
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

    pub fn match_class_format(attribute: &Attribute) -> Option<ClassFormat> {
        match &attribute.kind {
            AttributeKind::ClassFormat { format } => Some(format.clone()),
            _ => None,
        }
    }

    pub fn match_allow_warnings(attribute: &Attribute) -> Option<&Vec<String>> {
        match &attribute.kind {
            AttributeKind::Allow { allowed_warnings } => Some(allowed_warnings),
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
    ClassFormat { format: ClassFormat },
    Compress { compress_args: bool, compress_return: bool },
    Deprecated { reason: Option<String> },
    Oneway,

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
    pub fn new(reporter: &mut DiagnosticReporter, directive: &String, arguments: &[String], span: &Span) -> Self {
        // Check for known attributes, if a parsing error occurs return an unknown attribute.
        let unmatched_attribute = AttributeKind::Other {
            directive: directive.to_owned(),
            arguments: arguments.to_owned(),
        };

        let attribute_kind: Option<AttributeKind> = match directive.as_str() {
            ALLOW => {
                // Check that the `allow` attribute has arguments.
                if arguments.is_empty() {
                    Diagnostic::new(Error::MissingRequiredArgument {
                        argument: r#"allow(<arguments>)"#.to_owned(),
                    })
                    .set_span(span)
                    .report(reporter);
                }
                validate_allow_arguments(arguments, reporter);

                Some(AttributeKind::Allow {
                    allowed_warnings: arguments.to_owned(),
                })
            }

            COMPRESS => {
                if !arguments.is_empty() {
                    let invalid_arguments = arguments
                        .iter()
                        .filter(|arg| !COMPRESS_ARGS.contains(&arg.as_str()))
                        .collect::<Vec<&String>>();
                    match invalid_arguments[..] {
                        [] => Some(AttributeKind::Compress {
                            compress_args: arguments.contains(&"Args".to_owned()),
                            compress_return: arguments.contains(&"Return".to_owned()),
                        }),
                        _ => {
                            for arg in invalid_arguments.iter() {
                                Diagnostic::new(Error::ArgumentNotSupported {
                                    argument: arg.to_string(),
                                    directive: "compress".to_owned(),
                                })
                                .set_span(span)
                                .add_note(
                                    "The valid arguments for the compress attribute are 'Args' and 'Return'",
                                    Some(span),
                                )
                                .report(reporter)
                            }
                            return unmatched_attribute;
                        }
                    }
                } else {
                    Some(AttributeKind::Compress {
                        compress_args: false,
                        compress_return: false,
                    })
                }
            }

            DEPRECATED => match arguments {
                [] => Some(AttributeKind::Deprecated { reason: None }),
                [reason] => Some(AttributeKind::Deprecated {
                    reason: Some(reason.to_owned()),
                }),
                [..] => {
                    Diagnostic::new(Error::TooManyArguments {
                        expected: DEPRECATED.to_owned(),
                    })
                    .set_span(span)
                    .add_note("The deprecated attribute takes at most one argument", Some(span))
                    .report(reporter);
                    return unmatched_attribute;
                }
            },

            FORMAT => {
                // Check that the format attribute has arguments
                if arguments.is_empty() {
                    Diagnostic::new(Error::MissingRequiredArgument {
                        argument: r#"format(<arguments>)"#.to_owned(),
                    })
                    .add_note(
                        "The valid arguments for the format attribute are 'Compact' and 'Sliced'",
                        None,
                    )
                    .set_span(span)
                    .report(reporter);
                    return unmatched_attribute;
                }

                // Check if the arguments are valid
                let invalid_args = arguments
                    .iter()
                    .filter(|arg| ClassFormat::from_str(arg).is_err())
                    .collect::<Vec<&String>>();
                invalid_args.iter().for_each(|arg| {
                    Diagnostic::new(Error::ArgumentNotSupported {
                        argument: arg.to_string(),
                        directive: "format".to_owned(),
                    })
                    .set_span(span)
                    .add_note(
                        "The valid arguments for the format attribute are 'Compact' and 'Sliced'",
                        Some(span),
                    )
                    .report(reporter);
                });
                if !invalid_args.is_empty() {
                    return unmatched_attribute;
                };

                // Safe unwrap since args.len() > 0 and we checked that all the arguments are valid
                Some(AttributeKind::ClassFormat {
                    format: ClassFormat::from_str(&arguments[0]).unwrap(),
                })
            }

            ONEWAY => match arguments {
                [] => Some(AttributeKind::Oneway),
                _ => {
                    Diagnostic::new(Error::TooManyArguments {
                        expected: ONEWAY.to_owned(),
                    })
                    .set_span(span)
                    .add_note("The oneway attribute does not take any arguments", Some(span))
                    .report(reporter);
                    return unmatched_attribute;
                }
            },

            _ => None,
        };

        // If the attribute is not known, return check if it is a single or multiple arguments
        attribute_kind.unwrap_or(unmatched_attribute)
    }

    pub fn is_repeatable(&self) -> bool {
        match &self {
            AttributeKind::Allow { .. } => true,
            AttributeKind::ClassFormat { .. } => false,
            AttributeKind::Compress { .. } => false,
            AttributeKind::Deprecated { .. } => false,
            AttributeKind::Oneway => false,
            AttributeKind::LanguageKind { kind } => kind.is_repeatable(),
            AttributeKind::Other { .. } => true,
        }
    }
}

implement_Element_for!(Attribute, "attribute");
implement_Symbol_for!(Attribute);

// This is a standalone function because it's used by both the `allow` attribute, and the `--allow-warning` CLI option.
pub fn validate_allow_arguments(arguments: &[String], diagnostic_reporter: &mut DiagnosticReporter) {
    todo!()
}
