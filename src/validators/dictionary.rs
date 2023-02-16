// Copyright (c) ZeroC, Inc.

use crate::diagnostics::*;
use crate::grammar::*;
use crate::validators::{ValidationChain, Validator};

pub fn dictionary_validators() -> ValidationChain {
    vec![
        Validator::Dictionaries(has_allowed_key_type),
        Validator::Dictionaries(has_allowed_value_type),
    ]
}

pub fn has_allowed_key_type(dictionaries: &[&Dictionary], diagnostic_reporter: &mut DiagnosticReporter) {
    for dictionary in dictionaries {
        if let Some(e) = check_dictionary_key_type(&dictionary.key_type) {
            e.report(diagnostic_reporter)
        }
    }
}

pub fn has_allowed_value_type(dictionaries: &[&Dictionary], diagnostic_reporter: &mut DiagnosticReporter) {
    for dictionary in dictionaries {
        check_dictionary_value_type(&dictionary.value_type, diagnostic_reporter);
    }
}

fn check_dictionary_key_type(type_ref: &TypeRef) -> Option<Error> {
    // Optional types cannot be used as dictionary keys.
    if type_ref.is_optional {
        return Some(Error::new(ErrorKind::KeyMustBeNonOptional).set_span(type_ref.span()));
    }

    let definition = type_ref.definition();
    let is_valid = match definition.concrete_type() {
        Types::Struct(struct_def) => {
            // Only compact structs can be used for dictionary keys.
            if !struct_def.is_compact {
                return Some(
                    Error::new(ErrorKind::StructKeyMustBeCompact)
                        .set_span(type_ref.span())
                        .add_note(
                            format!("struct '{}' is defined here:", struct_def.identifier()),
                            Some(struct_def.span()),
                        ),
                );
            }

            // Check that all the data members of the struct are also valid key types. We collect the invalid members
            // so we can report them in the error message.
            let errors = struct_def
                .members()
                .into_iter()
                .filter_map(|member| check_dictionary_key_type(member.data_type()))
                .collect::<Vec<_>>();
            if !errors.is_empty() {
                let mut error = Error::new(ErrorKind::StructKeyContainsDisallowedType {
                    struct_identifier: struct_def.identifier().to_owned(),
                })
                .set_span(type_ref.span());

                // Convert each error into a note and add it to the struct key error.
                for e in errors {
                    error = error.add_note(e.to_string(), e.span());
                }
                return Some(error);
            }
            true
        }
        Types::Class(_) => false,
        Types::Exception(_) => false,
        Types::Interface(_) => false,
        Types::Enum(_) => true,
        Types::CustomType(_) => true,
        Types::Sequence(_) => false,
        Types::Dictionary(_) => false,
        Types::Primitive(primitive) => {
            primitive.is_integral() || matches!(primitive, Primitive::Bool | Primitive::String)
        }
    };

    if !is_valid {
        return Some(
            Error::new(ErrorKind::KeyTypeNotSupported {
                kind: formatted_kind(definition),
            })
            .set_span(type_ref.span()),
        );
    }
    None
}

fn formatted_kind(definition: &dyn Type) -> String {
    let kind = definition.kind();
    match definition.concrete_type() {
        Types::Class(c) => format!("{} '{}'", c.kind(), c.identifier()),
        Types::Exception(e) => format!("{} '{}'", e.kind(), e.identifier()),
        Types::Interface(i) => format!("{} '{}'", i.kind(), i.identifier()),
        _ => kind.to_owned(),
    }
}

fn check_dictionary_value_type(type_ref: &TypeRef, diagnostic_reporter: &mut DiagnosticReporter) {
    let definition = type_ref.definition();
    match definition.concrete_type() {
        Types::Sequence(s) => {
            if let Types::Dictionary(dictionary) = s.element_type.concrete_type() {
                if let Some(e) = check_dictionary_key_type(&dictionary.key_type) {
                    e.report(diagnostic_reporter)
                };
                check_dictionary_value_type(&dictionary.value_type, diagnostic_reporter);
            }
        }
        Types::Dictionary(dictionary) => {
            if let Some(e) = check_dictionary_key_type(&dictionary.key_type) {
                e.report(diagnostic_reporter)
            };
            check_dictionary_value_type(&dictionary.value_type, diagnostic_reporter);
        }
        _ => (),
    }
}
