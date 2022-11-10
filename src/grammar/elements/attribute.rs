// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::diagnostics::{DiagnosticReporter, Error, ErrorKind, Note};
use crate::grammar::attributes::{COMPRESS, DEPRECATED, FORMAT, IGNORE_WARNINGS};
use crate::slice_file::Span;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct Attribute {
    pub kind: AttributeKind,
    pub span: Span,
}

impl Attribute {
    pub fn directive(&self) -> String {
        match &self.kind {
            AttributeKind::Deprecated { .. } => DEPRECATED,
            AttributeKind::Compress { .. } => COMPRESS,
            AttributeKind::Format { .. } => FORMAT,
            AttributeKind::IgnoreWarnings { .. } => IGNORE_WARNINGS,
            AttributeKind::SingleArgument { directive, .. } => directive,
            AttributeKind::MultipleArguments { directive, .. } => directive,
            AttributeKind::NoArgument { directive } => directive,
        }
        .to_owned()
    }
}

impl From<(&mut &mut DiagnosticReporter, &String, Option<Vec<String>>, Span)> for Attribute {
    fn from(
        (reporter, directive, arguments, span): (&mut &mut DiagnosticReporter, &String, Option<Vec<String>>, Span),
    ) -> Self {
        let kind = AttributeKind::from((reporter, directive, &arguments, &span));
        Self { kind, span }
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

impl From<(&mut &mut DiagnosticReporter, &String, &Option<Vec<String>>, &Span)> for AttributeKind {
    fn from(
        (reporter, directive, arguments, span): (&mut &mut DiagnosticReporter, &String, &Option<Vec<String>>, &Span),
    ) -> Self {
        // Check for known attributes, if a parsing error occurs return an unknown attribute.
        let unknown_attribute = match arguments {
            Some(args) => match args.len() {
                0 => AttributeKind::NoArgument {
                    directive: directive.to_owned(),
                },
                1 => AttributeKind::SingleArgument {
                    directive: directive.to_owned(),
                    argument: args[0].to_owned(),
                },
                _ => AttributeKind::MultipleArguments {
                    directive: directive.to_owned(),
                    arguments: args.to_owned(),
                },
            },
            _ => AttributeKind::NoArgument {
                directive: directive.to_owned(),
            },
        };

        let attribute_kind: Option<AttributeKind> = match directive.as_str() {
            COMPRESS => {
                let valid_options = ["Args", "Return"];
                match arguments {
                    Some(arguments) => {
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
                                        ErrorKind::ArgumentNotSupported(
                                            arg.to_string(),
                                            "compress attribute".to_owned(),
                                        ),
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
                    }
                    None => Some(AttributeKind::Compress {
                        compress_args: false,
                        compress_return: false,
                    }),
                }
            }
            DEPRECATED => Some(AttributeKind::Deprecated {
                reason: arguments.as_ref().map(|args| args[0].to_owned()),
            }),
            FORMAT => {
                // Check that the format attribute has arguments
                if arguments.is_none() || arguments.is_some() && arguments.as_ref().unwrap().is_empty() {
                    reporter.report_error(Error::new(
                        ErrorKind::CannotBeEmpty("format attribute".to_owned()),
                        Some(span),
                    ));
                    return unknown_attribute;
                }

                // Safe unwrap
                let args = arguments.as_ref().unwrap();

                // Check if the arguments are valid
                let invalid_args = args
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
                    format: ClassFormat::from_str(&args[0]).unwrap(),
                })
            }
            IGNORE_WARNINGS => Some(AttributeKind::IgnoreWarnings {
                warning_codes: arguments.to_owned(),
            }),
            _ => None,
        };

        // If the attribute is not known, return check if it is a single or multiple arguments
        attribute_kind.unwrap_or(unknown_attribute)
    }
}

implement_Element_for!(Attribute, "attribute");
implement_Symbol_for!(Attribute);
