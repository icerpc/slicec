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
            AttributeKind::Single { directive, .. } => directive,
            AttributeKind::Multiple { directive, .. } => directive,
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
    Single {
        directive: String,
        argument: Option<String>,
    },
    Multiple {
        directive: String,
        arguments: Option<Vec<String>>,
    },
}

impl From<(&mut &mut DiagnosticReporter, &String, &Option<Vec<String>>, &Span)> for AttributeKind {
    fn from(
        (reporter, directive, arguments, span): (&mut &mut DiagnosticReporter, &String, &Option<Vec<String>>, &Span),
    ) -> Self {
        // Check for known attributes, if a parsing error occurs return AttributeKind::Multiple.
        let unknown_attribute = AttributeKind::Multiple {
            directive: directive.to_owned(),
            arguments: arguments.to_owned(),
        };

        let attribute_kind: Option<AttributeKind> = match directive.as_str() {
            COMPRESS => {
                // Check for invalid arguments.
                let valid_options = ["Args", "Return"];
                let default = vec![];
                let invalid_args = arguments
                    .as_ref()
                    .unwrap_or(&default)
                    .iter()
                    .filter(|arg| !valid_options.contains(&arg.as_str()))
                    .collect::<Vec<&String>>();
                invalid_args.iter().for_each(|arg| {
                    reporter.report_error(Error::new_with_notes(
                        ErrorKind::ArgumentNotSupported(arg.to_string(), "compress attribute".to_owned()),
                        Some(span),
                        vec![Note::new(
                            format!(
                                "The valid argument(s) for the compress attribute are {}",
                                message_value_separator(&valid_options).as_str(),
                            ),
                            Some(span),
                        )],
                    ));
                });

                if !invalid_args.is_empty() {
                    return unknown_attribute;
                };

                Some(AttributeKind::Compress {
                    compress_args: arguments
                        .as_ref()
                        .map(|args| args.contains(&"Args".to_owned()))
                        .unwrap_or(false),
                    compress_return: arguments
                        .as_ref()
                        .map(|args| args.contains(&"Return".to_owned()))
                        .unwrap_or(false),
                })
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
                            format!(
                                "The valid arguments for the format attribute are {}",
                                message_value_separator(&["Compact", "Sliced"])
                            ),
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
        attribute_kind.unwrap_or_else(|| {
            if let Some(args) = &arguments {
                if args.len() > 1 {
                    AttributeKind::Multiple {
                        directive: directive.to_owned(),
                        arguments: arguments.to_owned(),
                    }
                } else {
                    AttributeKind::Single {
                        directive: directive.to_owned(),
                        argument: args.get(0).map(|arg| arg.to_owned()),
                    }
                }
            } else {
                AttributeKind::Single {
                    directive: directive.to_owned(),
                    argument: None,
                }
            }
        })
    }
}

implement_Element_for!(Attribute, "attribute");
implement_Symbol_for!(Attribute);

/// Helper
fn message_value_separator(valid_strings: &[&str]) -> String {
    let separator = match valid_strings.len() {
        0 | 1 => "",
        2 => " and ",
        _ => ", ",
    };
    let mut backtick_strings = valid_strings
        .iter()
        .map(|arg| "`".to_owned() + arg + "`")
        .collect::<Vec<_>>();
    match valid_strings.len() {
        0 | 1 | 2 => backtick_strings.join(separator),
        _ => {
            let last = backtick_strings.pop().unwrap();
            backtick_strings.join(separator) + ", and " + &last
        }
    }
}
