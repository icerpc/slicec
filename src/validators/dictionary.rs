// Copyright (c) ZeroC, Inc.

use crate::diagnostics::*;
use crate::grammar::*;
use crate::validators::{ValidationChain, Validator};

pub fn dictionary_validators() -> ValidationChain {
    vec![Validator::Dictionaries(has_allowed_key_type)]
}

pub fn has_allowed_key_type(dictionaries: &[&Dictionary], diagnostic_reporter: &mut DiagnosticReporter) {
    for dictionary in dictionaries {
        check_dictionary_key_type(&dictionary.key_type, diagnostic_reporter);
    }
}

fn check_dictionary_key_type(type_ref: &TypeRef, diagnostic_reporter: &mut DiagnosticReporter) -> bool {
    // Optional types cannot be used as dictionary keys.
    if type_ref.is_optional {
        Error::new(ErrorKind::KeyMustBeNonOptional)
            .set_span(type_ref.span())
            .report(diagnostic_reporter);
        return false;
    }

    let definition = type_ref.definition();
    let is_valid: bool = match definition.concrete_type() {
        Types::Struct(struct_def) => {
            // Only compact structs can be used for dictionary keys.
            if !struct_def.is_compact {
                Error::new(ErrorKind::StructKeyMustBeCompact)
                    .set_span(type_ref.span())
                    .add_note(
                        format!("struct '{}' is defined here:", struct_def.identifier()),
                        Some(struct_def.span()),
                    )
                    .report(diagnostic_reporter);
                return false;
            }

            // Check that all the data members of the struct are also valid key types.
            let mut contains_invalid_key_types = false;
            for member in struct_def.members() {
                let identifier = member.identifier().to_owned();
                if !check_dictionary_key_type(member.data_type(), diagnostic_reporter) {
                    Error::new(ErrorKind::KeyTypeNotSupported {
                        kind: format!("'{identifier}'"),
                    })
                    .set_span(member.span())
                    .report(diagnostic_reporter);
                    contains_invalid_key_types = true;
                }
            }

            if contains_invalid_key_types {
                Error::new(ErrorKind::StructKeyContainsDisallowedType {
                    struct_identifier: struct_def.identifier().to_owned(),
                })
                .set_span(type_ref.span())
                .add_note(
                    format!("struct '{}' is defined here:", struct_def.identifier()),
                    Some(struct_def.span()),
                )
                .report(diagnostic_reporter);
                return false;
            }
            return true;
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
        let kind = definition.kind().to_owned();
        let formatted_kind = match definition.concrete_type() {
            Types::Class(c) => format!("{} '{}'", c.kind(), c.identifier()),
            Types::Exception(e) => format!("{} '{}'", e.kind(), e.identifier()),
            Types::Interface(i) => format!("{} '{}'", i.kind(), i.identifier()),
            _ => kind,
        };
        Error::new(ErrorKind::KeyTypeNotSupported { kind: formatted_kind })
            .set_span(type_ref.span())
            .report(diagnostic_reporter);
    }
    is_valid
}
