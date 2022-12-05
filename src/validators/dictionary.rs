// Copyright (c) ZeroC, Inc. All rights reserved.

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
        ErrorBuilder::new(ErrorKind::KeyMustBeNonOptional)
            .span(type_ref.span())
            .report(diagnostic_reporter);
        return false;
    }

    let definition = type_ref.definition();
    let (is_valid, named_symbol): (bool, Option<&dyn NamedSymbol>) = match definition.concrete_type() {
        Types::Struct(struct_def) => {
            // Only compact structs can be used for dictionary keys.
            if !struct_def.is_compact {
                ErrorBuilder::new(ErrorKind::StructKeyMustBeCompact)
                    .span(type_ref.span())
                    .note(
                        format!("struct '{}' is defined here:", struct_def.identifier()),
                        Some(struct_def.span()),
                    )
                    .report(diagnostic_reporter);
                return false;
            }

            // Check that all the data members of the struct are also valid key types.
            let mut contains_invalid_key_types = false;
            for member in struct_def.members() {
                if !check_dictionary_key_type(member.data_type(), diagnostic_reporter) {
                    ErrorBuilder::new(ErrorKind::KeyTypeNotSupported(member.identifier().to_owned()))
                        .span(member.span())
                        .report(diagnostic_reporter);
                    contains_invalid_key_types = true;
                }
            }

            if contains_invalid_key_types {
                ErrorBuilder::new(ErrorKind::StructKeyContainsDisallowedType(
                    struct_def.identifier().to_owned(),
                ))
                .span(type_ref.span())
                .note(
                    format!("struct '{}' is defined here:", struct_def.identifier()),
                    Some(struct_def.span()),
                )
                .report(diagnostic_reporter);
                return false;
            }
            return true;
        }
        Types::Class(class_def) => (false, Some(class_def)),
        Types::Exception(exception_def) => (false, Some(exception_def)),
        Types::Interface(interface_def) => (false, Some(interface_def)),
        Types::Enum(_) => (true, None),
        Types::CustomType(_) => (true, None),
        Types::Sequence(_) => (false, None),
        Types::Dictionary(_) => (false, None),
        Types::Primitive(primitive) => (
            !matches!(primitive, Primitive::Float32 | Primitive::Float64 | Primitive::AnyClass),
            None,
        ),
    };

    if !is_valid {
        let pluralized_kind = match definition.concrete_type() {
            Types::Primitive(_) => definition.kind().to_owned(),
            Types::Class(_) => "classes".to_owned(),
            Types::Dictionary(_) => "dictionaries".to_owned(),
            _ => definition.kind().to_owned() + "s",
        };

        let builder = ErrorBuilder::new(ErrorKind::KeyTypeNotSupported(pluralized_kind)).span(type_ref.span());

        // If the key type is a user-defined type, point to where it was defined.
        let error = if let Some(named_symbol_def) = named_symbol {
            builder
                .note(
                    format!(
                        "{} '{}' is defined here:",
                        named_symbol_def.kind(),
                        named_symbol_def.identifier(),
                    ),
                    Some(named_symbol_def.span()),
                )
                .build()
        } else {
            builder.build()
        };
        diagnostic_reporter.report(error);
    }
    is_valid
}
