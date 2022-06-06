// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::error::{Error, ErrorLevel};
use crate::grammar::*;
use crate::validators::{Validate, ValidationChain, ValidationResult};

pub fn dictionary_validators() -> ValidationChain {
    vec![Validate::Dictionary(has_allowed_key_type)]
}

pub fn has_allowed_key_type(dictionaries: &[&Dictionary]) -> ValidationResult {
    let mut errors = vec![];

    for dictionary in dictionaries {
        check_dictionary_key_type(&dictionary.key_type, &mut errors);
    }

    match errors.is_empty() {
        true => Ok(()),
        false => Err(errors),
    }
}

fn check_dictionary_key_type(type_ref: &TypeRef, errors: &mut Vec<Error>) -> bool {
    // Optional types cannot be used as dictionary keys.
    if type_ref.is_optional {
        errors.push(Error {
            message: "invalid dictionary key type: optional types cannot be used as a dictionary key type".into(),
            location: Some(type_ref.location.clone()),
            severity: ErrorLevel::Error,
        });
        return false;
    }

    let definition = type_ref.definition();
    let (is_valid, named_symbol): (bool, Option<&dyn NamedSymbol>) = match definition.concrete_type() {
        Types::Struct(struct_def) => {
            // Only compact structs can be used for dictionary keys.
            if !struct_def.is_compact {
                errors.push(Error {
                    message: "invalid dictionary key type: structs must be compact to be used as a dictionary key type"
                        .into(),
                    location: Some(type_ref.location.clone()),
                    severity: ErrorLevel::Error,
                });
                errors.push(Error {
                    message: format!("struct '{}' is defined here:", struct_def.identifier()),
                    location: Some(struct_def.location.clone()),
                    severity: ErrorLevel::Note,
                });
                return false;
            }

            // Check that all the data members of the struct are also valid key types.
            let mut contains_invalid_key_types = false;
            for member in struct_def.members() {
                if !check_dictionary_key_type(member.data_type(), errors) {
                    errors.push(Error {
                        message: format!(
                            "data member '{}' cannot be used as a dictionary key type",
                            member.identifier(),
                        ),
                        location: Some(member.location.clone()),
                        severity: ErrorLevel::Error,
                    });
                    contains_invalid_key_types = true;
                }
            }

            if contains_invalid_key_types {
                errors.push(Error{
                    message: format!(
                        "invalid dictionary key type: struct '{}' contains members that cannot be used as a dictionary key type",
                        struct_def.identifier(),
                    ),
                    location: Some(type_ref.location.clone()),
                    severity: ErrorLevel::Error
                });
                errors.push(Error {
                    message: format!("struct '{}' is defined here:", struct_def.identifier()),
                    location: Some(struct_def.location.clone()),
                    severity: ErrorLevel::Note,
                });
                return false;
            }
            return true;
        }
        Types::Class(class_def) => (false, Some(class_def)),
        Types::Exception(exception_def) => (false, Some(exception_def)),
        Types::Interface(interface_def) => (false, Some(interface_def)),
        Types::Enum(_) => (true, None),
        Types::Trait(trait_def) => (false, Some(trait_def)),
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

        errors.push(Error {
            message: format!(
                "invalid dictionary key type: {} cannot be used as a dictionary key type",
                pluralized_kind,
            ),
            location: Some(type_ref.location.clone()),
            severity: ErrorLevel::Error,
        });

        // If the key type is a user-defined type, point to where it was defined.
        if let Some(named_symbol_def) = named_symbol {
            errors.push(Error {
                message: format!(
                    "{} '{}' is defined here:",
                    named_symbol_def.kind(),
                    named_symbol_def.identifier(),
                ),
                location: Some(named_symbol_def.location().clone()),
                severity: ErrorLevel::Note,
            });
        }
    }
    is_valid
}
