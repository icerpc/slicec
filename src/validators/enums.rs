// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::diagnostics::{DiagnosticReporter, Error, ErrorKind};
use crate::grammar::*;
use super::ValidatorVisitor;

use std::collections::HashMap;

impl ValidatorVisitor<'_> {
/// Validate that the enumerators are within the bounds of the specified underlying type.
pub(super) fn backing_type_bounds(&mut self, enum_def: &Enum) {
    if enum_def.supported_encodings().supports(&Encoding::Slice1) {
        // Enum was defined in a Slice1 file, so it's underlying type is int32 and its enumerators must be positive.
        for enumerator in enum_def.enumerators() {
            let value = enumerator.value;
            if value < 0 || value > i32::MAX as i128 {
                Error::new(ErrorKind::EnumeratorValueOutOfBounds(
                    enumerator.identifier().to_owned(),
                    value,
                    0,
                    i32::MAX as i128,
                ))
                .set_span(enumerator.span())
                .report(self.diagnostic_reporter);
            }
        }
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
                    Error::new(error)
                        .set_span(enumerator.span())
                        .report(diagnostic_reporter);
                });
        }
        match &enum_def.underlying {
            Some(underlying_type) => {
                if underlying_type.is_integral() {
                    check_bounds(enum_def, underlying_type, self.diagnostic_reporter);
                }
            }
            None => {
                // No underlying type, the default is varint32 for Slice2.
                check_bounds(enum_def, &Primitive::VarInt32, self.diagnostic_reporter);
            }
        }
    }
}

/// Validate that the backing type specified for a Slice2 enums is an integral type.
pub(super) fn allowed_underlying_types(&mut self, enum_def: &Enum) {
    if enum_def.supported_encodings().supports(&Encoding::Slice1) {
        return;
    }
    match &enum_def.underlying {
        Some(underlying_type) => {
            if !underlying_type.is_integral() {
                Error::new(ErrorKind::UnderlyingTypeMustBeIntegral(
                    enum_def.identifier().to_owned(),
                    underlying_type.definition().kind().to_owned(),
                ))
                .set_span(enum_def.span())
                .report(self.diagnostic_reporter);
            }
        }
        None => (), // No underlying type, the default is varint32 for Slice2 which is integral.
    }
}

/// Validate that enumerator values aren't re-used within an enum.
pub(super) fn enumerator_values_are_unique(&mut self, enum_def: &Enum) {
    let mut value_to_enumerator_map: HashMap<i128, &Enumerator> = HashMap::new();
    for enumerator in enum_def.enumerators() {
        // If the value is already in the map, another enumerator already used it. Get that enumerator from the map
        // and emit an error. Otherwise add the enumerator and its value to the map.
        if let Some(alt_enum) = value_to_enumerator_map.get(&enumerator.value) {
            Error::new(ErrorKind::DuplicateEnumeratorValue(enumerator.value))
                .set_span(enumerator.span())
                .add_note(
                    format!("the value was previously used by `{}` here:", alt_enum.identifier()),
                    Some(alt_enum.span()),
                )
                .report(self.diagnostic_reporter);
        } else {
            value_to_enumerator_map.insert(enumerator.value, enumerator);
        }
    }
}

/// Validate the the underlying type of an enum is not optional.
pub(super) fn underlying_type_cannot_be_optional(&mut self, enum_def: &Enum) {
    if let Some(ref typeref) = enum_def.underlying {
        if typeref.is_optional {
            Error::new(ErrorKind::CannotUseOptionalUnderlyingType(
                enum_def.identifier().to_owned(),
            ))
            .set_span(enum_def.span())
            .report(self.diagnostic_reporter);
        }
    }
}

/// Validate that a checked enum must not be empty.
pub(super) fn nonempty_if_checked(&mut self, enum_def: &Enum) {
    if !enum_def.is_unchecked && enum_def.enumerators.is_empty() {
        Error::new(ErrorKind::MustContainEnumerators(enum_def.identifier().to_owned()))
            .set_span(enum_def.span())
            .report(self.diagnostic_reporter);
    }
}
}
