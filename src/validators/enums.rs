// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{Diagnostic, Diagnostics, Error};
use crate::grammar::*;

use std::collections::HashMap;

pub fn validate_enum(enum_def: &Enum, diagnostics: &mut Diagnostics) {
    backing_type_bounds(enum_def, diagnostics);
    allowed_underlying_types(enum_def, diagnostics);
    enumerator_values_are_unique(enum_def, diagnostics);
    underlying_type_cannot_be_optional(enum_def, diagnostics);
    nonempty_if_checked(enum_def, diagnostics);
}

/// Validate that the enumerators are within the bounds of the specified underlying type.
fn backing_type_bounds(enum_def: &Enum, diagnostics: &mut Diagnostics) {
    if enum_def.supported_encodings().supports(Encoding::Slice1) {
        // Enum was defined in a Slice1 file, so it's underlying type is int32 and its enumerators must be positive.
        for enumerator in enum_def.enumerators() {
            let value = enumerator.value();
            if value < 0 || value > i32::MAX as i128 {
                Diagnostic::new(Error::EnumeratorValueOutOfBounds {
                    enumerator_identifier: enumerator.identifier().to_owned(),
                    value,
                    min: 0,
                    max: i32::MAX as i128,
                })
                .set_span(enumerator.span())
                .push_into(diagnostics);
            }
        }
    } else {
        // Enum was defined in a Slice2 file.
        // Non-integrals are handled by `allowed_underlying_types`
        fn check_bounds(enum_def: &Enum, underlying_type: &Primitive, diagnostics: &mut Diagnostics) {
            let (min, max) = underlying_type.numeric_bounds().unwrap();
            enum_def
                .enumerators()
                .iter()
                .filter(|enumerator| enumerator.value() < min || enumerator.value() > max)
                .for_each(|enumerator| {
                    let error = Error::EnumeratorValueOutOfBounds {
                        enumerator_identifier: enumerator.identifier().to_owned(),
                        value: enumerator.value(),
                        min,
                        max,
                    };
                    Diagnostic::new(error)
                        .set_span(enumerator.span())
                        .push_into(diagnostics);
                });
        }
        match &enum_def.underlying {
            Some(underlying_type) => {
                if underlying_type.is_integral() {
                    check_bounds(enum_def, underlying_type, diagnostics);
                }
            }
            None => {
                // No underlying type, the default is varint32 for Slice2.
                check_bounds(enum_def, &Primitive::VarInt32, diagnostics);
            }
        }
    }
}

/// Validate that the backing type (if present) is an integral type.
fn allowed_underlying_types(enum_def: &Enum, diagnostics: &mut Diagnostics) {
    match &enum_def.underlying {
        Some(underlying_type) => {
            if !underlying_type.is_integral() {
                Diagnostic::new(Error::EnumUnderlyingTypeNotSupported {
                    enum_identifier: enum_def.identifier().to_owned(),
                    kind: Some(underlying_type.definition().kind().to_owned()),
                })
                .set_span(enum_def.span())
                .push_into(diagnostics);
            }
        }
        None => (), // No underlying type, the default is varint32 for Slice2 which is integral.
    }
}

/// Validate that enumerator values aren't re-used within an enum.
fn enumerator_values_are_unique(enum_def: &Enum, diagnostics: &mut Diagnostics) {
    let mut value_to_enumerator_map: HashMap<i128, &Enumerator> = HashMap::new();
    for enumerator in enum_def.enumerators() {
        // If the value is already in the map, another enumerator already used it. Get that enumerator from the map
        // and report an error. Otherwise add the enumerator and its value to the map.
        if let Some(alt_enum) = value_to_enumerator_map.get(&enumerator.value()) {
            Diagnostic::new(Error::DuplicateEnumeratorValue {
                enumerator_value: enumerator.value(),
            })
            .set_span(enumerator.span())
            .add_note(
                format!("the value was previously used by '{}' here:", alt_enum.identifier()),
                Some(alt_enum.span()),
            )
            .push_into(diagnostics);
        } else {
            value_to_enumerator_map.insert(enumerator.value(), enumerator);
        }
    }
}

/// Validate the the underlying type of an enum is not optional.
fn underlying_type_cannot_be_optional(enum_def: &Enum, diagnostics: &mut Diagnostics) {
    if let Some(ref typeref) = enum_def.underlying {
        if typeref.is_optional {
            Diagnostic::new(Error::CannotUseOptionalUnderlyingType {
                enum_identifier: enum_def.identifier().to_owned(),
            })
            .set_span(enum_def.span())
            .push_into(diagnostics);
        }
    }
}

/// Validate that a checked enum must not be empty.
fn nonempty_if_checked(enum_def: &Enum, diagnostics: &mut Diagnostics) {
    if !enum_def.is_unchecked && enum_def.enumerators.is_empty() {
        Diagnostic::new(Error::MustContainEnumerators {
            enum_identifier: enum_def.identifier().to_owned(),
        })
        .set_span(enum_def.span())
        .push_into(diagnostics);
    }
}
