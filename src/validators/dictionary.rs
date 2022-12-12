// Copyright (c) ZeroC, Inc. All rights reserved.

use super::ValidatorVisitor;
use crate::diagnostics::{Error, ErrorKind};
use crate::grammar::*;

impl ValidatorVisitor<'_> {
pub(super) fn has_allowed_key_type(&mut self, dictionaries: &[&Dictionary]) {
    for dictionary in dictionaries {
        self.check_dictionary_key_type(&dictionary.key_type);
    }
}

fn check_dictionary_key_type(&mut self, type_ref: &TypeRef) -> bool {
    // Optional types cannot be used as dictionary keys.
    if type_ref.is_optional {
        Error::new(ErrorKind::KeyMustBeNonOptional)
            .set_span(type_ref.span())
            .report(self.diagnostic_reporter);
        return false;
    }

    let definition = type_ref.definition();
    let (is_valid, named_symbol): (bool, Option<&dyn NamedSymbol>) = match definition.concrete_type() {
        Types::Struct(struct_def) => {
            // Only compact structs can be used for dictionary keys.
            if !struct_def.is_compact {
                Error::new(ErrorKind::StructKeyMustBeCompact)
                    .set_span(type_ref.span())
                    .add_note(
                        format!("struct '{}' is defined here:", struct_def.identifier()),
                        Some(struct_def.span()),
                    )
                    .report(self.diagnostic_reporter);
                return false;
            }

            // Check that all the data members of the struct are also valid key types.
            let mut contains_invalid_key_types = false;
            for member in struct_def.members() {
                if !self.check_dictionary_key_type(member.data_type()) {
                    Error::new(ErrorKind::KeyTypeNotSupported(member.identifier().to_owned()))
                        .set_span(member.span())
                        .report(self.diagnostic_reporter);
                    contains_invalid_key_types = true;
                }
            }

            if contains_invalid_key_types {
                Error::new(ErrorKind::StructKeyContainsDisallowedType(
                    struct_def.identifier().to_owned(),
                ))
                .set_span(type_ref.span())
                .add_note(
                    format!("struct '{}' is defined here:", struct_def.identifier()),
                    Some(struct_def.span()),
                )
                .report(self.diagnostic_reporter);
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

        let mut error = Error::new(ErrorKind::KeyTypeNotSupported(pluralized_kind)).set_span(type_ref.span());

        // If the key type is a user-defined type, point to where it was defined.
        if let Some(named_symbol_def) = named_symbol {
            error = error.add_note(
                format!(
                    "{} '{}' is defined here:",
                    named_symbol_def.kind(),
                    named_symbol_def.identifier(),
                ),
                Some(named_symbol_def.span()),
            )
        }
        error.report(self.diagnostic_reporter);
    }
    is_valid
}
}
