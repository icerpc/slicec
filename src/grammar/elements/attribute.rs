// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::diagnostics::{DiagnosticReporter, Error, ErrorKind, Note};
use crate::grammar::attributes::*;
use crate::slice_file::Span;
use std::str::FromStr;

#[derive(Clone, Debug)]
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
            AttributeKind::Deprecated { .. } => DEPRECATED,
            AttributeKind::Compress { .. } => COMPRESS,
            AttributeKind::Format { .. } => FORMAT,
            AttributeKind::IgnoreWarnings { .. } => IGNORE_WARNINGS,
            AttributeKind::SingleArgument { directive, .. } => directive,
            AttributeKind::MultipleArguments { directive, .. } => directive,
            AttributeKind::NoArgument { directive } => directive,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttributeKind {
    Deprecated {
        reason: Option<String>,
    },
    Compress {
        compress_args: bool,
        compress_return: bool,
    },
    Format {
        format: ClassFormat,
    },
    IgnoreWarnings {
        warning_codes: Option<Vec<String>>,
    },

    // The following are used for attributes that are not recognized by the compiler. They may be language mapping
    // specific attributes that will be handled by the respective language mapping.
    /// An attribute with a no arguments.
    NoArgument {
        directive: String,
    },

    /// An attribute with a single argument.
    SingleArgument {
        directive: String,
        argument: String,
    },

    /// An attribute with multiple arguments.
    MultipleArguments {
        directive: String,
        arguments: Vec<String>,
    },
}

impl AttributeKind {
    pub fn new(reporter: &mut DiagnosticReporter, directive: &String, arguments: &Vec<String>, span: &Span) -> Self {
        // Check for known attributes, if a parsing error occurs return an unknown attribute.
        let unknown_attribute = match arguments.len() {
            0 => AttributeKind::NoArgument {
                directive: directive.to_owned(),
            },
            1 => AttributeKind::SingleArgument {
                directive: directive.to_owned(),
                argument: arguments[0].to_owned(),
            },
            _ => AttributeKind::MultipleArguments {
                directive: directive.to_owned(),
                arguments: arguments.to_owned(),
            },
        };

        let attribute_kind: Option<AttributeKind> = match directive.as_str() {
            COMPRESS => {
                let valid_options = ["Args", "Return"];
                if !arguments.is_empty() {
                    let invalid_arguments = arguments
                        .iter()
                        .filter(|arg| !valid_options.contains(&arg.as_str()))
                        .collect::<Vec<&String>>();
                    match invalid_arguments[..] {
                        [] => Some(AttributeKind::Compress {
                            compress_args: arguments.contains(&"Args".to_owned()),
                            compress_return: arguments.contains(&"Return".to_owned()),
                        }),
                        _ => {
                            for arg in invalid_arguments.iter() {
                                reporter.report_error(Error::new_with_notes(
                                    ErrorKind::ArgumentNotSupported(arg.to_string(), "compress attribute".to_owned()),
                                    Some(span),
                                    vec![Note::new(
                                        "The valid argument(s) for the compress attribute are `Args` and `Return`",
                                        Some(span),
                                    )],
                                ))
                            }
                            return unknown_attribute;
                        }
                    }
                } else {
                    Some(AttributeKind::Compress {
                        compress_args: false,
                        compress_return: false,
                    })
                }
            }
            DEPRECATED => Some(AttributeKind::Deprecated {
                reason: arguments.get(0).map(|arg| arg.to_owned()),
            }),
            FORMAT => {
                // Check that the format attribute has arguments
                if arguments.is_empty() {
                    reporter.report_error(Error::new(
                        ErrorKind::CannotBeEmpty("format attribute".to_owned()),
                        Some(span),
                    ));
                    return unknown_attribute;
                }

                // Check if the arguments are valid
                let invalid_args = arguments
                    .iter()
                    .filter(|arg| ClassFormat::from_str(arg).is_err())
                    .collect::<Vec<&String>>();
                invalid_args.iter().for_each(|arg| {
                    reporter.report_error(Error::new_with_notes(
                        ErrorKind::ArgumentNotSupported(arg.to_string(), "format attribute".to_owned()),
                        Some(span),
                        vec![Note::new(
                            "The valid arguments for the format attribute are `Compact` and `Sliced`",
                            Some(span),
                        )],
                    ));
                });
                if !invalid_args.is_empty() {
                    return unknown_attribute;
                };

                // Safe unwrap since args.len() > 0 and we checked that all the arguments are valid
                Some(AttributeKind::Format {
                    format: ClassFormat::from_str(&arguments[0]).unwrap(),
                })
            }
            IGNORE_WARNINGS => Some(AttributeKind::IgnoreWarnings {
                warning_codes: Some(arguments.to_owned()),
            }),
            _ => None,
        };

        // If the attribute is not known, return check if it is a single or multiple arguments
        attribute_kind.unwrap_or(unknown_attribute)
    }
}

implement_Element_for!(Attribute, "attribute");
implement_Symbol_for!(Attribute);
