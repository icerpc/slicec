// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::diagnostics::*;
use crate::grammar::*;
use crate::validators::{ValidationChain, Validator};

use std::collections::HashMap;

pub fn enum_validators() -> ValidationChain {
    vec![
        Validator::Enums(backing_type_bounds),
        Validator::Enums(allowed_underlying_types),
        Validator::Enums(enumerator_values_are_unique),
        Validator::Enums(underlying_type_cannot_be_optional),
        Validator::Enums(nonempty_if_checked),
    ]
}

/// Validate that the enumerators are within the bounds of the specified underlying type.
fn backing_type_bounds(enum_def: &Enum, diagnostic_reporter: &mut DiagnosticReporter) {
    if enum_def.supported_encodings().supports(&Encoding::Slice1) {
        // Enum was defined in a Slice1 file.
        // Slice1 does not allow negative numbers.
        enum_def
            .enumerators()
            .iter()
            .filter(|enumerator| enumerator.value < 0)
            .for_each(|enumerator| {
                let error = ErrorKind::MustBePositive("enumerator values".to_owned());
                diagnostic_reporter.report_error(Error::new(error, Some(enumerator.span())));
            });
        // Enums in Slice1 always have an underlying type of int32.
        enum_def
            .enumerators()
            .iter()
            .filter(|enumerator| enumerator.value > i32::MAX as i64)
            .for_each(|enumerator| {
                let error = ErrorKind::EnumeratorValueOutOfBounds(
                    enumerator.identifier().to_owned(),
                    enumerator.value,
                    0,
                    i32::MAX as i64,
                );
                diagnostic_reporter.report_error(Error::new(error, Some(enumerator.span())));
            });
    } else {
        // Enum was defined in a Slice2 file.
        // Non-integrals are handled by `allowed_underlying_types`
        fn check_bounds(enum_def: &Enum, underlying_type: &Primitive, diagnostic_reporter: &mut DiagnosticReporter) {
            let (min, max) = underlying_type.numeric_bounds().unwrap();
            enum_def
                .enumerators()
                .iter()
                .filter(|enumerator| enumerator.value < min || enumerator.value > max)
                .for_each(|enumerator| {
                    let error = ErrorKind::EnumeratorValueOutOfBounds(
                        enumerator.identifier().to_owned(),
                        enumerator.value,
                        min,
                        max,
                    );
                    diagnostic_reporter.report_error(Error::new(error, Some(enumerator.span())));
                });
        }
        match &enum_def.underlying {
            Some(underlying_type) => {
                if underlying_type.is_integral() {
                    check_bounds(enum_def, underlying_type, diagnostic_reporter);
                }
            }
            None => {
                // No underlying type, the default is varint32 for Slice2.
                check_bounds(enum_def, &Primitive::VarInt32, diagnostic_reporter);
            }
        }
    }
}

/// Validate that the backing type specified for a Slice2 enums is an integral type.
fn allowed_underlying_types(enum_def: &Enum, diagnostic_reporter: &mut DiagnosticReporter) {
    if enum_def.supported_encodings().supports(&Encoding::Slice1) {
        return;
    }
    match &enum_def.underlying {
        Some(underlying_type) => {
            if !underlying_type.is_integral() {
                let error = ErrorKind::UnderlyingTypeMustBeIntegral(
                    enum_def.identifier().to_owned(),
                    underlying_type.definition().kind().to_owned(),
                );
                diagnostic_reporter.report_error(Error::new(error, Some(enum_def.span())));
            }
        }
        None => (), // No underlying type, the default is varint32 for Slice2 which is integral.
    }
}

/// Validate that enumerator values aren't re-used within an enum.
fn enumerator_values_are_unique(enum_def: &Enum, diagnostic_reporter: &mut DiagnosticReporter) {
    let mut value_to_enumerator_map: HashMap<i64, &Enumerator> = HashMap::new();
    for enumerator in enum_def.enumerators() {
        // If the value is already in the map, another enumerator already used it. Get that enumerator from the map
        // and emit an error. Otherwise add the enumerator and its value to the map.
        if let Some(other_enumerator) = value_to_enumerator_map.get(&enumerator.value) {
            let error = ErrorKind::DuplicateEnumeratorValue(enumerator.value);
            let note = Note::new(
                format!(
                    "The value was previously used by `{}` here:",
                    other_enumerator.identifier(),
                ),
                Some(other_enumerator.span()),
            );
            diagnostic_reporter.report_error(Error::new_with_notes(error, Some(enumerator.span()), vec![note]));
        } else {
            value_to_enumerator_map.insert(enumerator.value, enumerator);
        }
    }
}

/// Validate the the underlying type of an enum is not optional.
fn underlying_type_cannot_be_optional(enum_def: &Enum, diagnostic_reporter: &mut DiagnosticReporter) {
    if let Some(ref typeref) = enum_def.underlying {
        if typeref.is_optional {
            diagnostic_reporter.report_error(Error::new(
                ErrorKind::CannotUseOptionalUnderlyingType(enum_def.identifier().to_owned()),
                Some(enum_def.span()),
            ));
        }
    }
}

/// Validate that a checked enum must not be empty.
fn nonempty_if_checked(enum_def: &Enum, diagnostic_reporter: &mut DiagnosticReporter) {
    if !enum_def.is_unchecked && enum_def.enumerators.is_empty() {
        diagnostic_reporter.report_error(Error::new(
            ErrorKind::MustContainEnumerators(enum_def.identifier().to_owned()),
            Some(enum_def.span()),
        ));
    }
}
