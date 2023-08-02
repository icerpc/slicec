// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{Diagnostic, Diagnostics, Error};
use crate::grammar::*;

pub fn validate_dictionary(dictionary: &Dictionary, diagnostics: &mut Diagnostics) {
    has_allowed_key_type(dictionary, diagnostics);
}

fn has_allowed_key_type(dictionary: &Dictionary, diagnostics: &mut Diagnostics) {
    if let Some(e) = check_dictionary_key_type(&dictionary.key_type) {
        e.push_into(diagnostics)
    }
}

fn check_dictionary_key_type(type_ref: &TypeRef) -> Option<Diagnostic> {
    // Optional types cannot be used as dictionary keys.
    if type_ref.is_optional {
        return Some(Diagnostic::new(Error::KeyMustBeNonOptional).set_span(type_ref.span()));
    }

    let definition = type_ref.definition();
    let is_valid = match definition.concrete_type() {
        Types::Struct(struct_def) => {
            // Only compact structs can be used for dictionary keys.
            if !struct_def.is_compact {
                return Some(Diagnostic::new(Error::StructKeyMustBeCompact).set_span(type_ref.span()));
            }

            // Check that all the fields of the struct are also valid key types.
            // We collect the invalid fields so we can report them in the error message.
            let errors = struct_def
                .fields()
                .into_iter()
                .filter_map(|field| check_dictionary_key_type(field.data_type()))
                .collect::<Vec<_>>();
            if !errors.is_empty() {
                let mut error = Diagnostic::new(Error::StructKeyContainsDisallowedType {
                    struct_identifier: struct_def.identifier().to_owned(),
                })
                .set_span(type_ref.span());

                // Convert each error into a note and add it to the struct key error.
                for e in errors {
                    error = error.add_note(e.message(), e.span());
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
            Diagnostic::new(Error::KeyTypeNotSupported {
                kind: formatted_kind(definition),
            })
            .set_span(type_ref.span()),
        );
    }
    None
}

fn formatted_kind(definition: &dyn Type) -> String {
    match definition.concrete_type() {
        Types::Class(c) => format!("{} '{}'", c.kind(), c.identifier()),
        Types::Exception(e) => format!("{} '{}'", e.kind(), e.identifier()),
        Types::Interface(i) => format!("{} '{}'", i.kind(), i.identifier()),
        _ => definition.kind().to_owned(),
    }
}
