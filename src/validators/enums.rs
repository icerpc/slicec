// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::error::Error;
use crate::grammar::*;
use crate::validators::{Validate, ValidationChain, ValidationResult};

pub fn enum_validators() -> ValidationChain {
    vec![
        Validate::Enums(backing_type_bounds),
        Validate::Enums(allowed_underlying_types),
        Validate::Enums(enumerators_are_unique),
        Validate::Enums(underlying_type_cannot_be_optional),
        Validate::Enums(nonempty_if_checked),
    ]
}

/// Validate that the enumerators are within the bounds of the specified underlying type.
fn backing_type_bounds(enum_def: &Enum) -> ValidationResult {
    let mut errors = vec![];
    if enum_def.supported_encodings().supports(&Encoding::Slice1) {
        // Enum was defined in a Slice1 file.
        // Slice1 does not allow negative numbers.
        enum_def
            .enumerators()
            .iter()
            .filter(|enumerator| enumerator.value < 0)
            .for_each(|enumerator| {
                errors.push(Error {
                    message: format!(
                        "invalid enumerator value on enumerator `{}`: enumerators must be non-negative",
                        &enumerator.identifier()
                    ),
                    location:  Some(enumerator.location.clone()),
                    severity: crate::error::ErrorLevel::Error,
                });
            });
        // Enums in Slice1 always have an underlying type of int32.
        enum_def
                .enumerators()
                .iter()
                .filter(|enumerator| enumerator.value > i32::MAX as i64)
                .for_each(|enumerator| {
                    errors.push(Error {
                        message: format!(
                            "invalid enumerator value on enumerator `{identifier}`: must be smaller than than {max}",
                            identifier = &enumerator.identifier(),
                            max = i32::MAX,
                        ),
                        location:  Some(enumerator.location.clone()),
                        severity: crate::error::ErrorLevel::Error,
                    });
                });
        match errors.is_empty() {
            true => Ok(()),
            false => Err(errors),
        }
    } else {
        // Enum was defined in a Slice2 file.
        // Non-integrals are handled by `allowed_underlying_types`
        fn check_bounds(enum_def: &Enum, underlying_type: &Primitive, errors: &mut Vec<Error>) {
            let (min, max) = underlying_type.numeric_bounds().unwrap();
            enum_def
                .enumerators()
                .iter()
                .map(|enumerator| enumerator.value)
                .filter(|value| *value < min || *value > max)
                .for_each(|value| {
                    errors.push(Error {
                        message: format!(
                            "enumerator value '{value}' is out of bounds. The value must be between `{min}..{max}`, inclusive, for the underlying type `{underlying}`",
                            value = value,
                            underlying=underlying_type.kind(),
                            min = min,
                            max = max,
                        ),
                        location:  Some(enum_def.location.clone()),
                        severity: crate::error::ErrorLevel::Error,
                    });
                });
        }

        match &enum_def.underlying {
            Some(underlying_type) => {
                if underlying_type.is_integral() {
                    check_bounds(enum_def, underlying_type, &mut errors);
                }
            }
            None => {
                // No underlying type, the default is varint32 for Slice2.
                check_bounds(enum_def, &Primitive::VarInt32, &mut errors);
            }
        }
        match errors.is_empty() {
            true => Ok(()),
            false => Err(errors),
        }
    }
}

/// Validate that the backing type specified for a Slice2 enums is an integral type.
fn allowed_underlying_types(enum_def: &Enum) -> ValidationResult {
    let mut errors = vec![];
    if enum_def.supported_encodings().supports(&Encoding::Slice1) {
        return Ok(());
    }
    match &enum_def.underlying {
        Some(underlying_type) => {
            if !underlying_type.is_integral() {
                errors.push(Error {
                    message: format!(
                        "underlying type '{underlying}' is not allowed for enums",
                        underlying = underlying_type.definition().kind(),
                    ),
                    location: Some(enum_def.location.clone()),
                    severity: crate::error::ErrorLevel::Error,
                });
            }
        }
        None => (), // No underlying type, the default is varint32 for Slice2 which is integral.
    }
    match errors.is_empty() {
        true => Ok(()),
        false => Err(errors),
    }
}

/// Validate that the enumerators for an enum are unique.
fn enumerators_are_unique(enum_def: &Enum) -> ValidationResult {
    // The enumerators must be sorted by value first as we are using windowing to check the
    // n + 1 enumerator against the n enumerator. If the enumerators are sorted by value then
    // the windowing will reveal any duplicate enumerators.
    let mut errors = vec![];
    let enumerators = enum_def.enumerators();
    let mut sorted_enumerators = enumerators.clone();
    sorted_enumerators.sort_by_key(|m| m.value);
    sorted_enumerators.windows(2).for_each(|window| {
        if window[0].value == window[1].value {
            errors.push(Error {
                message: format!(
                    "invalid enumerator value on enumerator `{}`: enumerators must be unique",
                    &window[1].identifier()
                ),
                location: Some(window[1].location.clone()),
                severity: crate::error::ErrorLevel::Error,
            });
            errors.push(Error {
                message: format!(
                    "The enumerator `{}` has previous used the value `{}`",
                    &window[0].identifier(),
                    window[0].value
                ),
                location: Some(window[0].location.clone()),
                severity: crate::error::ErrorLevel::Error,
            });
        }
    });
    match errors.is_empty() {
        true => Ok(()),
        false => Err(errors),
    }
}

/// Validate the the underlying type of an enum is not optional.
fn underlying_type_cannot_be_optional(enum_def: &Enum) -> ValidationResult {
    let mut errors = vec![];
    if let Some(ref typeref) = enum_def.underlying {
        if typeref.is_optional {
            errors.push(Error {
                message: format!("underlying type '{}' cannot be optional: enums cannot have optional underlying types", typeref.type_string),
                location: Some(enum_def.location.clone()),
                severity: crate::error::ErrorLevel::Error,
            });
        }
    }
    match errors.is_empty() {
        true => Ok(()),
        false => Err(errors),
    }
}

/// Validate that a checked enum must not be empty.
fn nonempty_if_checked(enum_def: &Enum) -> ValidationResult {
    let mut errors = vec![];
    if !enum_def.is_unchecked && enum_def.enumerators.is_empty() {
        errors.push(Error {
            message: "enums must contain at least one enumerator".to_owned(),
            location: Some(enum_def.location.clone()),
            severity: crate::error::ErrorLevel::Error,
        });
    }
    match errors.is_empty() {
        true => Ok(()),
        false => Err(errors),
    }
}
