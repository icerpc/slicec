// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::diagnostics::{DiagnosticReporter, Error, ErrorKind, Warning};
use crate::slice_file::Span;
use std::str::FromStr;

const COMPRESS: &str = "compress";
const COMPRESS_ARGS: [&str; 2] = ["Args", "Return"]; // The valid arguments for the `compress` attribute.
const DEPRECATED: &str = "deprecated";
const FORMAT: &str = "format";
const IGNORE_WARNINGS: &str = "ignoreWarnings";
const ONEWAY: &str = "oneway";

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
            AttributeKind::ClassFormat { .. } => FORMAT,
            AttributeKind::IgnoreWarnings { .. } => IGNORE_WARNINGS,
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

    pub fn match_ignore_warnings(attribute: &Attribute) -> Option<Option<Vec<String>>> {
        match &attribute.kind {
            AttributeKind::IgnoreWarnings { warning_codes } => Some(warning_codes.clone()),
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

#[derive(Clone, Debug)]
pub enum AttributeKind {
    Deprecated { reason: Option<String> },
    Compress { compress_args: bool, compress_return: bool },
    ClassFormat { format: ClassFormat },
    IgnoreWarnings { warning_codes: Option<Vec<String>> },
    Oneway,

    // The following are used for attributes that are not recognized by the compiler. They may be language mapping
    // specific attributes that will be handled by the respective language mapping.
    LanguageKind { kind: Box<dyn LanguageKind> },

    Other { directive: String, arguments: Vec<String> },
}

pub trait LanguageKind {
    fn directive(&self) -> &str;
    fn as_any(&self) -> &dyn std::any::Any;
    fn clone_kind(&self) -> Box<dyn LanguageKind>;
    fn debug_kind(&self) -> &str;
}

impl Clone for Box<dyn LanguageKind> {
    fn clone(&self) -> Self {
        self.clone_kind()
    }
}

impl std::fmt::Debug for Box<dyn LanguageKind> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.debug_kind())
    }
}

impl AttributeKind {
    pub fn new(reporter: &mut DiagnosticReporter, directive: &String, arguments: &[String], span: &Span) -> Self {
        // Check for known attributes, if a parsing error occurs return an unknown attribute.
        let unmatched_attribute = AttributeKind::Other {
            directive: directive.to_owned(),
            arguments: arguments.to_owned(),
        };

        let attribute_kind: Option<AttributeKind> = match directive.as_str() {
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
                                Error::new(ErrorKind::ArgumentNotSupported {
                                    argument_name: arg.to_string(),
                                    method_name: "compress attribute".to_owned(),
                                })
                                .set_span(span)
                                .add_note(
                                    "The valid arguments for the compress attribute are `Args` and `Return`",
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

            ONEWAY => match arguments {
                [] => Some(AttributeKind::Oneway),
                _ => {
                    Error::new(ErrorKind::TooManyArguments {
                        expected: ONEWAY.to_owned(),
                    })
                    .set_span(span)
                    .add_note("The oneway attribute does not take any arguments", Some(span))
                    .report(reporter);
                    return unmatched_attribute;
                }
            },

            DEPRECATED => match arguments {
                [] => Some(AttributeKind::Deprecated { reason: None }),
                [reason] => Some(AttributeKind::Deprecated {
                    reason: Some(reason.to_owned()),
                }),
                [..] => {
                    Error::new(ErrorKind::TooManyArguments {
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
                    Error::new(ErrorKind::CannotBeEmpty {
                        member_identifier: "format attribute".to_owned(),
                    })
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
                    Error::new(ErrorKind::ArgumentNotSupported {
                        argument_name: arg.to_string(),
                        method_name: "format attribute".to_owned(),
                    })
                    .set_span(span)
                    .add_note(
                        "The valid arguments for the format attribute are `Compact` and `Sliced`",
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

            IGNORE_WARNINGS => {
                for arg in arguments {
                    if !Warning::all_codes().contains(&arg.as_str()) {
                        // No exact match was found, check if the casing did not match
                        let uppercase = arg.to_uppercase();
                        if Warning::all_codes().contains(&uppercase.as_str()) {
                            // The casing did not match, report an error with a note
                            Error::new(ErrorKind::InvalidWarningCode { code: arg.to_owned() })
                                .set_span(span)
                                .add_note(
                                    format!("The warning code is case sensitive, did you mean to use '{uppercase}'?"),
                                    Some(span),
                                )
                                .report(reporter);
                        } else {
                            // No exact match and no casing match, report an error
                            Error::new(ErrorKind::InvalidWarningCode { code: arg.to_owned() })
                                .set_span(span)
                                .report(reporter);
                        }
                    }
                }
                Some(AttributeKind::IgnoreWarnings {
                    warning_codes: Some(arguments.to_owned()),
                })
            }

            _ => None,
        };

        // If the attribute is not known, return check if it is a single or multiple arguments
        attribute_kind.unwrap_or(unmatched_attribute)
    }
}

implement_Element_for!(Attribute, "attribute");
implement_Symbol_for!(Attribute);
